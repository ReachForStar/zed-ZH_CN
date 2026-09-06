//! 项目侧模型：一个已识别为 SQLite 的数据库文件。
//!
//! 仅保存文件定位元信息；数据库连接在每次渲染/操作时由视图层按需打开。

use anyhow::{Context as _, Result};
use gpui::{App, AppContext as _, AsyncApp, Entity, Task};
use project::{Project, ProjectEntryId, ProjectPath};

use crate::db::{SQLITE_MAGIC, is_sqlite_extension};

/// SQLite 数据库项目项。
pub struct SqliteDatabase {
    entry_id: Option<ProjectEntryId>,
    project_path: ProjectPath,
    /// 魔数校验结果；false 表示扩展名匹配但内容不是 SQLite（视图层提示）
    pub is_sqlite: bool,
}

impl SqliteDatabase {
    pub fn project_path(&self) -> &ProjectPath {
        &self.project_path
    }

    /// 解析数据库的绝对路径（本地文件），在同步 `App` 上下文中调用。
    pub fn resolve_abs_path(&self, project: &Project, cx: &App) -> Result<String> {
        let worktree = project
            .worktree_for_id(self.project_path.worktree_id, cx)
            .context("数据库所在 worktree 不存在")?;
        let abs = worktree.read(cx).absolutize(&self.project_path.path);
        Ok(abs.to_string_lossy().into_owned())
    }
}

impl project::ProjectItem for SqliteDatabase {
    fn try_open(
        project: &Entity<Project>,
        path: &ProjectPath,
        cx: &mut App,
    ) -> Option<Task<Result<Entity<Self>>>> {
        // 1. 扩展名初筛：不匹配直接放行给默认编辑器
        let ext = path.path.extension()?.to_lowercase();
        if !is_sqlite_extension(&ext) {
            return None;
        }

        let project = project.clone();
        let path = path.clone();
        Some(cx.spawn(
            async move |cx: &mut AsyncApp| -> Result<Entity<SqliteDatabase>> {
                // 同步段：取绝对路径
                let abs_path = cx.update(|cx| {
                    let this = SqliteDatabase {
                        entry_id: None,
                        project_path: path.clone(),
                        is_sqlite: false,
                    };
                    let project_ref = project.read(cx);
                    this.resolve_abs_path(project_ref, cx)
                })?;

                // 后台段：读文件头校验魔数
                let is_sqlite = cx
                    .background_executor()
                    .spawn(async move {
                        use std::io::Read;
                        let mut f = std::fs::File::open(&abs_path)?;
                        let mut head = [0u8; 16];
                        let n = f.read(&mut head)?;
                        Ok::<_, anyhow::Error>(
                            n >= SQLITE_MAGIC.len() && &head[..16] == SQLITE_MAGIC,
                        )
                    })
                    .await?;

                // 同步段：查询项目入口并创建实体
                let entry_id = project.update(cx, |project, cx| {
                    project.entry_for_path(&path, cx).map(|e| e.id)
                });
                Ok(cx.update(|cx| {
                    cx.new(|_| SqliteDatabase {
                        entry_id,
                        project_path: path,
                        is_sqlite,
                    })
                }))
            },
        ))
    }

    fn entry_id(&self, _cx: &App) -> Option<ProjectEntryId> {
        self.entry_id
    }

    fn project_path(&self, _cx: &App) -> Option<ProjectPath> {
        Some(self.project_path.clone())
    }

    fn is_dirty(&self) -> bool {
        false
    }
}
