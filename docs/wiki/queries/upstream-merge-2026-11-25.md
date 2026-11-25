---
title: 2026-11-25 合并 upstream/main（1.31.0 线，68 提交）：冲突消解、上游旧实现回填与本地化测试环境
type: query
tags: [upstream-merge, conflict, i18n, fork]
created: 2026-11-25
updated: 2026-11-25
status: active
---

# 2026-11-25 合并 upstream/main

## 范围

- 上游基线：`upstream/main` 的 `cd585371e8`，距上次合并（`f126950f9d`）新增 68 个提交、282 个文件（+23088 / −11266）。
- 版本号：上游 `crates/zed/Cargo.toml` 仍是 `1.31.0`，与 fork 相同 → 本轮版本号行没有冲突（此前每轮必冲突）。按用户要求不发布新版本：未改版本号、未打 tag，`v1.31.0` 标签仍指 `199b8cfa64`。
- 合并落点：本仓库 `main`，`git merge --no-edit upstream/main`。
- 合并提交：`314485e923`（父提交 `2093c0ea1e` + `cd585371e8`），已推送 fork `main`（`git push https://github.com/ReachForStar/zed-ZH_CN.git main:main`）。
- 上游要点：ACP SDK 升 5.5.1（schema 1.21.2）、Rust 升 1.100、gpui 移除实体借用的 `.unwrap()` 并保留跨帧布局、Windows 原生 AppContainer 沙箱、bedrock 运行期模型发现、worktree picker 显示已关闭的 checkout、对话级 model/profile/effort 选择。

## 冲突与判定

13 个未合并路径：

| 文件 | 判定 |
| --- | --- |
| `.github/workflows/{after_release,autofix_pr,release_nightly}.yml` | fork 已删除、上游修改 → 取删除，仍只保留 `release.yml` + `run_tests.yml` |
| `.github/workflows/run_tests.yml` | 取 fork（新增 `run_sandbox_tests_windows_arm` job、`cargo test -p gpui --lib --no-run` 步骤、`cargo-shear@1.14.1` 均不引入） |
| `Cargo.lock` | 取上游侧 + `cargo metadata` 重裁；fork 的 `pdf-render = "=1.3.0"` 许可证锁与 `notify` path vendor 条目由 cargo 自动补回 |
| `script/bundle-windows.ps1` | 上游 #67404 删除 AGS SDK 下载（`DownloadAMDGpuServices`、`CollectFiles` 的 `amd_ags_x64.dll` 搬移、调用行）→ 随上游删除；保留 fork 的 `Invoke-DownloadWithRetry` 重试 helper（`DownloadConpty` 仍在用） |
| `crates/acp_thread/src/acp_thread.rs` | 两侧各加字段（fork `auto_compact_armed` × 上游 `usage_meta`）→ 两侧都保留 |
| `crates/agent_ui/src/conversation_view/subagent_card.rs` | 上游给 `SubagentStatus` 加 `Idle` 变体 → 取上游分支结构 + fork 的 `t!` 键（漏 `Idle` 会 E0004 分支不全） |
| `crates/agent_ui/src/conversation_view/thread_view.rs` | 7 处：`submission_text_parts` 取上游带 `separator` 的新签名并重写为返回 `Cow<'a, str>`；`Disconnected { receipt: None }` 保留 fork 键、改 locale 文案；`SandboxNotAppliedReason::ErrorLinuxWsl` → 上游改名 `CreationFailed`；Windows 驱动器写入警告（`render_sandbox_windows_fs_warning` / `ntfs_warning_enabled`）随上游删除；`render_sandbox_authorization_path_row` 取上游整函数 |
| `crates/agent_ui/src/threads_archive_view.rs` | 上游改用 `thread.display_title()` → 取新调用 + fork 的 `t!` 插值键 |
| `crates/ai_onboarding/src/plan_definitions.rs` | 上游改学生套餐文案 → 保留 `t!("…tokens_10")`，改 en/zh-CN 的值 |
| `crates/git_ui/src/git_panel.rs` | `gpui` 导入两侧合并（fork `PromptButton` × 上游 `ReadGlobal`）；`GenerateCommitMessageConfigurationTooltip` 取上游新增的 `match self` 结构并汉化两个分支 |
| `crates/git_ui_core/src/worktree_picker.rs` | 仅 `gpui` 导入两侧合并（fork `PromptButton` × 上游 `AsyncWindowContext`）；上游 1401 行新增（closed checkouts）全部并入 |

## 上游旧实现回填（本轮最重要的发现）

fork HEAD 的 `render_sandbox_authorization_path_row` 是上游 #58358（`Improve sandbox write-path authorization UI`）之前的旧版（参数 `path: &Path, show_border: bool`），而合并基线 `f126950f9d` 已是 `granted: &settings::GrantedWritePath` 新版。这是早前某轮合并「冲突取 ours」吞掉上游重写的结果，本轮才以冲突形式暴露：git 只把真正不同的头部行标为冲突，签名与函数体分别来自两侧，拼出的代码不成立。

处理：整函数按上游重写（含 Source / Target / Write Path 三态），调用点也改回上游形式（`render_sandbox_authorization_path_row(entry_ix, path_ix, path, cx)`，原先 fork 传 `&path.requested` 与 `path_ix < paths.len() - 1`），再补 `t!`。

检测方法（本轮新增，替代人工比对）：对上游本轮改过的每个 `.rs`，取 `git diff -U0 f126950f9d..upstream/main` 的新增行集合，逐行核对是否出现在合并结果里。本轮 282 个文件中 8 个报缺失，逐条确认全部是「被 `t!` 替换」或「导入两侧合并」，无真实丢失。脚本一次性使用，未入库。

## 汉化

新增 9 个键（en / zh-CN 同步）：

- `agent_ui.thread_view.sandbox_source_path`、`sandbox_target_path`（上游写入路径行的 Source / Target 标题；非重定向情形复用已有 `requested_write_path`）
- `agent_ui.thread_view.thread_only_mode`、`thread_only_mode_note`、`reset_to_defaults`、`save_as_default`（上游对话级 model/profile/effort 选择）
- `agent_ui.thread_view.subagent_idle`（上游 `SubagentStatus::Idle` 图标提示）
- `agent_ui.conversation_view.unconfirmed_saved_messages`（上游恢复未确认提交提示）
- `git_ui.git_panel.saved_model_unavailable_hint`（上游 tooltip 新增分支）

改值不改键：`ai_onboarding.plan_definitions.tokens_10`（`$10 of tokens in Zed agent` → `$10/month in GPT Luna credits`）、`agent_ui.thread_view.submission_detail_connection_lost_unknown`（上游重写为「智能体可能已处理过该消息」口径）。

删除死键 4 个：`agent_ui.thread_view.{windows_drive_write_warning,windows_drive_isolation_note,configure_windows_drive_warning,dont_show_this_warning_again}`（上游 #66918 用 Windows AppContainer 取代 WSL DrvFs 警告，连 `AgentSettings.sandbox_permissions.warn_ntfs_grants` 设置项一并移除；fork 曾在 `9684f5903a` 特意补回该警告，本轮随上游删除）。

`scripts/i18n/check-missing-keys.py`：代码静态引用 4003 键、en / zh-CN 各 4065 键、0 问题。上游本轮改动的 UI 文件按「上游新增行」核对，未留未汉化文案（`Create new worktree based on …`、`any host`、`unrestricted except Git metadata` 属历史欠账，base 已存在，本轮不动）。

## 编译与测试失败链

1. `acp_thread` E0308：ACP SDK 5.5.1 把 `CompactionUpdate.status` 改成 `MaybeUndefined<CompactionStatus>` → fork 的 `upsert_context_compaction_update_inner` 完成判定改为 `matches!(update.status, MaybeUndefined::Value(acp_v2::CompactionStatus::Completed))`。
2. `agent_ui` 语法错误 `unexpected closing delimiter`：按行号整函数替换时区间末位少算一行，留下重复的 `}`；`rustfmt --edition 2024 --check <file>` 直接给出行列号，比 `cargo check` 更易定位。
3. `agent_ui` E0515 `cannot return value referencing function parameter`：`submission_text_parts` 返回 `Cow<'a, str>` 后 `.flat_map(|part| part.chars())` 借用局部值 → 改 `.collect::<String>().chars()`（std 有 `FromIterator<Cow<'_, str>> for String`）。
4. `webrtc-sys` 两段环境坑：C++ 头 `virtual const Network* Network() const` 在 GCC 11 下报 `changes meaning` → `CXXFLAGS=-fpermissive`；预编译包下载中断后，`download_webrtc()` 每次先 `remove_dir_all(OUT_DIR/webrtc-<tag>/<triple>)` 再重下，留下空目录 → 用 `curl -C - -x <proxy>` 断点续传 zip，解压到 `target/webrtc-sdk/`，以 `LK_CUSTOM_WEBRTC=<abs>/target/webrtc-sdk/linux-x64-release` 跳过下载（zip 根目录即 `linux-x64-release`）。
5. `-p git` 的 `test_blame_untracked_path_with_large_contents` 失败：`crates/git/src/blame.rs` 用 `GIT_BLAME_NO_PATH = "fatal: no such path"` 匹配 stderr，本机 `LANG=zh_CN.UTF-8` 使 git 输出「致命错误：no such path …」→ 判定为环境所致（该测试来自上游 `58be7923cf`，base 与 fork HEAD 均有，`blame.rs` 与上游完全一致），用 `LC_ALL=C LANG=C` 复跑 85/85 通过。合并与回归测试批次统一加 `LC_ALL=C LANG=C`。

## 验证

- `cargo check --workspace --all-targets`：通过。
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`：通过。
- `cargo fmt --all --check`：通过（提交前 `python3 scripts/normalize-crlf.py` 还原工作区 CRLF）。
- `script/check-todos`、`script/check-keymaps`：通过（CRLF 工作区用 `git show :script/<名> > tmp/<名>.sh` 取 LF 版本执行）；`typos` 本机未安装，仅由 fork CI 覆盖。
- 测试 38 个套件 / 7030 项全通过：
  - 批次一（fork 关键）：acp_thread 321、agent_servers 202（1 ignored）、agent_ui 611、fs 35 + fs 集成 29（1 ignored）、git_ui 223、git_ui_core 51、markdown 207、project_panel 150、settings_ui 72。
  - 批次二：agent 819、agent_settings 44、editor 1236、git 85、gpui 592 + 文档测试 89、language_model_selector 8、language_models 157、office_preview 14、settings_content 52 + 文档测试 2、sidebar 171、sum_tree 11、tabular_data_preview 23、terminal_view 101、workspace 310、zed_i18n 2。
  - 批次三：agent_skills 95、project 114 + 集成 444、prompt_store 26、vim 586、zed 148。
  - 未跑：`collab`（需 Postgres）、`sandbox*`（需特权与 Windows 环境），仅由 `--all-targets` 编译覆盖。

## fork 特性核对

- `zed_i18n` locale 分片 en / zh-CN 各 70 个文件，一一对应，无缺侧。
- `acp_thread`：`resolve_location` 目录跳过（`project.fs().metadata()`）仍在；`drop_compacted_entries` 内存回收与 `maybe_trigger_auto_compact` 钩子仍在（钩子调用点 5096 / 6496 未受上游 `TokenUsageUpdated` 变化影响）。
- `crates/fs`：`check_watch_health` 与 `WatchBackend::watched_paths`（含 `fs.rs`、`fs_watcher.rs` 两个测试后端实现）仍在。
- `tabular_data_preview` 23 项测试通过，表内设置面板仍挂 `TableView`；`office_preview` 通过，`pdf-render` 锁 1.3.0 未被上游 lock 覆盖。
- `settings_ui`：`SettingItem.title/description` 仍为 `SharedString`。
- `.github/workflows` 仍只有 `release.yml` + `run_tests.yml`；未运行 `cargo xtask workflows`。
- `third_party/notify` path vendor 保留（`Cargo.lock` 中 `notify` 无 `source` 字段即为证）。
- `README.md` 顶部 `> [!IMPORTANT]` 与 `> Remove this line …` 完好。
- `crates/zed/resources/windows/zed.iss` 与上游只差 fork 的 `RunOnceId: "RemoveZedAppxPackage"`。
- 含 `t!(` 调用的 crate 全部有 `zed_i18n` 依赖（按 crate 目录扫描，0 缺依赖）。

## 复发预防

- 上游删功能时，fork 早年为「补回合并丢失」而恢复的代码会在下一轮以冲突重现（本次的 WSL DrvFs 警告，`9684f5903a` 补回、本轮上游 #66918 正式移除）。判定顺序：先查该代码在上游的当前状态（`git grep <符号> upstream/main`）与删除它的提交，再决定取删除，随后清 locale 死键。
- 冲突块只覆盖函数头部时，函数签名与函数体可能分别来自两侧；此类判定应「整函数取上游 + 重新汉化」，不要按冲突块裁剪。
- 上游把可选字段包成 `MaybeUndefined`（ACP SDK 每次对齐 schema 都会发生），fork 直读该字段的钩子在 `cargo check` 期即报 E0308，改法是用 `MaybeUndefined::Value(..)` 包模式。
- 汉化后返回类型从 `&'a str` 变 `Cow<'a, str>` 时，链式 `.flat_map(str::chars)` 之类借用式适配器要改成先 `collect::<String>()` 再处理。
- 本机跑测试固定加 `LC_ALL=C LANG=C`：`git` 子进程输出被中文 locale 影响，`crates/git` 的 stderr 文案匹配与若干外部命令断言都会假失败。
- 合并后必跑「上游新增行 ⊆ 合并结果」集合核对，比逐文件人工读 diff 可靠。

## 关联页面

- [汉化 fork 同步 upstream 的合并模式与特性保留清单](../concepts/fork-upstream-sync.md) — 本轮新增的要点（整函数回填、`MaybeUndefined`、`LC_ALL=C`、webrtc 预编译包）已并入该清单，合并后按清单逐项核对。
- [2026-11-23 合并 upstream/main（1.31.0，135 提交）](upstream-merge-2026-11-23.md) — 上一次合并基线 `f126950f9d`；该轮的 `project_panel` 回归修复本轮未复现问题，本轮的旧实现回填同样源自更早轮次的「冲突取 ours」。
- [fork 发布 release 的流程与版本号策略](../concepts/fork-release-publishing.md) — 本轮按要求停在 1.31.0、不打标签；下轮发布前需按该页把版本号与 `RELEASE_NOTES` 同步。
- [ACP 面板自动压缩上下文不生效](acp-auto-compact-external-agents.md) — fork 的 `auto_compact_armed` 与 `drop_compacted_entries` 特性，本轮因 ACP SDK `MaybeUndefined` 调整了完成判定。
