//! SQLite 数据库查看/编辑视图。
//!
//! 布局：顶部为表/视图切换条；主体为固定表头 + 虚拟化数据行 + 分页栏；
//! 底部为单元格编辑行（Editor::single_line）。单击单元格进入编辑态，
//! 失焦或按 Enter 提交写库后自动刷新当前页。

use std::sync::Arc;

use anyhow::{Result, anyhow};
use editor::{Editor, EditorEvent, actions::SelectAll};
use file_icons::FileIcons;
use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, EntityId, EventEmitter, FocusHandle,
    Focusable, FontWeight, IntoElement, ParentElement, Pixels, Render, SharedString, Styled,
    Subscription, Task, TextStyleRefinement, WeakEntity, Window, div, px, uniform_list,
};
use project::Project;
use settings::Settings as _;
use ui::{
    ActiveTheme as _, Button, ButtonCommon as _, ButtonSize, ButtonStyle, Color, Label, LabelSize,
    TextSize, TintColor, Tooltip, prelude::*,
};
use workspace::item::{
    Item, ProjectItem as WorkspaceProjectItem, SerializableItem, TabContentParams,
};
use workspace::{ItemId, ItemSettings, Pane, Workspace, WorkspaceId};
use zed_i18n::t;

use crate::db::{
    CellValue, ColumnInfo, DbObject, DbObjectKind, count_rows, get_columns, list_objects,
    load_page, update_cell,
};
use crate::document::SqliteDatabase;

const ROW_NUMBER_WIDTH: Pixels = px(64.0);
const CELL_WIDTH: Pixels = px(200.0);
const ROW_HEIGHT: Pixels = px(24.0);
const PAGE_SIZE: i64 = 100;

/// 表格状态
enum TableState {
    Loading,
    Ready {
        columns: Arc<Vec<ColumnInfo>>,
        rows: Arc<Vec<Vec<CellValue>>>,
        total: i64,
        page: i64,
        has_more: bool,
        has_prev: bool,
    },
    Error(String),
    Empty,
}

/// 页数据（后台加载结果）
struct PagePayload {
    columns: Vec<ColumnInfo>,
    rows: Vec<Vec<CellValue>>,
    total: i64,
    page: i64,
    has_more: bool,
}

pub struct SqliteViewer {
    document: Entity<SqliteDatabase>,
    project: Entity<Project>,
    focus_handle: FocusHandle,
    objects: Vec<DbObject>,
    active_object: Option<String>,
    table: TableState,
    /// 编辑中单元格 (row_in_page, col)
    editing_cell: Option<(usize, usize)>,
    cell_editor: Option<Entity<Editor>>,
    cell_editor_value: SharedString,
    _subscription: Option<Subscription>,
    loading: bool,
}

impl SqliteViewer {
    pub fn new(
        document: Entity<SqliteDatabase>,
        project: Entity<Project>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut this = Self {
            document,
            project,
            focus_handle: cx.focus_handle(),
            objects: Vec::new(),
            active_object: None,
            table: TableState::Loading,
            editing_cell: None,
            cell_editor: None,
            cell_editor_value: SharedString::default(),
            _subscription: None,
            loading: false,
        };
        this.reload_objects(window, cx);
        this
    }

    fn db_path(&self, cx: &App) -> String {
        let project = self.project.read(cx);
        self.document
            .read(cx)
            .resolve_abs_path(project, cx)
            .unwrap_or_else(|e| format!("无法解析路径: {e}"))
    }

    /// 后台加载对象列表
    fn reload_objects(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.loading {
            return;
        }
        let path = self.db_path(cx);
        let weak = cx.weak_entity();
        self.loading = true;
        cx.spawn(async move |_, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { list_objects(&path) })
                .await;
            weak.update(cx, |this, cx| {
                this.loading = false;
                match result {
                    Ok(objects) => {
                        this.objects = objects;
                        let first = this
                            .objects
                            .iter()
                            .find(|o| matches!(o.kind, DbObjectKind::Table | DbObjectKind::View))
                            .map(|o| o.name.clone());
                        if let Some(name) = first {
                            this.active_object = Some(name);
                            this.table = TableState::Loading;
                            // 加载首页数据
                            this.load_page_data(0, cx);
                        } else {
                            this.table = TableState::Empty;
                        }
                        cx.notify();
                    }
                    Err(e) => {
                        this.table = TableState::Error(e.to_string());
                        cx.notify();
                    }
                }
            })
            .ok();
        })
        .detach();
    }

    /// 加载指定页数据（active_object 必须已设置）
    fn load_page_data(&mut self, page: i64, cx: &mut Context<Self>) {
        let Some(name) = self.active_object.clone() else {
            return;
        };
        if self.loading {
            return;
        }
        let path = self.db_path(cx);
        let weak = cx.weak_entity();
        let page_no = page.max(0);
        let offset = page_no * PAGE_SIZE;
        self.loading = true;
        self.table = TableState::Loading;
        cx.spawn(async move |_, cx| {
            let result: Result<PagePayload> = cx
                .background_executor()
                .spawn({
                    let path = path.clone();
                    let name = name.clone();
                    async move {
                        let columns = get_columns(&path, &name)?;
                        let total = if columns.is_empty() {
                            0
                        } else {
                            count_rows(&path, &name)?
                        };
                        let p = load_page(&path, &name, &columns, offset, PAGE_SIZE)?;
                        let rows: Vec<Vec<CellValue>> =
                            p.rows.into_iter().map(|r| r.values).collect();
                        Ok(PagePayload {
                            columns,
                            rows,
                            total,
                            page: page_no,
                            has_more: p.has_more,
                        })
                    }
                })
                .await;
            weak.update(cx, |this, cx| {
                this.loading = false;
                match result {
                    Ok(payload) => {
                        this.table = TableState::Ready {
                            columns: Arc::new(payload.columns),
                            rows: Arc::new(payload.rows),
                            total: payload.total,
                            page: payload.page,
                            has_more: payload.has_more,
                            has_prev: payload.page > 0,
                        };
                    }
                    Err(e) => this.table = TableState::Error(e.to_string()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn switch_object(&mut self, name: String, _window: &mut Window, cx: &mut Context<Self>) {
        if self.active_object.as_deref() == Some(name.as_str()) {
            return;
        }
        self.active_object = Some(name);
        self.editing_cell = None;
        self.cell_editor = None;
        self._subscription = None;
        self.load_page_data(0, cx);
    }

    fn go_to_page(&mut self, page: i64, cx: &mut Context<Self>) {
        let TableState::Ready { page: current, .. } = &self.table else {
            return;
        };
        let target = (*current + page).max(0);
        self.load_page_data(target, cx);
    }

    /// 创建单元格编辑用 Editor（在 Render 期懒调用）
    fn spawn_cell_editor(&mut self, text: String, window: &mut Window, cx: &mut Context<Self>) {
        let editor_value: SharedString = text.into();
        let editor = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_text(editor_value.clone().to_string(), window, cx);
            editor.set_text_style_refinement(TextStyleRefinement {
                color: Some(cx.theme().colors().text),
                font_size: Some(TextSize::Small.rems(cx).into()),
                ..Default::default()
            });
            editor.select_all(&SelectAll, window, cx);
            editor
        });
        let subscription = cx.subscribe_in(&editor, window, {
            move |this, editor, event, window, cx| match event {
                EditorEvent::Edited { .. } => {
                    this.cell_editor_value = editor.read(cx).text(cx).into();
                    cx.notify();
                }
                EditorEvent::Blurred => {
                    // 值有变化则提交，否则取消编辑
                    let changed = this
                        .cell_display_text(cx)
                        .map(|old| old.as_str() != this.cell_editor_value.as_ref())
                        .unwrap_or(true);
                    if changed {
                        this.commit_cell_edit(window, cx);
                    } else {
                        this.cancel_cell_edit(cx);
                    }
                }
                _ => {}
            }
        });
        self.cell_editor_value = editor_value;
        editor.focus_handle(cx).focus(window, cx);
        self.cell_editor = Some(editor);
        self._subscription = Some(subscription);
        cx.notify();
    }

    fn cancel_cell_edit(&mut self, cx: &mut Context<Self>) {
        self.editing_cell = None;
        self.cell_editor = None;
        self._subscription = None;
        cx.notify();
    }

    /// 提交编辑值写库，成功后重载当前页
    fn commit_cell_edit(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let Some((row, col)) = self.editing_cell else {
            self.cancel_cell_edit(cx);
            return;
        };
        let Some(name) = self.active_object.clone() else {
            return;
        };
        let TableState::Ready {
            columns,
            rows,
            page,
            ..
        } = &self.table
        else {
            return;
        };
        let Some(orig_row) = rows.get(row) else {
            return;
        };
        let columns = columns.clone();
        let orig_row = orig_row.clone();
        let page = *page;
        let new_value = self.cell_editor_value.to_string();
        let path = self.db_path(cx);

        self.editing_cell = None;
        self.cell_editor = None;
        self._subscription = None;

        let weak = cx.weak_entity();
        cx.spawn(async move |_, cx| {
            let result = cx
                .background_executor()
                .spawn(
                    async move { update_cell(&path, &name, &columns, col, &orig_row, &new_value) },
                )
                .await;
            weak.update(cx, |this, cx| match result {
                Ok(()) => {
                    // 关闭 loading 以便再次加载
                    this.loading = false;
                    this.table = TableState::Loading;
                    this.load_page_data(page, cx);
                    cx.notify();
                }
                Err(e) => {
                    this.loading = false;
                    this.table = TableState::Error(e.to_string());
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    fn cell_display_text(&self, _cx: &App) -> Option<String> {
        let (row, col) = self.editing_cell?;
        let TableState::Ready { rows, .. } = &self.table else {
            return None;
        };
        rows.get(row).and_then(|r| r.get(col)).map(|v| v.display())
    }

    // ---------- 渲染 ----------

    fn render_centered_message(&self, message: SharedString) -> AnyElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child(Label::new(message).color(Color::Muted))
            .into_any_element()
    }

    fn object_label(obj: &DbObject) -> String {
        match obj.kind {
            DbObjectKind::View => format!("{} (视图)", obj.name),
            _ => obj.name.clone(),
        }
    }

    fn render_top_bar(&self, cx: &mut Context<Self>) -> AnyElement {
        let active = self.active_object.clone().unwrap_or_default();
        let chips: Vec<AnyElement> = self
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| matches!(o.kind, DbObjectKind::Table | DbObjectKind::View))
            .map(|(idx, obj)| {
                let selected = obj.name == active;
                let name = obj.name.clone();
                Button::new(("db-object", idx), Self::object_label(obj))
                    .size(ButtonSize::Compact)
                    .toggle_state(selected)
                    .selected_style(ButtonStyle::Tinted(TintColor::Accent))
                    .tooltip(Tooltip::text(match obj.kind {
                        DbObjectKind::Table => t!("sqlite_viewer.open_table").into(),
                        DbObjectKind::View => t!("sqlite_viewer.open_view").into(),
                        _ => SharedString::default(),
                    }))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.switch_object(name.clone(), window, cx);
                    }))
                    .into_any_element()
            })
            .collect();

        h_flex()
            .id("sqlite-viewer-top")
            .px_2()
            .py_1()
            .gap_2()
            .items_center()
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .child(
                Label::new(t!("sqlite_viewer.title"))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(
                div()
                    .id("db-object-chips")
                    .flex_1()
                    .overflow_x_scroll()
                    .child(h_flex().gap_1().children(chips)),
            )
            .into_any_element()
    }

    fn render_table_header(&self, columns: &[ColumnInfo], cx: &App) -> AnyElement {
        let border = cx.theme().colors().border;
        let bg = cx.theme().colors().panel_background;
        h_flex()
            .h(ROW_HEIGHT)
            .flex_none()
            .child(
                div()
                    .w(ROW_NUMBER_WIDTH)
                    .h_full()
                    .flex_none()
                    .px_2()
                    .bg(bg)
                    .border_r_1()
                    .border_color(border)
                    .child(
                        Label::new(t!("sqlite_viewer.row_number"))
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    ),
            )
            .children(columns.iter().map(|col| {
                let text = if col.is_pk {
                    format!("{} 🔑", col.name)
                } else {
                    col.name.clone()
                };
                div()
                    .w(CELL_WIDTH)
                    .h_full()
                    .flex_none()
                    .px_2()
                    .bg(bg)
                    .border_r_1()
                    .border_color(border)
                    .child(Label::new(text).single_line().weight(FontWeight::MEDIUM))
            }))
            .into_any_element()
    }

    fn render_table(&self, cx: &mut Context<Self>) -> AnyElement {
        match &self.table {
            TableState::Loading => self.render_centered_message(t!("sqlite_viewer.loading").into()),
            TableState::Empty => {
                self.render_centered_message(t!("sqlite_viewer.no_objects").into())
            }
            TableState::Error(e) => {
                self.render_centered_message(t!("sqlite_viewer.error", error = e.clone()).into())
            }
            TableState::Ready {
                columns,
                rows,
                total,
                has_more,
                has_prev,
                ..
            } => {
                let col_count = columns.len();
                let table_width = ROW_NUMBER_WIDTH + CELL_WIDTH * col_count as f32;
                let header = self.render_table_header(columns, cx);
                let row_count = rows.len();
                let body: AnyElement = if row_count == 0 {
                    self.render_centered_message(t!("sqlite_viewer.no_rows").into())
                } else {
                    let rows = rows.clone();
                    let view = cx.weak_entity();
                    uniform_list("sqlite-viewer-rows", row_count, move |range, window, cx| {
                        range
                            .map(|index| {
                                Self::render_row(&view, &rows, index, col_count, window, cx)
                            })
                            .collect()
                    })
                    .flex_1()
                    .w(table_width)
                    .into_any_element()
                };
                let pagination = self.render_pagination(*total, *has_prev, *has_more, cx);
                v_flex()
                    .id("sqlite-viewer-body")
                    .flex_1()
                    .bg(cx.theme().colors().editor_background)
                    .child(
                        div()
                            .id("sqlite-viewer-scroll")
                            .flex_1()
                            .overflow_x_scroll()
                            .child(v_flex().w(table_width).h_full().child(header).child(body)),
                    )
                    .child(pagination)
                    .into_any_element()
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_row(
        view: &WeakEntity<SqliteViewer>,
        rows: &Arc<Vec<Vec<CellValue>>>,
        index: usize,
        col_count: usize,
        _window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let border = cx.theme().colors().border;
        let row = rows.get(index).cloned().unwrap_or_default();
        h_flex()
            .id(("sqlite-row", index))
            .h(ROW_HEIGHT)
            .flex_none()
            .child(
                div()
                    .w(ROW_NUMBER_WIDTH)
                    .h_full()
                    .flex_none()
                    .px_2()
                    .border_r_1()
                    .border_color(border)
                    .child(
                        Label::new((index + 1).to_string())
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    ),
            )
            .children((0..col_count).map(|col| {
                let value = row.get(col).cloned().unwrap_or(CellValue::Null);
                let display = value.display();
                let is_null = matches!(value, CellValue::Null);
                let view = view.clone();
                div()
                    .id(("sqlite-cell", index * 1000 + col))
                    .w(CELL_WIDTH)
                    .h_full()
                    .flex_none()
                    .px_2()
                    .border_r_1()
                    .border_color(border)
                    .cursor_pointer()
                    .on_click(move |_ev, _window, cx| {
                        if let Some(view) = view.upgrade() {
                            view.update(cx, |this, cx| {
                                // 值存于 rows；启动行编辑（render 中懒建 Editor）
                                this.editing_cell = Some((index, col));
                                cx.notify();
                            });
                        }
                    })
                    .child(Label::new(display).single_line().color(if is_null {
                        Color::Placeholder
                    } else {
                        Color::Muted
                    }))
                    .into_any_element()
            }))
            .into_any_element()
    }

    fn render_pagination(
        &self,
        total: i64,
        has_prev: bool,
        has_more: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let page = match &self.table {
            TableState::Ready { page, .. } => *page,
            _ => 0,
        };
        let info = if total <= 0 {
            t!("sqlite_viewer.row_count_zero").to_string()
        } else {
            let start = page * PAGE_SIZE + 1;
            let end = ((page + 1) * PAGE_SIZE).min(total);
            t!(
                "sqlite_viewer.row_range",
                start = start,
                end = end,
                total = total
            )
        };
        h_flex()
            .px_2()
            .py_1()
            .gap_2()
            .items_center()
            .justify_between()
            .border_t_1()
            .border_color(cx.theme().colors().border)
            .child(Label::new(info).color(Color::Muted).size(LabelSize::Small))
            .child(
                h_flex()
                    .gap_1()
                    .child(
                        Button::new("sqlite-prev", t!("sqlite_viewer.prev_page"))
                            .size(ButtonSize::Compact)
                            .disabled(!has_prev)
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.go_to_page(-1, cx);
                            })),
                    )
                    .child(
                        Button::new("sqlite-next", t!("sqlite_viewer.next_page"))
                            .size(ButtonSize::Compact)
                            .disabled(!has_more)
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.go_to_page(1, cx);
                            })),
                    ),
            )
            .into_any_element()
    }

    fn render_editor_bar(&self, cx: &mut Context<Self>) -> AnyElement {
        let label: SharedString = match self.editing_cell {
            Some((row, col)) => {
                let col_name = match &self.table {
                    TableState::Ready { columns, .. } => columns
                        .get(col)
                        .map(|c| c.name.as_str())
                        .unwrap_or_default(),
                    _ => "",
                };
                t!("sqlite_viewer.editing_cell", row = row + 1, col = col_name).into()
            }
            None => t!("sqlite_viewer.click_cell_to_edit").into(),
        };
        let editor_element: AnyElement = if let Some(editor) = &self.cell_editor {
            editor.clone().into_any_element()
        } else {
            div().into_any_element()
        };
        h_flex()
            .id("sqlite-viewer-editor-bar")
            .px_2()
            .py_1()
            .gap_2()
            .items_center()
            .border_t_1()
            .border_color(cx.theme().colors().border)
            .child(Label::new(label).color(Color::Muted).size(LabelSize::Small))
            .child(
                div()
                    .id("sqlite-cell-editor")
                    .flex_1()
                    .child(editor_element),
            )
            .child(
                Button::new("sqlite-cancel-edit", t!("sqlite_viewer.cancel"))
                    .size(ButtonSize::Compact)
                    .disabled(self.editing_cell.is_none())
                    .on_click(cx.listener(|this, _, _window, cx| {
                        this.cancel_cell_edit(cx);
                    })),
            )
            .into_any_element()
    }
}

impl EventEmitter<()> for SqliteViewer {}

impl Render for SqliteViewer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 单击单元格后（editing_cell 已设置），在渲染期懒建 Editor
        if self.editing_cell.is_some() && self.cell_editor.is_none() {
            let text = self.cell_display_text(cx).unwrap_or_default();
            self.spawn_cell_editor(text, window, cx);
        }

        let base = v_flex()
            .id("sqlite-viewer")
            .track_focus(&self.focus_handle)
            .key_context("SqliteViewer")
            .size_full()
            .bg(cx.theme().colors().editor_background);

        if !self.document.read(cx).is_sqlite {
            return base
                .child(self.render_centered_message(t!("sqlite_viewer.not_a_sqlite_db").into()))
                .into_any_element();
        }

        base.child(self.render_top_bar(cx))
            .child(self.render_table(cx))
            .child(self.render_editor_bar(cx))
            .into_any_element()
    }
}

impl Focusable for SqliteViewer {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Item for SqliteViewer {
    type Event = ();

    fn tab_content_text(&self, _detail: usize, cx: &App) -> SharedString {
        self.document
            .read(cx)
            .project_path()
            .path
            .file_name()
            .unwrap_or_else(|| "SQLite")
            .to_owned()
            .into()
    }

    fn tab_tooltip_text(&self, cx: &App) -> Option<SharedString> {
        Some(self.db_path(cx).into())
    }

    fn tab_icon(&self, _window: &Window, cx: &App) -> Option<Icon> {
        let db_path = self.db_path(cx);
        let path = std::path::Path::new(&db_path);
        ItemSettings::get_global(cx)
            .file_icons
            .then(|| FileIcons::get_icon(path, cx))
            .flatten()
            .map(Icon::from_path)
    }

    fn for_each_project_item(
        &self,
        cx: &App,
        f: &mut dyn FnMut(EntityId, &dyn project::ProjectItem),
    ) {
        f(self.document.entity_id(), self.document.read(cx))
    }

    fn clone_on_split(
        &self,
        _workspace_id: Option<WorkspaceId>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<Option<Entity<Self>>>
    where
        Self: Sized,
    {
        let document = self.document.clone();
        let project = self.project.clone();
        Task::ready(Some(cx.new(|cx| Self::new(document, project, window, cx))))
    }

    fn tab_content(&self, params: TabContentParams, _window: &Window, cx: &App) -> AnyElement {
        Label::new(self.tab_content_text(params.detail.unwrap_or_default(), cx))
            .single_line()
            .color(params.text_color())
            .when(params.preview, |label| label.italic())
            .into_any_element()
    }

    fn show_toolbar(&self) -> bool {
        false
    }
}

impl WorkspaceProjectItem for SqliteViewer {
    type Item = SqliteDatabase;

    fn for_project_item(
        project: Entity<Project>,
        _pane: Option<&Pane>,
        item: Entity<Self::Item>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::new(item, project, window, cx)
    }
}

impl SerializableItem for SqliteViewer {
    fn serialized_item_kind() -> &'static str {
        "SqliteViewer"
    }

    fn deserialize(
        _project: Entity<Project>,
        _workspace: WeakEntity<Workspace>,
        _workspace_id: WorkspaceId,
        _item_id: ItemId,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Task<anyhow::Result<Entity<Self>>> {
        Task::ready(Err(anyhow!("SqliteViewer 不支持会话恢复")))
    }

    fn cleanup(
        _workspace_id: WorkspaceId,
        _alive_items: Vec<ItemId>,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Task<anyhow::Result<()>> {
        Task::ready(Ok(()))
    }

    fn serialize(
        &mut self,
        _workspace: &mut Workspace,
        _item_id: ItemId,
        _closing: bool,
        _cx: &mut Context<Self>,
    ) -> Option<Task<anyhow::Result<()>>> {
        None
    }

    fn should_serialize(&self, _event: &Self::Event) -> bool {
        false
    }
}
