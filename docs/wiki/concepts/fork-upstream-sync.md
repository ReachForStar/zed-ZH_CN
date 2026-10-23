---
title: 汉化 fork 同步 upstream 的合并模式与特性保留清单
type: concept
tags: [fork, upstream, merge, i18n, workflow]
created: 2026-09-26
updated: 2026-10-16
status: active
---

# 汉化 fork 同步 upstream 的合并模式与特性保留清单

## 定义

本仓库是 zed-industries/zed 的简体中文汉化 fork（remote：`fork`=ReachForStar/zed-ZH_CN，`upstream`=zed-industries/zed）。周期性把 upstream/main 合并进本仓库 main，同时保留 fork 自有功能。

## 要点

- **fetch 走 HTTPS**：upstream/fork 的 remote 是 SSH（`git@github.com:`），本机 SSH :22 被墙；fetch/push 一律用 HTTPS（经 git-proxy，`http://127.0.0.1:7890`）：
  `git fetch https://github.com/zed-industries/zed.git +refs/heads/main:refs/remotes/upstream/main`。
- **判读 diff 的陷阱**：`git diff <fork>..upstream` 列出的文件≠upstream 新提交改的文件。fork 自有改动（如 fs_watcher 健康检查、vendor notify）也会出现在该 diff 里。判断“upstream 这次改了什么”应看 `git log fork..upstream` 的提交及其 `git show`，避免误以为上游动了 fork 特性文件。
- **典型冲突模式：上游 API 重构 × 本仓库 t! 本地化**。上游重构改调用形（如 `model.model.name()` → `model.name()`）并把字符串写死；fork 同位置是 `t!("...")`。解法=取上游新 API + 保留 `t!` 本地化键（键通常仍在 locale 中）。见 2026-09 的 language_model 重构冲突（agent_model_selector.rs、inline_prompt_editor.rs）。
- **上游移动根目录配置 × fork 精简 workflow**：上游 #64303 把 `typos.toml`/`lychee.toml`/`livekit.yaml` 移到 `.config/`、`renovate.json` 移到 `.github/`，并同步改生成版 run_tests.yml；本仓库手工维护的 `run_tests.yml` 不会跟着改，typos 步骤仍指 `./typos.toml` 导致 CI 直接失败（"could not read config"）。合并后跑 `cargo xtask` 之外的自有 workflow 需人工核对配置路径引用。
- **版本号行**：上游切 release 分支时会给 main 预升版本号（如发布 1.21 后 main 直接写 1.22→1.23）；fork 发布取上游已发布版本号、自改 `crates/zed/Cargo.toml` 与 `Cargo.lock` 后，该两行与上游长期分歧，每次合并必冲突、取 fork 侧。见 [fork 发布 release 的流程与版本号策略](fork-release-publishing.md)。
- **静默功能丢失模式：冲突取 ours 吞掉上游新增调用点**。UI 大文件（如 `extensions_ui.rs`）因 `t!` 与上游重构冲突，解决时整文件保留 fork 旧版，结果上游新增的函数与 `extension_card.rs` 的 builder 方法（context_menu、repository_icon 等）整体丢失；丢的是“谁调用新模块”一侧，新模块文件本身还在，`mod` 声明保留 → 编译通过但 clippy `-D warnings` 报 dead_code 才暴露（v1.21.0 发布时 `ExtensionVersionSelector` 即此）。合并后对上游改动过的 UI crate 跑 `git diff <upstream-parent> HEAD -- <crate>` 核对引用数，防止整代际落后。
- **vendor 依赖保留**：fork 把 notify vendor 到 `third_party/notify`（含 Windows 溢出修复），workspace Cargo.toml 用 `path=`；upstream 用 git rev。合并时保留 fork 的 path vendor 版本。
- **fork 给共享 trait 加方法 × 上游新增 test 后端**：fork 给某 trait（如 `fs_watcher::WatchBackend` 的 `watched_paths`）加了方法后，上游新增的实现该 trait 的测试后端（`#[cfg(test)]` / `feature="test-support"`）不会自动带上该方法，导致 **`cargo test` 编译失败但 `cargo check` 通过**（test-only 代码不参与普通 check）。合并后必跑 `cargo test -p <crate> --no-run` 或 `cargo check -p <crate> --features test-support` 才暴露。见 2026-09 的 fs `watched_paths` 遗留（fs.rs `FakeWatchBackend`、fs_watcher/diagnostics.rs `Backend`）。
- **类型分歧：运行期本地化的 `SharedString` × 上游全静态字面量**。fork 把设置 UI 的标题/描述（`SettingItem.title/description`、`add_basic_renderer` 形参）与部分设置项类型改成运行期本地化的 `SharedString`；上游仍是 `&'static str`。取上游新代码后，裸字面量必须补 `.into()`，上游新增测试里 `HashMap::<&'static str, _>` 之类的键要改 `String`，否则报 E0308 / E0521（见 [2026-10 合并记录](../queries/upstream-merge-2026-10.md)）。
- **`t!` 返回类型分歧**：带参数返回 `String`、不带参数返回 `Cow<'_, str>`（fork 的「t! 返回 Cow 消除翻译分配」提交）。同一 `if`/`else` 或 `match` 分支混用两者即 E0308，需显式统一类型；反过来给 `String` 再加 `.to_string()` 会触发 `clippy::redundant_clone`（`-D warnings` 下编译失败）。
- **本地校验要过行尾关**：`cargo fmt --all` 重排过的文件被 rustfmt 写成 LF，提交前须把本次编辑过的文件还原 CRLF（工作区约定 CRLF、Git 存储 LF）；CRLF 工作区里 `script/check-todos`、`script/check-keymaps` 直接执行会因 `#!/usr/bin/env bash\r` 失败，用 `git show :script/<名> > tmp/<名>.sh` 取索引中的 LF 版本再跑。
- **上游重构内部状态模型 × fork 特性直读旧字段**：上游把 watcher 的 `PathRegistrationState.has_os_watcher`（布尔）换成 `coverage: Coverage::Watched { covered } | CoveredBy(ancestor)`，并从 `WatcherRegistrationState` 去掉 `path` 字段。fork 的 `check_watch_health` 直接读这些字段 → E0609。解法=按新模型重写判定：只有 `Coverage::Watched` 的注册才可能有自己的后端 watch 被静默撤销；路径改从 `path_state.path` 取。取上游新结构后要重新核对所有「读内部字段」的 fork 特性。
- **上游把文案迁到新模块 × 汉化点跟着迁**：上游把子智能体卡片的标题与状态文案从 `agent_ui/conversation_view/thread_view.rs` 迁到新文件 `agent_ui/conversation_view/subagent_card.rs`（`SubagentCardState::title`）。汉化键要跟着迁到新文件，否则新界面回落英文，且旧文件里的键会变成死键。
- **上游事件枚举改签名 × fork 钩子跟着改**：`AcpThreadEvent::TokenUsageUpdated` 变为携带 `OutputTokensUpdate`（`Unchanged` / `LegacyAdd`）。fork 的「外部智能体自动压缩」钩子挂在 `update_usage` 末尾，需改成 `TokenUsageUpdated(OutputTokensUpdate::Unchanged)` 后再调用钩子。
- **上游重排确认框文案 × 既有汉化键的占位符要跟着改**：上游把 git 面板「丢弃/移入回收站」确认框按 tracked/untracked 分流，并区分 `changes` / `unstaged changes`。fork 原本的 `discard_changes_confirm` 是固定文案，需改为带 `%{changes}` 插值，并新增 `discard_changes_to_count_files`、`trash_untracked_count_files` 等键；en 与 zh-CN 两侧模板同步改。
- **上游 lock 文件删条目 ≠ 本地要保留**：合并 `Cargo.lock` 冲突时把两侧条目取并集，交给 `cargo metadata` 重新裁剪（会删掉已无引用的 `cocoa-foundation 0.2.0` 之类条目），不要手工判断条目去留。
- **上游 git 依赖 rev 前进**：上游把 `lsp-types` rev 从 `f1783e63a7f4…` 前进到 `82d7fc66ebdf…`（dynamic registrations 需要新 API），fork 必须跟进新 rev，不能保留旧 rev，否则编译期缺 API。
- **fork 早期手工 backport × 上游后来正式实现 → 合并时静默丢失**：fork 曾手工 backport `markdown` 的 `pending_heading_scroll` 机制（提交 `fb319c54010`），上游后来正式实现并加入 `defer_parsing`。两套实现并存时 merge 把上游在 `reset()`/`parse()` 里的清理逻辑判成冲突并丢弃，fork HEAD 因此长期缺上游的 pending 清理。本轮按上游恢复（`reset()` 重新应用 pending 滚动、`parse()` 空源时清 pending）。
- **合并两侧新增 `use` 要按字母序**：rustfmt 默认开启 `reorder_imports`，两侧各加一个 `use` 时直接并排会触发 `cargo fmt --check` 失败；按字母序放置（如 `use ui_input::InputField;` → `use util::ResultExt;` → `use zed_i18n::t;`）。

## 特性保留清单（每次合并后逐项核对）

- `crates/zed_i18n/locales/zh-CN/*` 汉化文件齐全。
- `crates/tabular_data_preview/*`（表格/SQLite 预览）；表内联设置面板的挂载点是上游 `TableView`，勿回落到 `TabularDataPreviewPane`。
- `crates/settings_ui/*` 的设置项标题/描述为 `SharedString`（运行期本地化）；取上游新代码后补 `.into()`，上游新增设置项要与 `[SettingsPageItem; N]` 数组长度、`terminal_panel_section`/`git_hunks_section` 等分节数组同步。
- `crates/fs/src/fs_watcher.rs` 的 `check_watch_health`（watch 健康检查）——含 `WatchBackend::watched_paths`；上游新增的 test 后端需补实现（见要点）；上游重构 `PathRegistrationState` 后要按新 `coverage` 模型重写孤儿判定；健康检查线程用 `Weak<OsWatcher>` 持有 watcher，否则会把 `OsWatcher` 及其 `BackgroundExecutor` 永久钉住（上游 `watcher_retains_scheduler_until_teardown` 用例会失败）。
- `.github/workflows/` 仅 `release.yml` + `run_tests.yml`（精简 CI，禁止 `cargo xtask workflows` 覆盖）；上游删除 workflow 文件与 fork 的删除产生 delete/modify 冲突时取删除。
- `README.md` 顶部 `> [!IMPORTANT]` 与 `> Remove this line ...` 两行完好（.rules 硬规则，勿删）。

## 关联页面

- [2026-10 合并上游 main：冲突消解决策与编译失败链](../queries/upstream-merge-2026-10.md) — 本轮合并的决策、clippy 失败链、历史丢失去向与本地校验清单。
- [汉化口径与批量替换工作流](i18n-localization-workflow.md) — 合并后补齐汉化的口径、批量脚本与 `t!` 类型适配。
- [msvc_spectre_libs 构建失败](../queries/msvc-spectre-libs-build-failure.md) — 本地编译 repl/zed 的环境前置。
- [ACP 面板自动压缩上下文不生效](../queries/acp-auto-compact-external-agents.md) — fork 在 acp_thread 层的自有特性；同批修复 fs `watched_paths` merge 遗留。
