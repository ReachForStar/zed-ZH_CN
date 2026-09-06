//! SQLite 数据库读取/编辑的数据层。
//!
//! 与 office_preview 的只读文档不同，SQLite 查看器需要按绝对路径
//! 反复打开数据库执行查询与更新，因此本模块不持有长连接，而是在
//! 每次操作时于后台线程打开只读（或临时可写）连接完成任务。

use std::path::Path;

use anyhow::{Context as _, Result, anyhow};
use rusqlite::{Connection, OpenFlags, Row, types::Value as SqlValue};

/// SQLite 魔数：文件头 16 字节（含 NUL）
pub const SQLITE_MAGIC: &[u8; 16] = b"SQLite format 3\0";

/// 判断一个扩展名是否可能为 SQLite 数据库文件
pub fn is_sqlite_extension(ext: &str) -> bool {
    matches!(ext.to_lowercase().as_str(), "db" | "sqlite" | "sqlite3")
}

/// 判断路径后缀对应的扩展名
pub fn extension_of(path: &Path) -> Option<&str> {
    path.extension().and_then(|e| e.to_str())
}

/// 供 UI 展示的数据库对象类别
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DbObjectKind {
    Table,
    View,
    Index,
    Trigger,
}

impl DbObjectKind {
    pub fn type_name(&self) -> &'static str {
        match self {
            DbObjectKind::Table => "table",
            DbObjectKind::View => "view",
            DbObjectKind::Index => "index",
            DbObjectKind::Trigger => "trigger",
        }
    }

    pub fn type_sql(&self) -> &'static str {
        match self {
            DbObjectKind::Table => "table",
            DbObjectKind::View => "view",
            DbObjectKind::Index => "index",
            DbObjectKind::Trigger => "trigger",
        }
    }
}

/// 一个数据库对象（表/视图/索引/触发器）
#[derive(Clone, Debug)]
pub struct DbObject {
    pub name: String,
    pub kind: DbObjectKind,
    /// 所属表名（索引/触发器才有）
    pub tbl_name: Option<String>,
    /// sqlite_master.sql 原始建表语句
    pub sql: Option<String>,
}

/// 单个列的信息
#[derive(Clone, Debug)]
pub struct ColumnInfo {
    pub name: String,
    /// 声明类型（可能为空，SQLite 无强制类型）
    pub type_name: String,
    pub not_null: bool,
    pub is_pk: bool,
    /// 是否在 rowid 表中有 rowid 别名（INTEGER PRIMARY KEY）
    pub is_rowid_alias: bool,
}

/// 单元格值（面向 UI 的简单封装）
#[derive(Clone, Debug, PartialEq)]
pub enum CellValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl CellValue {
    /// 转成可显示字符串；NULL 显示为空
    pub fn display(&self) -> String {
        match self {
            CellValue::Null => String::new(),
            CellValue::Integer(v) => v.to_string(),
            CellValue::Real(v) => format!("{v}"),
            CellValue::Text(v) => v.clone(),
            CellValue::Blob(b) => format!("<BLOB:{}字节>", b.len()),
        }
    }

    /// 从 rusqlite 值转换（BLOB 只保留前若干字节避免内存爆炸）
    pub fn from_sql(value: SqlValue, blob_preview_len: usize) -> Self {
        match value {
            SqlValue::Null => CellValue::Null,
            SqlValue::Integer(v) => CellValue::Integer(v),
            SqlValue::Real(v) => CellValue::Real(v),
            SqlValue::Text(v) => CellValue::Text(v),
            SqlValue::Blob(b) => {
                let mut v = b;
                v.truncate(blob_preview_len);
                CellValue::Blob(v)
            }
        }
    }
}

/// 一行数据：列名对齐的单元格值数组
pub struct DbRow {
    pub values: Vec<CellValue>,
}

/// 表数据分页查询结果
pub struct TablePage {
    pub columns: Vec<ColumnInfo>,
    /// 当前页行数据
    pub rows: Vec<DbRow>,
    /// 可继续翻页
    pub has_more: bool,
    /// 若表存在显式 INTEGER PRIMARY KEY 或 rowid，编辑/分页使用该键
    pub rowid_alias: Option<String>,
}

/// 打开连接（后台线程使用）。`writable` 为 false 时使用只读模式，
/// 避免误改用户数据库；执行写操作前临时以可写方式重开。
fn open_connection(path: &str, writable: bool) -> Result<Connection> {
    let flags = if writable {
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX
    } else {
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX
    };
    let conn = Connection::open_with_flags(path, flags)
        .with_context(|| format!("无法打开 SQLite 数据库：{path}"))?;
    conn.busy_timeout(std::time::Duration::from_millis(500))?;
    Ok(conn)
}

/// 列出库中所有对象（排除 sqlite_ 内部表）
pub fn list_objects(path: &str) -> Result<Vec<DbObject>> {
    let conn = open_connection(path, false)?;
    let mut stmt = conn.prepare(
        "SELECT name, type, tbl_name, sql FROM sqlite_master \
         WHERE name NOT LIKE 'sqlite_%' ORDER BY type, name",
    )?;
    let objects = stmt
        .query_map([], |row| {
            let name: String = row.get(0)?;
            let type_: String = row.get(1)?;
            let tbl_name: Option<String> = row.get(2)?;
            let sql: Option<String> = row.get(3)?;
            Ok((name, type_, tbl_name, sql))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(objects
        .into_iter()
        .filter_map(|(name, type_, tbl_name, sql)| {
            let kind = match type_.as_str() {
                "table" => DbObjectKind::Table,
                "view" => DbObjectKind::View,
                "index" => DbObjectKind::Index,
                "trigger" => DbObjectKind::Trigger,
                _ => return None,
            };
            Some(DbObject {
                name,
                kind,
                tbl_name,
                sql,
            })
        })
        .collect())
}

/// 获取表/视图的列信息（PRAGMA table_info）
pub fn get_columns(path: &str, table: &str) -> Result<Vec<ColumnInfo>> {
    let conn = open_connection(path, false)?;
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let cols = stmt
        .query_map([], |row| {
            let cid: i64 = row.get(0)?;
            let _ = cid;
            let name: String = row.get(1)?;
            let type_name: String = row.get(2)?;
            let not_null: bool = row.get(3)?;
            let dflt: Option<String> = row.get(4)?;
            let _ = dflt;
            let pk: i64 = row.get(5)?;
            let is_rowid_alias = pk > 0 && type_name.eq_ignore_ascii_case("integer");
            Ok(ColumnInfo {
                name,
                type_name,
                not_null,
                is_pk: pk > 0,
                // INTEGER PRIMARY KEY 在 rowid 表中即 rowid 别名
                is_rowid_alias,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(cols)
}

/// 取表的总行数（用于分页状态显示）
pub fn count_rows(path: &str, table: &str) -> Result<i64> {
    let conn = open_connection(path, false)?;
    // 表名来自 sqlite_master，非用户输入拼接，但保守起见校验标识符
    let quoted = quote_identifier(table);
    let n: i64 = conn.query_row(&format!("SELECT COUNT(*) FROM {quoted}"), [], |r| r.get(0))?;
    Ok(n)
}

/// 分页读取表数据
pub fn load_page(
    path: &str,
    table: &str,
    columns: &[ColumnInfo],
    offset: i64,
    limit: i64,
) -> Result<TablePage> {
    let conn = open_connection(path, false)?;
    let col_list = columns
        .iter()
        .map(|c| quote_identifier(&c.name))
        .collect::<Vec<_>>()
        .join(", ");
    let table_q = quote_identifier(table);
    let sql = format!("SELECT {col_list} FROM {table_q} LIMIT {limit} OFFSET {offset}");
    let mut stmt = conn.prepare(&sql)?;
    let rowid_alias = columns
        .iter()
        .find(|c| c.is_rowid_alias)
        .map(|c| c.name.clone());

    let mut rows = Vec::new();
    let mut has_more = false;
    {
        let mut query = stmt.query([])?;
        // 多取一行判断 has_more
        while let Some(row) = query.next()? {
            if rows.len() == limit as usize {
                has_more = true;
                break;
            }
            rows.push(read_row(row)?);
        }
    }
    Ok(TablePage {
        columns: columns.to_vec(),
        rows,
        has_more,
        rowid_alias,
    })
}

fn read_row(row: &Row) -> Result<DbRow> {
    let mut values = Vec::with_capacity(row.as_ref().column_count());
    // BLOB 只保留预览长度（显示用），避免超长字段拖垮 UI
    const BLOB_PREVIEW: usize = 256 * 1024;
    for idx in 0..row.as_ref().column_count() {
        let v = row.get::<_, SqlValue>(idx)?;
        values.push(CellValue::from_sql(v, BLOB_PREVIEW));
    }
    Ok(DbRow { values })
}

/// 生成 UPDATE 语句更新一个单元格。
///
/// 定位行使用 rowid（或 INTEGER PRIMARY KEY 别名）；若无 rowid（如
/// WITHOUT ROWID 表或视图），返回错误提示，视图层禁用编辑。
pub fn update_cell(
    path: &str,
    table: &str,
    columns: &[ColumnInfo],
    column_idx: usize,
    row: &[CellValue],
    new_value: &str,
) -> Result<()> {
    if new_value.len() > 64 * 1024 {
        return Err(anyhow!("单元格内容过大，拒绝写入"));
    }
    let col = columns
        .get(column_idx)
        .ok_or_else(|| anyhow!("列索引越界"))?;
    let rowid_idx = columns.iter().position(|c| c.is_rowid_alias);
    // rowid 表：优先用 rowid 定位
    let conn = open_connection(path, true)?;

    // 定位键：若无 INTEGER PRIMARY KEY 别名则尝试隐藏 rowid
    let where_clause = if let Some(pk_col) = rowid_idx {
        // 主键列位于 row 数组中的索引
        let key_value = row.get(pk_col).ok_or_else(|| anyhow!("无法定位主键值"))?;
        format!(
            "{} = {}",
            quote_identifier(&columns[pk_col].name),
            quote_literal(key_value)?
        )
    } else {
        // 用 rowid 定位；rowid 表总是有 rowid（除非 WITHOUT ROWID）
        let is_without_rowid: bool = conn.query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1 AND sql LIKE '%WITHOUT ROWID%'",
            [table],
            |_| Ok(true),
        )
        .unwrap_or(false);
        if is_without_rowid {
            return Err(anyhow!("WITHOUT ROWID 表暂不支持单元格编辑"));
        }
        // 先查该行的 rowid
        let rowid = lookup_rowid(&conn, table, columns, row)?;
        format!("rowid = {rowid}")
    };

    let table_q = quote_identifier(table);
    let col_q = quote_identifier(&col.name);
    let val = if col.type_name.eq_ignore_ascii_case("blob") {
        // BLOB 类型列：写入文本按 UTF-8 存为 BLOB？不——按普通文本处理会类型不匹配。
        // 此处仍按文本处理并让 SQLite 自行转换；需要二进制编辑的用户请用 SQL。
        return Err(anyhow!("BLOB 列暂不支持单元格编辑"));
    } else {
        // 空字符串 vs NULL 由视图层决定（传 NULL 标记）
        quote_text(new_value)
    };

    let sql = format!("UPDATE {table_q} SET {col_q} = {val} WHERE {where_clause}");
    conn.execute(&sql, [])?;
    Ok(())
}

/// 查询行的 rowid（非 INTEGER PRIMARY KEY 表的定位辅助）
fn lookup_rowid(
    conn: &Connection,
    table: &str,
    columns: &[ColumnInfo],
    row: &[CellValue],
) -> Result<i64> {
    // 用整行值做条件不可靠（重复行、NULL 等），改为按行号查询：
    // 这里调用方传入当前行在结果集中的绝对偏移不安全。
    // 更稳做法：直接以 rowid 选择，但 ui 层只拿到分页数据。
    // 折中：遍历表按列值比对，仅用于精确匹配且值唯一时；失败给提示。
    let table_q = quote_identifier(table);
    let conds = columns
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let v = row.get(i)?;
            let lit = quote_literal(v).ok()?;
            Some(format!("{} IS {}", quote_identifier(&c.name), lit))
        })
        .collect::<Vec<_>>()
        .join(" AND ");
    let sql = format!("SELECT rowid FROM {table_q} WHERE {conds} LIMIT 1");
    conn.query_row(&sql, [], |r| r.get(0))
        .map_err(|_| anyhow!("无法在表中唯一定位该行，单元格编辑被取消"))
}

fn quote_identifier(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

/// 值字面量（用于定位键）
fn quote_literal(v: &CellValue) -> Result<String> {
    Ok(match v {
        CellValue::Null => "NULL".into(),
        CellValue::Integer(i) => i.to_string(),
        CellValue::Real(f) => format!("{f}"),
        CellValue::Text(s) => quote_text(s),
        CellValue::Blob(_) => return Err(anyhow!("无法用 BLOB 值定位行")),
    })
}

fn quote_text(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db() -> (tempfile::TempDir, String) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db").to_string_lossy().into_owned();
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, age INTEGER);
             INSERT INTO users (name, age) VALUES ('Alice', 30), ('Bob', 25);",
        )
        .unwrap();
        (dir, path)
    }

    #[test]
    fn detect_magic() {
        assert!(is_sqlite_extension("db"));
        assert!(is_sqlite_extension("SQLITE3"));
        assert!(!is_sqlite_extension("txt"));
    }

    #[test]
    fn list_and_read() {
        let (_dir, path) = temp_db();
        let objs = list_objects(&path).unwrap();
        assert!(
            objs.iter()
                .any(|o| o.name == "users" && o.kind == DbObjectKind::Table)
        );

        let cols = get_columns(&path, "users").unwrap();
        assert_eq!(cols.len(), 3);
        assert!(cols[0].is_pk && cols[0].is_rowid_alias);

        let page = load_page(&path, "users", &cols, 0, 10).unwrap();
        assert_eq!(page.rows.len(), 2);
        assert_eq!(page.rows[0].values[1], CellValue::Text("Alice".into()));
    }

    #[test]
    fn update_cell_works() {
        let (_dir, path) = temp_db();
        let cols = get_columns(&path, "users").unwrap();
        let page = load_page(&path, "users", &cols, 0, 10).unwrap();
        let alice_row = page
            .rows
            .iter()
            .find(|r| r.values[1] == CellValue::Text("Alice".into()))
            .unwrap();
        update_cell(&path, "users", &cols, 1, &alice_row.values, "Alicia").unwrap();

        let cols2 = get_columns(&path, "users").unwrap();
        let page2 = load_page(&path, "users", &cols2, 0, 10).unwrap();
        assert!(
            page2
                .rows
                .iter()
                .any(|r| r.values[1] == CellValue::Text("Alicia".into()))
        );
    }
}
