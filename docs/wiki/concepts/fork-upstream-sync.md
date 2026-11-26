---
title: 汉化 fork 同步 upstream 的合并模式与特性保留清单
type: concept
tags: [fork, upstream, merge, i18n, workflow]
created: 2026-09-26
updated: 2026-11-25
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
- **本地校验要过行尾关**：`cargo fmt --all` 重排过的文件被 rustfmt 写成 LF，提交前用 `python3 scripts/normalize-crlf.py` 还原成 CRLF（工作区约定 CRLF、Git 存储 LF）；无参数处理 `git diff HEAD` 里的 `.rs`，给路径则只处理指定文件（可重复执行）。CRLF 工作区里 `script/check-todos`、`script/check-keymaps` 直接执行会因 `#!/usr/bin/env bash\r` 失败，用 `git show :script/<名> > tmp/<名>.sh` 取索引中的 LF 版本再跑。
- **上游重构内部状态模型 × fork 特性直读旧字段**：上游把 watcher 的 `PathRegistrationState.has_os_watcher`（布尔）换成 `coverage: Coverage::Watched { covered } | CoveredBy(ancestor)`，并从 `WatcherRegistrationState` 去掉 `path` 字段。fork 的 `check_watch_health` 直接读这些字段 → E0609。解法=按新模型重写判定：只有 `Coverage::Watched` 的注册才可能有自己的后端 watch 被静默撤销；路径改从 `path_state.path` 取。取上游新结构后要重新核对所有「读内部字段」的 fork 特性。
- **上游把文案迁到新模块 × 汉化点跟着迁**：上游把子智能体卡片的标题与状态文案从 `agent_ui/conversation_view/thread_view.rs` 迁到新文件 `agent_ui/conversation_view/subagent_card.rs`（`SubagentCardState::title`）。汉化键要跟着迁到新文件，否则新界面回落英文，且旧文件里的键会变成死键。
- **上游事件枚举改签名 × fork 钩子跟着改**：`AcpThreadEvent::TokenUsageUpdated` 变为携带 `OutputTokensUpdate`（`Unchanged` / `LegacyAdd`）。fork 的「外部智能体自动压缩」钩子挂在 `update_usage` 末尾，需改成 `TokenUsageUpdated(OutputTokensUpdate::Unchanged)` 后再调用钩子。
- **上游重排确认框文案 × 既有汉化键的占位符要跟着改**：上游把 git 面板「丢弃/移入回收站」确认框按 tracked/untracked 分流，并区分 `changes` / `unstaged changes`。fork 原本的 `discard_changes_confirm` 是固定文案，需改为带 `%{changes}` 插值，并新增 `discard_changes_to_count_files`、`trash_untracked_count_files` 等键；en 与 zh-CN 两侧模板同步改。
- **上游 lock 文件删条目 ≠ 本地要保留**：合并 `Cargo.lock` 冲突时把两侧条目取并集，交给 `cargo metadata` 重新裁剪（会删掉已无引用的 `cocoa-foundation 0.2.0` 之类条目），不要手工判断条目去留。
- **上游 git 依赖 rev 前进**：上游把 `lsp-types` rev 从 `f1783e63a7f4…` 前进到 `82d7fc66ebdf…`（dynamic registrations 需要新 API），fork 必须跟进新 rev，不能保留旧 rev，否则编译期缺 API。
- **fork 早期手工 backport × 上游后来正式实现 → 合并时静默丢失**：fork 曾手工 backport `markdown` 的 `pending_heading_scroll` 机制（提交 `fb319c54010`），上游后来正式实现并加入 `defer_parsing`。两套实现并存时 merge 把上游在 `reset()`/`parse()` 里的清理逻辑判成冲突并丢弃，fork HEAD 因此长期缺上游的 pending 清理。本轮按上游恢复（`reset()` 重新应用 pending 滚动、`parse()` 空源时清 pending）。
- **合并两侧新增 `use` 要按字母序**：rustfmt 默认开启 `reorder_imports`，两侧各加一个 `use` 时直接并排会触发 `cargo fmt --check` 失败；按字母序放置（如 `use ui_input::InputField;` → `use util::ResultExt;` → `use zed_i18n::t;`）。
- **上游删模块 × fork 依赖点**：`http_client` 删除 `github`/`github_download`（统一资产下载）后，fork 的 `auto_update` 自建 GitHub Releases 通道必须改用 `github_releases` crate（`latest_github_release`、`get_release_by_tag_name`、`GithubReleaseAsset` 同名同签名），并补 `Cargo.toml` 依赖；用 `github_releases::` 全路径，别用别名导入。
- **上游把 struct 字段改成访问器**：ACP SDK 3.6 把 `AvailableCommand.name` 变成 `name()`，直接读字段报 `E0615`；fork 里凡是读 ACP schema 字段的钩子（如自动压缩找 `/compact` 命令）升级依赖后要跟着改。
- **上游整块删除功能 × fork 的汉化键**：agent checkpoints 被上游整体移除（`eee7501970`），冲突里 Restore Checkpoint 按钮取删除即可；随后要清掉只剩 locale 里引用的死键（`agent_ui.restore_checkpoint*`），否则永久残留。
- **冲突块跨越多个函数时逐个比对符号**：`subagent_card.rs` 的冲突块里夹着 `fullscreen_entry`，按冲突标记裁剪会把它一起删掉，只有编译报 `no method named ...` 才暴露。合并后对冲突文件做「上游 / fork HEAD / 合并结果」的函数名集合比对。
- **自己写解析块时容易多带一个闭合分隔符**：`title_bar.rs`、`thread_view.rs` 各出现重复 `};`，报 `unexpected closing delimiter`；`rustfmt --edition 2024 --check <file>` 会直接给出列号，比 `cargo check` 的报错更好定位。
- **本机 GCC 11 编 webrtc-sys 需 `-fpermissive`**：新 livekit rev 强制重编 C++ 源码，`virtual const Network* Network() const` 报 `changes meaning` 错误；用 `CXXFLAGS=-fpermissive` 构建（不要改仓库文件）。
- **测试链接缺 `libX11-xcb.so`**：系统只装了 `libX11-xcb.so.1`，`cargo test` 链接 `scap` 时 `-lX11-xcb` 失败；用 `tmp/libshim/libX11-xcb.so → /usr/lib/x86_64-linux-gnu/libX11-xcb.so.1` 加 `LIBRARY_PATH` 即可，不需要改动仓库或安装系统包。
- **上游新增测试与 fork 特性语义分歧 → 合并后测试失败**：二次合并带入的上游 `agent_servers` 测试（`test_load_session_replays_notifications_sent_before_response`、`v2_compaction_sdk_chunks_snapshots_and_historical_targeting`）期望上下文压缩完成后旧条目仍在，而 fork 的 `drop_compacted_entries`（内存回收特性）在 `Completed` 时丢弃被压缩区间，两者直接冲突。判定方法：回退到合并前 HEAD 复现同样失败 → 非本次合并引入；再 `git merge-base --is-ancestor <测试提交> <上次合并结果>` 定位带入轮次。此类失败不阻塞合并，但每次合并后跑 `cargo test -p agent_servers` 可见。
- **上游把函数下沉到新 crate 模块 × fork 侧旧定义**：`open_output` / `show_error_toast` 从 `git_ui::git_panel` 移到 `git_ui_core::notifications`。自动合并会同时留下 fork 的旧函数与上游的 `pub(crate) use ...`，报 `E0255 名称重复定义`；解法是删掉旧函数与之配套的测试，只保留 `pub(crate) use` 重导出。
- **上游换掉构造函数但 fork 汉化版仍在用**：连接通话挂断确认，上游改成 `window.prompt(PromptLevel::Warning, "...", None, &["Close window and hang up"], cx)`，fork 汉化版仍是 `PromptButton::new(t!(...))`。冲突解决取 fork 文案时，`gpui::{...}` 导入里必须仍有 `PromptButton`，否则 `E0433`。
- **tooltip / 回调是 `Fn` 不是 `FnOnce`**：把上游写死的 `&'static str` 换成 `t!` 后，`Cow`/`SharedString` 被 `move` 进回调时只有第一次调用能取走，`Disclosure::tooltip`、`PopoverMenu` 等多次调用的闭包里要 `label.clone()`。
- **上游新增 crate × fork 的 `zed_i18n`**：上游把模块拆成新 crate（如 `language_model_selector`）时，fork 的 `t!` 调用会跟着搬过去，但新 crate 的 `Cargo.toml` 没有 `zed_i18n` 依赖 → `E0432`。合并后对新 crate 逐个补 `zed_i18n.workspace = true`。
- **上游删 `.rules` 并入 `AGENTS.md`**：`git merge` 报 `.rules`/`AGENTS.md` 的 delete/modify 冲突（并生成 `AGENTS.md~<commit>` 残留文件）。取 fork 的两个文件，再删除残留文件；漏删会被误提交。
- **上游把「内联 UI 逻辑」收敛为共享函数 × fork 旧内联副本**：确认框由 `ProjectPanel::build_removal_prompt(RemovalKind, names, dirty_buffers)` 统一后，fork 早前为编译写的内联版本不会冲突、会静默留下，结果是「行为可用但上游测试断言失败」（未保存警告落在 detail 而非 message）。用 `git diff <upstream> -- <file>` 找上游已删除的旧实现并恢复共享函数，不要改测试。
- **上游改访问器语义 × fork 的回归测试**：`AgentThreadEntry::location(ix)` 从「未解析返回 None」改成「未解析也返回 `Some((原始位置, None))`」。fork 的目录跳过测试要改为断言元组第二项（`location(ix).and_then(|(_, resolved)| resolved)`），fork 逻辑本身不受影响。
- **冲突块只覆盖函数头部 → 签名与函数体分属两侧**：`render_sandbox_authorization_path_row` 的基线已是上游新版（`granted: &settings::GrantedWritePath`），fork HEAD 是上游 #58358 之前的旧版（`path: &Path, show_border: bool`），git 只把真正不同的头部行标冲突，拼出的代码不成立（`thread_view.rs` 语法错误 `unexpected closing delimiter` 也来自此处）。解法=整函数取上游 + 重新汉化，并把调用点一并改回上游形式；按行号整段替换前先核对被替换区间的末行属于该函数，否则会多留一个 `}`。
- **上游新增行是否全部进入合并结果，用集合核对**：对上游本轮改过的每个 `.rs`，取 `git diff -U0 <上次基线>..<upstream>` 的新增行（去空白、忽略 <12 字符），逐行查是否在合并结果里。报缺失的只应是「被 `t!` 替换」与「导入两侧合并」两类。该方法比人工读 diff 可靠，比「计数对比」更精确（见下），已入库为 `scripts/check-upstream-added.py`：`python3 scripts/check-upstream-added.py <上次合并的上游基线> <上游 ref>`。
- **上游删功能 × fork 早年补回的代码**：fork 曾为编译丢失补回 WSL DrvFs 写入警告（`9684f5903a`），上游 #66918（Windows 原生 AppContainer 沙箱）正式移除该功能与 `AgentSettings.sandbox_permissions.warn_ntfs_grants`。判定顺序：`git grep <符号> upstream/main` 确认上游现状 → 找到删除它的提交 → 取删除 → 清 locale 死键（本轮清 4 个）。
- **ACP SDK 每次对齐 schema 都会把可选字段包成 `MaybeUndefined`**：5.5.1 / schema 1.21.2 把 `CompactionUpdate.status` 变成 `MaybeUndefined<CompactionStatus>`，fork 直读该字段的判定要写成 `MaybeUndefined::Value(acp_v2::CompactionStatus::Completed)` 模式，`cargo check` 期即报 E0308。
- **汉化把 `&'a str` 改成 `Cow<'a, str>` 后，借用式迭代器适配器失效**：上游 `submission_text_parts(&content, separator) -> impl Iterator<Item = &'a str>` 的 `.flat_map(str::chars)` 在汉化版下报 E0515（引用闭包参数局部值）；改成 `.collect::<String>().chars()`（std 有 `FromIterator<Cow<'_, str>> for String`），复制路径的 `.collect::<String>()` 保持可用。
- **改文案不改键**：上游只换字符串值（如学生套餐 `$10/month in GPT Luna credits`、`Disconnected { receipt: None }` 的 detail）时，源码 `t!` 调用不动，改 en/zh-CN 两侧的值即可；新增枚举分支（`GenerateCommitMessageConfigurationTooltip::SavedModelUnavailable`）才需要新键。
- **上游给枚举加变体 × fork 汉化的 match**：`SubagentStatus::Idle` 加入后，fork 侧汉化的分支必须补该变体，否则 E0004 分支不全。
- **本机跑测试固定 `LC_ALL=C LANG=C`**：`crates/git/src/blame.rs` 以 `"fatal: no such path"` 匹配 git stderr，`LANG=zh_CN.UTF-8` 下 git 输出「致命错误：…」，`test_blame_untracked_path_with_large_contents` 假失败（该用例来自上游 `58be7923cf`，`blame.rs` 与上游一致时属环境问题，不是回归）。
- **webrtc-sys 预编译包下载不可续传**：`download_webrtc()` 每次先 `remove_dir_all(OUT_DIR/webrtc-<tag>/<triple>)` 再整包下载，中断即留空目录且下次重下。稳定做法：`curl -C - -x <proxy>` 断点续传 zip → 解压到 `target/webrtc-sdk/`（zip 根目录是 `linux-x64-release`）→ 构建时设 `LK_CUSTOM_WEBRTC=<abs>/target/webrtc-sdk/linux-x64-release` 跳过下载；C++ 侧仍需 `CXXFLAGS=-fpermissive`。
- **确认「本轮是否引入未汉化文案」用计数对比**：对 merge 变动的 rs 文件，统计 `Label::new("…")`、`Tooltip::text("…")`、`SectionHeader("…")`、`.entry("…")`、`Button::new(id, "…")` 等模式里**未包 `t!` 的命中数**，与 `HEAD:<same file>` 比较；只有计数上升才是本轮引入，历史欠账不会被误报成回归。

## 特性保留清单（每次合并后逐项核对）

- `crates/zed_i18n/locales/zh-CN/*` 汉化文件齐全。
- `crates/tabular_data_preview/*`（表格/SQLite 预览）；表内联设置面板的挂载点是上游 `TableView`，勿回落到 `TabularDataPreviewPane`。
- `crates/settings_ui/*` 的设置项标题/描述为 `SharedString`（运行期本地化）；取上游新代码后补 `.into()`，上游新增设置项要与 `[SettingsPageItem; N]` 数组长度、`terminal_panel_section`/`git_hunks_section` 等分节数组同步。
- `crates/fs/src/fs_watcher.rs` 的 `check_watch_health`（watch 健康检查）——含 `WatchBackend::watched_paths`；上游新增的 test 后端需补实现（见要点）；上游重构 `PathRegistrationState` 后要按新 `coverage` 模型重写孤儿判定；健康检查线程用 `Weak<OsWatcher>` 持有 watcher，否则会把 `OsWatcher` 及其 `BackgroundExecutor` 永久钉住（上游 `watcher_retains_scheduler_until_teardown` 用例会失败）。
- `.github/workflows/` 仅 `release.yml` + `run_tests.yml`（精简 CI，禁止 `cargo xtask workflows` 覆盖）；上游删除 workflow 文件与 fork 的删除产生 delete/modify 冲突时取删除。
- `crates/agent_ui/src/conversation_view/thread_view.rs` 的 `render_sandbox_authorization_path_row` 必须保持上游 `granted: &settings::GrantedWritePath` 签名（含 Source / Target / Write Path 三态），fork 曾长期保留 #58358 之前的旧实现；标题走 `agent_ui.thread_view.{sandbox_source_path,sandbox_target_path,requested_write_path}`。
- `crates/acp_thread/src/acp_thread.rs` 的 `resolve_location` 用 `project.fs().metadata()` 跳过目录路径（上游未修，会为每个目录位置记 EISDIR 错误并可能在项目外创建工作树）；合并后确认该判断仍在，`FakeFs::read_call_count` 仍是 `test_tool_locations_skip_directories` 的断言依据。
- `crates/auto_update` 的自建更新通道：`GITHUB_RELEASE_REPO`（现名 `ReachForStar/zed-ZH_CN`）、`release_tag` 去 `pre`/`build` 后拼 tag、`select_github_asset` 资产选择、`fork_release_notes` + `select_release_notes` 的更新说明三级兜底，以及 `release_notes_url` 指汉化版 GitHub Release 页面；`crates/zed/src/zed.rs` 关于页的 `ZH_CN_REPO_URL` 必须是同一仓库。上游若改 `github_releases::GithubRelease` 字段，fork 依赖的 `body` 要跟着核对。`crates/auto_update` 的 dev-dependencies 需保留 `db` 的 `test-support` feature，否则 `test_auto_update_downloads` 在 `db::ThreadSafeConnection::global` panic。
- `README.md` 顶部 `> [!IMPORTANT]` 与 `> Remove this line ...` 两行完好（.rules 硬规则，勿删）。

## 关联页面

- [应用内查看 Release Note 空白](../queries/release-notes-blank-fork-channel.md) — 版本号超前于上游发布线时 zed.dev 更新说明正文为空；`auto_update` 以自建 GitHub Release 正文兜底。
- [2026-11-25 合并 upstream/main（1.31.0 线，68 提交）](../queries/upstream-merge-2026-11-25.md) — 本轮 13 处冲突判定、上游旧实现回填、ACP SDK `MaybeUndefined` 适配、`LC_ALL=C` 测试环境坑与「上游新增行集合核对」方法，38 套件 7030 项测试数值。
- [2026-11-23 合并上游 main（1.31.0，135 提交）](../queries/upstream-merge-2026-11-23.md) — 本轮冲突判定、`language_model_selector` 缺 `zed_i18n` 的编译失败、既有 fork 回归（窗口失活守卫、删除确认框收敛、`location()` 语义）修复与验证数值。
- [2026-11 合并上游 main：冲突消解、编译失败链与 fork 特性核对](../queries/upstream-merge-2026-11.md) — 本轮（1.29.0，77 提交）的冲突决策、编译失败链与 fork 修复核对表。
- [2026-10 合并上游 main：冲突消解决策与编译失败链](../queries/upstream-merge-2026-10.md) — 上一轮合并的决策、clippy 失败链、历史丢失去向与本地校验清单。
- [汉化口径与批量替换工作流](i18n-localization-workflow.md) — 合并后补齐汉化的口径、批量脚本与 `t!` 类型适配。
- [msvc_spectre_libs 构建失败](../queries/msvc-spectre-libs-build-failure.md) — 本地编译 repl/zed 的环境前置。
- [ACP 面板自动压缩上下文不生效](../queries/acp-auto-compact-external-agents.md) — fork 在 acp_thread 层的自有特性；同批修复 fs `watched_paths` merge 遗留。
- [新建智能体会话慢](../queries/slow-new-agent-session.md) — 外部智能体二进制首次下发与目录路径误读。
