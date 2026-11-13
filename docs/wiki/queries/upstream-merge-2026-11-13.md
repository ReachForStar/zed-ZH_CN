---
title: 2026-11-13 合并 upstream/main（1.30.0，76 提交）：冲突消解与 fork 特性核对
type: query
tags: [upstream-merge, conflict, i18n, fork]
created: 2026-11-13
updated: 2026-11-13
status: active
---

# 2026-11-13 合并 upstream/main

## 范围

- 上游基线：`upstream/main` 的 `27bbbc1cc4`（上游 `Cargo.toml` 为 1.30.0），距上次合并（`b5cd310b3b`，1.29.0）新增 76 个提交，`git diff --stat` 943 个文件。
- 合并落点：本仓库 `main`，版本行保持 fork 的 `1.29.1`（`crates/zed/Cargo.toml`、`Cargo.lock`）。

## 冲突与判定

9 处冲突：

| 文件 | 判定 |
| --- | --- |
| `.github/workflows/triage_queue_board.yml` | fork 已删除辅助 workflow（上游改它）→ 取删除，仍只保留 `release.yml` + `run_tests.yml` |
| `crates/zed/Cargo.toml` | 上游 1.30.0 × fork 1.29.1 → 取 fork |
| `Cargo.lock` | 取上游侧，`crates/zed` 版本改回 1.29.1，`cargo metadata` 重新裁剪（新增 merman、serde-saphyr、aws-lc-sys 等条目） |
| `crates/git_ui/src/branch_picker.rs` | 冲突在 `gpui` 导入列表：fork 仍用 `PromptButton::new(t!(...))` 做强删确认 → 保留 `PromptButton` |
| `crates/open_path_prompt/src/open_path_prompt.rs` | 上游新增 `OpenPathPromptMode`/`OpenPathPromptResult` 导入 × fork 的 `use zed_i18n::t` → 两者合并 |
| `crates/settings_ui/src/page_data.rs` | 上游新增 `title_bar.show` 设置项、把 `file_diff_section` 扩成 `commit_view_section`（新增 Expand Description / Expand Files）→ 取上游结构并汉化新增项 |
| `crates/tabular_data_preview/src/renderer/preview_view.rs` | 上游新增 `extend_selection`/`clear_selection` 动作 × fork 的表内设置面板挂载 → 两者都保留（面板仍挂在 `TableView`） |
| `crates/workspace/src/security_modal.rs` | 上游把内联 `trusted = Some(false); dismiss()` 提为 `stay_restricted_and_dismiss`，并把远端主机标签提为 `remote_host_label`（覆盖 SSH/WSL/Docker）→ 取上游 |
| `crates/zed/src/main.rs` | 导入合并（`handle_cli_request` + fork 的 `handle_ui_language_changes`）；恢复失败的 toast 文案保留 fork 的 `t!`，上游新调用点同样本地化，删除因此失去引用的英文 `restore_errors_message` |

## 汉化

新增 7 个键（en / zh-CN 同步）：

- `settings_ui.page_data.commit_view_section`（Git 提交视图）
- `git_commit_view_expand_description_{title,description}`
- `git_commit_view_expand_files_{title,description}`
- `title_bar_show_title_bar_{title,description}`

删除失效键 `settings_ui.page_data.file_diff_section`（分节改名）。`scripts/i18n/check-missing-keys.py`：代码静态引用 3993 键、en/zh-CN 各 4054 键、0 问题。

## fork 特性核对

- `AcpThread::resolve_location` 的目录跳过（`project.fs().metadata()`）仍在；
- `drop_compacted_entries`（压缩后释放条目，`InProgress` 作屏障）仍在；
- `maybe_trigger_auto_compact` 与三档日志仍在；
- `crates/fs` 的 `check_watch_health` / `WatchBackend::watched_paths` 仍在；
- `tabular_data_preview` 表内设置面板仍由 `TableView` 渲染；
- `third_party/notify` path vendor 保留；
- `README.md` 顶部两行 `[!IMPORTANT]` 完好。

## 验证

- `cargo check -p zed`：通过。
- `cargo clippy -p zed -p acp_thread -p agent_ui -p agent_servers -p settings_ui -p git_ui -p workspace -p tabular_data_preview -p open_path_prompt --all-targets -- -D warnings`：通过（首轮暴露 `branch_picker` 的 `TaskExt` 未使用导入，删除后全绿）。
- 测试：acp_thread 316、agent_servers 198、settings_ui 76、tabular_data_preview 23 全过；git_ui 首轮 212 过 / 1 失败（`test_history_tab_focus_border_only_for_keyboard_focus`），已定位并修复，修后 213 全过（见下节）。
- `rustfmt --check`（本次编辑的 6 个文件）：通过。
- 本地构建注意：webrtc-sys 需 clang（`.cargo/config.toml [env]` 强制 `CC/CXX`，构建后还原）；debug profile 首次编译需要 webrtc 预编译包，网络不可达时用 `LK_CUSTOM_WEBRTC` 指向 release 产物目录。

## 已知失败的真因：焦点句柄重复创建（2026-11-13 修复）

`test_history_tab_focus_border_only_for_keyboard_focus` 自 2026-10 起被记为「本地环境失败」，实际是早前合并冲突引入的缺陷：`GitPanel` 构造函数在局部 `focus_handle` 上注册 `cx.on_focus(..., Self::focus_in)`，结构体字段却写成 `focus_handle: cx.focus_handle()` 另建了一个句柄。两者 id 不同，真实界面与测试聚焦的都是字段句柄，`focus_in` 从未触发，键盘焦点边框永远不出现。上游结构体字段是同一句柄的简写 `focus_handle,`。

修复：`crates/git_ui/src/git_panel.rs` 改回 `focus_handle,`。验证：单测通过、`git_ui` 213 项全过、`clippy -p git_ui -- -D warnings` 与 fmt 通过。提交 `a111afd787`。

全仓同类问题扫描（脚本按三种规则做函数级/文件级匹配，先用旧版 `git_panel.rs` 自检确认能命中）：

1. **构造函数订阅的局部句柄 × 结构体另建 `focus_handle: cx.focus_handle()`**：修复后 0 处（自检旧版命中 1 处）。
2. **局部句柄只用于 `on_focus*` 订阅、随后未存储也未 `track_focus`**：6 处，全部是订阅子实体/面板自身的句柄（`split.rs` 的左右编辑器、`commit_modal` 的提交编辑器、`recent_projects` 的 picker、`buffer_search` 的可搜索条目、`workspace::add_panel` 的 panel handle），非缺陷。
3. **`impl Focusable` 返回子句柄**：98 处，均为委托模式（`picker` / `editor` / `state` 等），与本次缺陷不同类。

结论：除已修复的 `git_panel` 外没有同类句柄重复创建问题。

## 其他缺陷扫描（2026-11-13）

- **全工作区测试目标编译**：`cargo check --workspace --all-targets` 通过（0 错误），排除「合并只破坏测试目标」的遗留。
- **feature 门控**：`cargo check -p fs --features test-support --all-targets` 通过，符合本页记录过的「fork 给共享 trait 加方法 × 上游新增 test 后端」检查要求。
- **新增未汉化文案计数对比**（按 `Label::new("…")`、`Tooltip::text("…")`、`.title("…")` 等模式比较合并前后每个 rs 文件）：仅 `agent_ui/src/conversation_view/thread_view.rs` 新增 2 处裸字面量（`Open Session` / `Unavailable Session`），对应键 `agent_ui.thread_view.open_session` / `unavailable_session` 已在 en/zh-CN 中存在，已改用 `t!`；`conversation_view.rs` 的 `Delegate` / `Investigator` 是测试夹具数据，不计入。
- **新增 panic / unsafe 抽查**：本轮新增的 `panic!` 全部位于测试；新增 `unsafe` 均为 macOS objc 桥接代码（上游）。
- **焦点句柄三类扫描**：见上节，除 `git_panel` 外无同类问题。

## 复发预防

- 上游把 `commit_view_section` 一类的分节数组扩容时，汉化键要跟着增删，并用 `check-missing-keys` 双向核对。
- debug 与 release 的 webrtc 预编译目录不共享，断网环境先给 debug 也准备一份，或用 `LK_CUSTOM_WEBRTC`。
- 解决构造函数冲突时，核对「注册监听用的 focus handle」与「结构体字段保存的 handle」是否为同一个：上游惯用 `focus_handle` 简写，误写成 `focus_handle: cx.focus_handle()` 会让 `on_focus`/`on_blur` 静默失效，只有聚焦行为类测试能发现。