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
- 测试：acp_thread 316、agent_servers 198、settings_ui 76、tabular_data_preview 23 全过；git_ui 212 过、1 个已知本地失败 `git_panel::tests::test_history_tab_focus_border_only_for_keyboard_focus`（上游本轮未改该逻辑，属既有本地环境失败）。
- `rustfmt --check`（本次编辑的 6 个文件）：通过。
- 本地构建注意：webrtc-sys 需 clang（`.cargo/config.toml [env]` 强制 `CC/CXX`，构建后还原）；debug profile 首次编译需要 webrtc 预编译包，网络不可达时用 `LK_CUSTOM_WEBRTC` 指向 release 产物目录。

## 复发预防

- 上游把 `commit_view_section` 一类的分节数组扩容时，汉化键要跟着增删，并用 `check-missing-keys` 双向核对。
- debug 与 release 的 webrtc 预编译目录不共享，断网环境先给 debug 也准备一份，或用 `LK_CUSTOM_WEBRTC`。