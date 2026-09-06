//! SQLite 数据库查看/编辑器（.db / .sqlite / .sqlite3）。
//!
//! 仿照 `office_preview` 的 ProjectItem 分发模式：注册后按扩展名与文件头
//! 魔数拦截文件打开流程，打开 SQLite 数据库并渲染为可浏览、可编辑的视图。
//!
//! 数据层见 [`db`]：每次操作在后台线程打开（只读或临时可写）连接，不持有
//! 长连接，避免阻塞 UI 且降低误写风险。

pub mod db;
mod document;
mod view;

pub use document::SqliteDatabase;
pub use view::SqliteViewer;

use gpui::App;

pub fn init(cx: &mut App) {
    workspace::register_project_item::<SqliteViewer>(cx);
    workspace::register_serializable_item::<SqliteViewer>(cx);
}
