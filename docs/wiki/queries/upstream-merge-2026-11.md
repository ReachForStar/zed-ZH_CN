---
title: 2026-11 合并上游 main：冲突消解、编译失败链与 fork 特性核对
type: query
tags: [fork, upstream, merge, clippy, i18n, acp]
created: 2026-11-08
updated: 2026-11-08
status: active
---

# 2026-11 合并上游 main：冲突消解、编译失败链与 fork 特性核对

## 问题

把 upstream/main（`b5cd310b3b5`，`Cargo.toml` 版本 1.29.0，与上次合并基线 `422b53cff8` 之间 77 个提交）合并进本仓库 main，保留汉化与 fork 自有特性。版本号仍取 fork 侧 `1.27.0`。

## 冲突消解与关键决策

- **版本号**：`crates/zed/Cargo.toml` 取 fork 的 `1.27.0`；`Cargo.lock` 取上游后用 `cargo metadata` 重新裁剪。
- **workflow**：`.github/workflows/` 整体取 fork（仅 `release.yml` + `run_tests.yml`），删除上游新增的 `community_pr_board.yml`、`community_pr_board_refresh.yml`、`slack_notify_community_automation_failure.yml`。
- **ACP 协议升级**：`agent-client-protocol` 升到 5.0.0，模块改为 `agent_client_protocol::v2::schema as acp_v2`。
- **plan 渲染重构**（`agent_ui/src/conversation_view/thread_view.rs`）：取上游拆分的 `render_plan_summary` / `render_plan_body` / `render_plan_entries`，复制按钮改 `CopyButton` + `agent_message_copy_index`；写死文案改 `t!`。
- **git 面板分支模式**（`git_ui/src/git_panel.rs`、新文件 `git_panel/branch_mode.rs`）：取上游 `Section::Branch`、`changes_mode` 菜单；`EntryMenuTitles` 字段改成 `SharedString` 并包 `t!`；`show_checkbox` 相关两处按上游 `when` 包裹。
- **git 日期格式**（`settings_ui/src/page_data.rs`、`blame_ui.rs`）：取上游 `date_format` / `date_style` 设置与 `git_timestamp::format_git_date`，删除 fork 旧 `blame_entry_relative_timestamp`。
- **搜索落地按钮**（`crates/search`）：`render_landing_action_button` 形参改 `impl Into<SharedString>`。
- **`open_output` / `show_error_toast` 下沉**：上游把这两个函数从 `git_ui::git_panel` 移到 `git_ui_core::notifications`；合并时删掉 fork 侧旧定义与两个已迁走的测试，改 `pub(crate) use` 重新导出，避免 `E0255` 重复定义。

## 编译失败链（`cargo check -p zed` 逐轮收敛）

1. **E0433 `PromptButton` 未声明**：`crates/workspace/src/workspace.rs` 的 `gpui::{...}` 导入漏了 `PromptButton`（上游已把它从 `PromptButton::new(...).into()` 改成 `window.prompt(..., &["..."], cx)`，但 fork 汉化版仍用 `PromptButton`）。
2. **残留冲突标记**：`settings_ui/src/page_data.rs` 有三处 `>>>>>>> upstream/main`，导致 `expected expression, found >>` 与大面积 `E0308`；删标记后剩余类型错误才是真错误。
3. **`SettingItem` 字段类型**：上游 `title/description` 是 `&'static str`，fork 是 `SharedString`；取上游新代码后裸字面量必须 `t!(...).into()`。
4. **match 分支 `&str` 与 `Cow` 混用**（`git_ui/src/branch_picker.rs`）：`placeholder_text` 的 `PickerState::List if ... => "Select a branch…"` 与相邻分支的 `t!` 返回类型不同；该字面量补 `t!("git_ui.branch_picker.placeholder_select_branch")` 键。
5. **`Fn` 闭包移动 `SharedString`**（`git_ui/src/git_graph.rs`）：`Disclosure::tooltip` 的回调会被多次调用，捕获的 `label` 必须 `clone()`。
6. **`agent_ui` 活动状态文案**（`thread_view.rs`）：`(_, SharedString, _)` 元组里 `t!` 返回 `Cow`，补 `.into()`；同处「1 个/多个后台任务」与提示文案改 `t!` 键（`task_in_background`、`tasks_in_background`、`background_work_tooltip`）。

## 汉化补齐

- 新增 `[settings_ui.page_data]` 键 49 个（37 个新增设置项/分节 + 冲突消解引入的 12 个 date_style/date_format/edit_prediction_button 键）（`suggest_extensions`、`markdown_preview_heading_font_weight`、`mermaid_font_family`、`file_finder_prefill_query_from_selection`、`title_bar_open_menus_on_hover`、`fullscreen_mode`、`terminal_starts_open`、`git_panel_starts_open`、`git_panel_commit_editor`、`debugger_default_width/height`、`git_stash_picker_date_style`、`git_graph_date_style`、`git_diff_base`、`agent_max_idle_retained_threads`、`agent_auto_generate_thread_titles`、`languages_soft_wrap_indent` 及 4 个分节标题），en 与 zh-CN 同步。
- 新增 `[git_ui.git_panel]` 分支模式键 8 个、`[git_ui.git_graph]` 4 个、`[git_ui.commit_context_menu]` 1 个、`[git_ui.branch_picker]` 1 个、`[git_ui_core]` 1 个、`[agent_ui.thread_view]` 3 个。
- 「上游新增但未汉化」的比对方法：对本次变动的 rs 文件，统计 `Label::new("…")`/`Tooltip::text("…")`/`SectionHeader("…")`/`.entry("…")` 等模式的**未 `t!` 化命中数**，与 `HEAD` 同文件比较；只有计数上升才是本轮引入。结果仅 `crates/ui/src/components/context_menu/preview.rs` 上升，属组件内部预览画廊，不是产品界面。
- 已知历史欠账（非本轮引入）：`language_models` 的 ollama / llama.cpp / lmstudio 设置页、`edit_prediction_ui` 的部分 Copilot 菜单、`settings_ui/src/pages/tool_permissions_setup.rs` 等仍有硬编码英文，`HEAD` 与合并结果计数相同，未在本次处理。
- `python3 scripts/i18n/check-missing-keys.py` 最终 0 问题（代码静态引用 3987 键，en/zh-CN 各 4048 键）。

## fork 特性核对（上游是否已实现同类修复）

| fork 修复 | 上游状态 | 处理 |
| --- | --- | --- |
| `drop_compacted_entries`（压缩完成后释放被摘要覆盖的条目，`d852cc1723`、边界修正 `2494d1a3b4`） | `upstream/main` 无该符号 | 保留 fork |
| `maybe_trigger_auto_compact` / `auto_compact_armed` / `is_native`（外部智能体自动压缩，`8f2bde6f4e`） | 上游无对应实现 | 保留 fork |
| git 面板折叠条目过滤与 Changes/History 焦点导航（`2cc0bda01a`） | 上游无（合并后仍用 fork 的 `section_is_collapsed` / `activation_focus_handle` 判定） | 保留 fork |
| 标题内 KaTeX 字号（`327f731db0`） | 上游没有 KaTeX 模块（fork 自有特性） | 保留 fork |
| 解析未完成时的标题滚动（`bc71984dbe`、`fb319c5401`） | 上游已正式实现 `pending_heading_scroll` + `defer_parsing` | 采用上游 |
| 写入路径混淆字符双路径扫描（`e26a09420f`） | 上游就有该检查 | 采用上游 |
| `typos` 配置路径、`run_tests` 手动触发、`scripts/normalize-crlf.py` 位置 | 均为 fork CI 自维护 | 保留 fork |

## 本地校验

```sh
CC=clang-22 CXX=clang++-22 CXXFLAGS=-fpermissive cargo check -p zed
CC=clang-22 CXX=clang++-22 CXXFLAGS=-fpermissive cargo clippy --workspace --all-targets -- --deny warnings
cargo fmt --all -- --check
python3 scripts/i18n/check-missing-keys.py
LIBRARY_PATH=$PWD/tmp/libshim cargo test -p acp_thread -p agent_servers -p git_ui -p settings_ui
```

- `libX11-xcb.so` shim：系统只有 `.so.1`，测试链接需要无版本号的名字，建 `tmp/libshim/libX11-xcb.so` 软链并设 `LIBRARY_PATH`。
- rustfmt 写 LF，提交前 `python3 scripts/normalize-crlf.py` 还原 CRLF。

结果：`cargo check -p zed` 通过；全量 `clippy --deny warnings` 通过；`cargo fmt --check` 通过；汉化键检查 0 问题；`acp_thread` 312 通过、`agent_servers` 196 通过（1 忽略，含 fork 的 `test_auto_compact_*` 两个用例）、`agent_ui` 585 通过、`settings_ui` 76 通过、`git_ui` 210 通过（1 个已知本地失败，见下）。

已知本地失败：`git_ui::tests::test_history_tab_focus_border_only_for_keyboard_focus`（210 通过 / 1 失败）。该测试及其依赖的 `focus_in`、`history_keyboard_nav`、`focused_history_entry` 代码与 `upstream/main` 完全一致（`git diff upstream/main -- crates/git_ui/src/git_panel.rs` 在这些符号上无差异），属测试环境（`simulate_keystrokes("f19")` 后 `window.last_input_was_keyboard()`）问题，与 fork 改动无关，留给上游 CI 核对。

## 会话启动性能（同批排查）

见 [新建智能体会话慢](slow-new-agent-session.md)：主因是 ACP registry 智能体二进制首次下发（`opencode-v2` 2.2.6，258 MB，约 97 秒），不是 fork 或合并引入的回归。

## 关联页面

- [汉化 fork 同步 upstream 的合并模式与特性保留清单](../concepts/fork-upstream-sync.md)
- [2026-10 合并上游 main](upstream-merge-2026-10.md)
- [ACP 面板自动压缩上下文不生效](acp-auto-compact-external-agents.md)
- [新建智能体会话慢](slow-new-agent-session.md)
