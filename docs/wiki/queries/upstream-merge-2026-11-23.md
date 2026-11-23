---
title: 2026-11-23 合并 upstream/main（1.31.0，135 提交）：冲突消解、既有 fork 回归修复与验证
type: query
tags: [upstream-merge, conflict, i18n, fork]
created: 2026-11-23
updated: 2026-11-23
status: active
---

# 2026-11-23 合并 upstream/main

## 范围

- 上游基线：`upstream/main` 的 `f126950f9d`（上游 `crates/zed/Cargo.toml` 为 `1.31.0`），距上次合并（`306d2b31dc`，上游 `27bbbc1cc4`、1.30.0）新增 135 个提交、650 个文件。
- 合并落点：本仓库 `main`，`git merge --no-ff --no-commit`，版本行保持 fork 的 `1.29.1`（版本号仍由用户指定，未改动）。
- 合并提交：`eb75b7af61`（父提交 `4595b8478b` + `f126950f9d`），已推送 fork `main`。

## 冲突与判定

27 个未合并路径：

| 文件 | 判定 |
| --- | --- |
| `.github/workflows/{after_release,community_pr_board_refresh,release_nightly}.yml` | fork 已删除、上游修改 → 取删除，仍只保留 `release.yml` + `run_tests.yml` |
| `.rules`、`AGENTS.md` | 上游删除 `.rules` 并把 Rust 约定并入 `AGENTS.md` → 两者都保留 fork 版本（`.rules` 硬规则、`AGENTS.md` 项目规则）；清理 git 生成的 `AGENTS.md~f126950f9d` 残留 |
| `crates/agent_ui/src/agent_model_selector.rs` | 上游把该模块并入 `language_model_selector` crate → 取删除；`AgentModelSelector` 结构已无引用 |
| `crates/zed/Cargo.toml` | 上游 1.31.0 × fork 1.29.1 → 取 fork |
| `crates/editor/Cargo.toml` | 上游删除 `tree-sitter-python`（改由 `language` 再导出）→ 取上游删除，保留 fork 的 `zed_i18n` |
| `Cargo.lock` | 取上游侧，`zed` 条目改回 1.29.1，`cargo metadata`（全量解析）重新裁剪 |
| `agent_ui/config_options.rs` | 上游 #67190 重排 ACP 配置项渲染，删掉 `current_value_name` / `handles_category_keybindings` / `render_trigger_button` → 取删除，新渲染里的 `"Unknown"` 改 `t!` |
| `agent_ui/conversation_view/thread_view.rs` | `PlanContent::File` 随上游移除 → 删该分支；快速模式与沙箱网络行走上游新结构 + `t!` |
| `git_ui/{git_panel,git_ui,remote_output}.rs`、`git_ui_core/worktree_picker.rs` | `url::Url`、`EntityId` 取上游；`PushPrLink` 变体随上游移除改用 `style`；保留 fork 的 `zed_actions` / `zed_i18n` / `PromptButton` 与 `t!` 文案 |
| `language_model_selector/model_selector_components.rs` | 导入取上游（`CycleFavoriteModels` 并入 `zed_actions::agent`）+ `t` |
| `onboarding`、`toolchain_selector`、`search/project_search`、`workspace/{pane,notifications}`、`project_panel`、`settings_ui/pages/{skill_creator,skills_setup}`、`sidebar` | 取上游新结构 + 保留 fork 本地化；`sidebar` 的工作树子菜单改由 `worktree_creation_base_workspaces` 遍历，新增带工作区名的文案键 |

## 汉化

新增 6 个键（en / zh-CN 同步）：

- `settings_ui.skills_setup.no_global_skills_on_host`（带 `%{remote_host}`）、`shared_scope_global_remote`（上游新增远程机器口径）
- `sidebar.new_thread.create_worktree_in`（`%{workspaces}`）
- `agent_ui.config_options.agent_options`
- `agent_ui.thread_view.open_plan_file`、`cycle_response_speed`

`scripts/i18n/check-missing-keys.py`：代码静态引用 3995 键、en/zh-CN 各 4057 键、0 问题。另按「与上游同文件裸字面量计数对比」核对，本轮未引入新的未汉化文案（`toggle_button.rs` 的 3 处位于 gallery 测试夹具，不计）。

## 编译失败链

1. `language_model_selector` 是上游新拆出的 crate，没有 `zed_i18n` 依赖 → `E0432 unresolved import zed_i18n`。补 `zed_i18n.workspace = true`。
2. `workspace/src/pane.rs` 合并后 `anchored` / `deferred` 失去引用 → `-D warnings` 报 unused import，删除。
3. webrtc-sys 预编译包下载超时：带 `HTTPS_PROXY` 构建，或 `LK_CUSTOM_WEBRTC` 指向已解压目录（debug 与 release 不共享）。

## 既有 fork 回归（本轮一并修复）

合并后 `cargo test -p project_panel` 暴露 2 项失败，根因是早前合并留下的分歧，与本次上游改动无关（`git show HEAD` 已缺相同代码）：

- `test_rename_survives_window_deactivation`：上游 #61852 在 `EditorEvent::Blurred` 加 `window.is_window_active()` 守卫，fork 早前合并时丢失 → 恢复守卫。
- `batch_trash_warns_about_unsaved_changes`：上游已把删除/回收站确认框收敛为 `ProjectPanel::build_removal_prompt(RemovalKind, names, dirty_buffers)`，fork 早前把 `undo.rs` 与 `project_panel.rs` 的内联提示改回旧写法，未保存警告落在 detail 而非 message。本轮恢复共享函数（确认按钮与 detail 用 `t!` / `SharedString` 保留本地化），两处调用点统一。修后 project_panel 150 项全过。

另 `acp_thread` 的 `test_tool_locations_skip_directories` 因上游把 `AgentThreadEntry::location()` 改成「未解析也返回原始位置、解析结果在元组第二项」而失败；目录跳过的 fork 逻辑本身仍在，测试改为断言 `location(i).and_then(|(_, resolved)| resolved).is_none()`。

## 验证

- `cargo check -p zed`：通过。
- `cargo check --workspace --all-targets`：通过（5m44s，排除 merge 只破坏测试目标）。
- `cargo clippy -p zed -p agent_ui -p agent_servers -p acp_thread -p settings_ui -p git_ui -p git_ui_core -p workspace -p tabular_data_preview -p open_path_prompt -p language_model_selector -p onboarding -p project_panel -p search -p editor -p sidebar -p toolchain_selector -p fs -p ui -p markdown -p settings -p project --all-targets -- -D warnings`：通过。
- `cargo fmt --all --check`：通过（CRLF 工作区同样通过）。
- 测试：acp_thread 317、agent_servers 203、agent_skills 95、agent_ui 594、git_ui 214、project_panel 150、settings_ui 72、tabular_data_preview 23、language_model_selector 8、fs 63，以及 editor/markdown/ui/git_ui_core/toolchain_selector/onboarding/sidebar/search 合计 1630 项，全部通过。
- 质量扫描：新增 `panic!` / `unsafe` / `unwrap()` 计数与上游 `27bbbc1cc4..f126950f9d` 完全一致（608 处，全部来自上游）；改动文件的焦点句柄重复创建扫描 0 处。

## fork 特性核对

- `AcpThread::resolve_location` 目录跳过仍在（`project.fs().metadata()`）；
- `drop_compacted_entries`、`maybe_trigger_auto_compact` 仍在；
- `crates/fs` 的 `check_watch_health` / `WatchBackend::watched_paths` 仍在；
- `tabular_data_preview` 表内设置面板仍挂在 `TableView`；
- `settings_ui` 的 `SettingItem.title/description` 仍为 `SharedString`；
- `markdown` 标题内 KaTeX 字号与 `pending_heading_scroll` 仍在（含 fork 的标题字号缓存测试）；
- `third_party/notify` path vendor 保留；
- `README.md` 顶部两行 `[!IMPORTANT]` 完好；
- `.rules` 与 fork 的 `AGENTS.md` 保留（上游 `.rules` 删除未生效）。

## 复发预防

- 上游新增 crate（如 `language_model_selector`）时，fork 的 `zed_i18n` 依赖要跟着加，否则只在编译期暴露。
- 上游重构「内联 UI 逻辑 → 共享构造函数」时，fork 早期的内联副本会静默留下；合并后对 `git diff f126950f9d -- <file>` 检查是否仍有上游已删除的旧实现，并用上游测试兜底。
- 上游改 `#[test]` 期望的文案位置（message 与 detail）时，fork 的旧内联实现会产生「行为对、断言错」的失败，优先恢复上游结构而不是改测试。
- `git merge` 遇到 `.rules`/`AGENTS.md` 会生成 `AGENTS.md~<commit>` 残留文件，解决后必须删除，避免误提交。