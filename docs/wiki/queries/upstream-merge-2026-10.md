---
title: 2026-10 合并上游 main：冲突消解决策与编译失败链
type: query
tags: [fork, upstream, merge, clippy, i18n]
created: 2026-10-16
updated: 2026-10-16
status: active
---

# 2026-10 合并上游 main：冲突消解决策与编译失败链

## 问题

把 upstream/main（`953ac75529`，与上次合并基线 `975845875b` 之间 297 个提交）合并进本仓库 main，同时保留 fork 的汉化与自有特性。合并提交为 `80851bcbbc`，相对 fork HEAD 改动 787 个文件（+114140/−31854）。

## 冲突消解与关键决策

- **workflow**：`.github/workflows/` 整体取 fork（`--ours`），不引入上游新增的 47 个 workflow 文件，保留精简的 `release.yml` + `run_tests.yml`。
- **依赖版本**：保留 fork 的 `calamine = "0.36"`、`ratex-*`、`zip = "8"`；取上游的 `cargo_metadata = "0.23"`、`rand`（`default-features = false`）、`zstd = "0.13"`、`agent-client-protocol = "=3.3.0"`。`crates/zed/Cargo.toml` 版本号取 fork 侧 `1.21.0`（上游 main 已预升到 1.26.0）。
- **image_resolver**：恢复上游两参数 API `Box<dyn Fn(&str, &App) -> Option<ImageSource>>`。上一轮合并曾把签名改成单参数，导致远程图片解析（markdown、agent 面板）失效；本轮全仓 grep 调用点与测试一并改齐。
- **acp_thread**：整体取上游 `acp_v1`/`acp_v2` 拆分后的结构，fork 的「外部智能体自动压缩」重新接回 `update_usage` 末尾与轮次结束处；模块路径 `\bacp::` → `acp_v1::`，仅 4 处需 v2。
- **tabular_data_preview**：保留 fork 的表内联设置面板，面板挂载点从 `TabularDataPreviewPane` 迁到上游新实体 `TableView`。
- **设置项类型**：fork 把设置项标题/描述改为 `SharedString`（运行期本地化），上游是全静态 `&'static str`；所有取上游的新代码里裸字面量一律补 `.into()`。
- **上游覆盖 fork 旧实现的判定**：`markdown::apply_heading_style`→fork 的 `heading_text_style`、`invalidate_mermaid_cache(s)`→fork 的 `invalidate_embedded_diagrams_caches`、`tabular_data_preview/renderer/settings.rs` 的 popover 系列→fork 内联面板，均属改名或等价实现，不重复补回上游旧名。

## 编译失败链（本地 `cargo clippy --workspace --all-targets -- --deny warnings` 逐轮收敛）

1. **lib E0057**：`image_resolver` 调用点仍用单参数闭包（`markdown/src/html/html_rendering.rs`、`agent_ui/src/conversation_view.rs`、`agent_ui/src/inline_prompt_editor.rs`）。
2. **test E0593**：`markdown` 测试里 6 处单参数闭包。
3. **E0433**：`markdown_preview_view.rs` 缺 `project::{Project, ProjectPath, image_store}` 与 `rel_path::RelPath` 导入。
4. **settings_ui / git_ui 45 处**（`SharedString` 与上游全静态字面量的混合结果）：`page_data.rs` 补 18 处 `.into()`、`settings_ui.rs` 补 15 处、`git_graph.rs` 两处 `tooltip_label.clone()`、`settings_ui.rs` 改 `when_some(root_nav_label.clone(), ...)`；`git_panel.rs` 补 gpui 的 `ClipboardItem` 导入与上游 `fn repo_path`。
5. **E0308**：`t!` 宏带参数返回 `String`、不带参数返回 `Cow`（fork 的「t! 返回 Cow 消除翻译分配」提交），同一 `if`/`else` 两分支混用即类型不符（`agent_ui/conversation_view/thread_view.rs` 编辑不可用提示）。
6. **clippy::redundant_clone**：`t!(...).to_string()` 中 `.to_string()` 冗余（`agent_ui/conversation_view.rs` 的任务计数文案）。
7. **上游新增测试的键类型**：`settings_ui.rs` 的 `tab_in_last_setting_keeps_focused_control_visible` 用 `HashMap::<&'static str, _>`，而 fork 的 `add_basic_renderer` 形参是 `&str`；键类型改 `String` 并 `.entry(title.to_string())`。

## 历次合并遗留内容的检测与处理

- 上游有、HEAD 无、合并结果也没有的符号（`fn` 名维度）用脚本对本次改动的 546 个 rs 文件做三方比对（上游文件集合 − 合并结果文件集合）。命中 9 个文件，逐个判定真伪：改名/等价实现属假阳性。
- 本次顺带补回的历史丢失（编译必需）：`settings_ui` 的 `git_panel.starts_open`、`terminal.starts_open`、`git.diff_base` 三个设置项（上游有、fork HEAD 无；数组长度 `terminal_panel_section` 4→5、`git_hunks_section` 4→5、`[SettingsPageItem; 19]`）。
- 判定为**功能等价、结构不同**、不补回：`project_panel` 的删除确认（上游重构成 `build_removal_prompt` + `RemovalPrompt`，fork 保留等价的内联实现，含未保存缓冲区警告）。
- 已确认存在、本轮**未**修复的 fork 历史丢失（用户可见但非编译必需）：`agent_ui/conversation_view/thread_view.rs` 的 NTFS 沙箱写入警告（`ntfs_warning_enabled`、`configure-ntfs-warning` 设置项、`render_sandbox_windows_fs_warning`）、`collab_ui/collab_panel.rs` 的 `dock_side` 与 10 处 `.dock(self.dock_side(cx))`（列表项焦点/选中样式）、`project_panel_tests.rs` 的 4 个拖拽选择测试。

## 本地校验清单（与 fork 的 run_tests.yml 对齐）

```sh
CC=clang-22 CXX=clang++-22 cargo clippy --workspace --all-targets -- --deny warnings
cargo fmt --all -- --check
bash tmp/check-todos.sh    # script/check-todos 的 LF 副本（见下）
bash tmp/check-keymaps.sh  # script/check-keymaps 的 LF 副本
./tmp/typos/typos --config .config/typos.toml
```

1. **`script/*` 在 CRLF 工作区无法直接执行**：本机工作区是 CRLF，`#!/usr/bin/env bash\r` 会报 `"bash\r": 没有那个文件或目录`；CI 在 Linux 上检出为 LF 所以不受影响。本地用 `git show :script/check-todos > tmp/check-todos.sh` 取索引中的 LF 版本再 `bash` 运行。
2. **typos 无本地二进制**：CI 用 `crate-ci/typos` action（固定提交 `2d0ce569feab`，对应版本 1.40.0）。本地从 release 页下载 `typos-v1.40.0-x86_64-unknown-linux-musl` 解压到 `tmp/typos/` 后按 `--config .config/typos.toml` 运行。
3. **rustfmt 会改写行尾**：`cargo fmt --all` 重排过的文件被写成 LF，提交前需把本次编辑过的文件统一回 CRLF（工作区约定 CRLF、Git 存储 LF），否则 `git add` 报「LF 将被 CRLF 替换」并留下行尾混用。

## 合并后逐 hunk 丢失审计（2026-10-16）

汉化补齐过程中发现一处合并静默丢失：`crates/agent_ui/src/conversation_view/thread_view.rs` 的 `sandbox_confusable_findings`，上游 `953ac75529` 已改为同时扫描 `requested` 与 `canonical_or_requested()` 两条路径（`BTreeSet` 去重），合并结果却保留了 fork 旧版（只扫 `requested`），导致混淆字符出现在解析后真实写入目标时不报警。修复提交 `e26a09420f`。

审计方法（可复用）：`git diff --numstat 953ac75529 HEAD -- '*.rs'` 取改动行数较少的文件（本轮取 ≤20 与 21–60 两档，共 186 个），再逐 hunk 过滤掉含 `t!(`、`zed_i18n`、`locales/` 的汉化 hunk，人工判读剩余 hunk 是 fork 自有改动还是上游删除。fork 的 markdown 内联图表（`render_embedded_diagrams` 与上游 `render_mermaid_diagrams` 并存）、`is_native`、`icon_size`、`office_preview`/`sqlite_viewer` 等均为有意保留，非丢失。

## 涉及模块

- `crates/settings_ui`、`crates/git_ui`、`crates/markdown`、`crates/markdown_preview`、`crates/acp_thread`、`crates/agent_ui`、`crates/tabular_data_preview`、`crates/extensions_ui`、`crates/gpui_*`、`.github/workflows/`。

## 复发预防

1. 合并后必须跑全量 clippy（`--all-targets --deny warnings`）与 `cargo fmt --all -- --check`；只跑 `cargo check` 查不出 test 目标与 clippy lint（历史上多次由 `cargo test`/clippy 才暴露）。
2. 对上一步改动文件做「上游有 / HEAD 无 / 合并结果无」三方符号比对，识别 git 静默接受的 fork 侧删除；注意改名产生的假阳性。
3. 上游把字符串写死的位置，若 fork 同类位置是 `t!`，先看 locale 里键是否还在；本仓库约定为新上游文案补键并同步 `zh-CN`（本次新增键的 en/zh-CN 均已补齐，遗留 3 个缺失键为 fork HEAD 既有问题：`agent_ui.thread_view.copy`、`settings_ui.page_data.agent_prevent_idle_sleep_title`、`..._description`）。
4. 推送时本机 `gh` 为 2.4.0，无 `gh auth token` 子命令；token 从 `~/.config/gh/hosts.yml` 的 `oauth_token` 读取，push 走 HTTPS + `https_proxy` + 临时 `-c credential.helper`（不写入任何配置文件）。

## 关联页面

- [汉化 fork 同步 upstream 的合并模式与特性保留清单](../concepts/fork-upstream-sync.md)
- [汉化口径与批量替换工作流](../concepts/i18n-localization-workflow.md)
- [fork 发布 release 的流程与版本号策略](../concepts/fork-release-publishing.md)
- [run_tests Clippy 失败链](run-tests-clippy-failure-chain.md)
- [ACP 面板自动压缩上下文不生效](acp-auto-compact-external-agents.md)
