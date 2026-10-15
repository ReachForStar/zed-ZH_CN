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

## 特性保留清单（每次合并后逐项核对）

- `crates/zed_i18n/locales/zh-CN/*` 汉化文件齐全。
- `crates/tabular_data_preview/*`（表格/SQLite 预览）；表内联设置面板的挂载点是上游 `TableView`，勿回落到 `TabularDataPreviewPane`。
- `crates/settings_ui/*` 的设置项标题/描述为 `SharedString`（运行期本地化）；取上游新代码后补 `.into()`，上游新增设置项要与 `[SettingsPageItem; N]` 数组长度、`terminal_panel_section`/`git_hunks_section` 等分节数组同步。
- `crates/fs/src/fs_watcher.rs` 的 `check_watch_health`（watch 健康检查）——含 `WatchBackend::watched_paths`；上游新增的 test 后端需补实现（见要点）。
- `.github/workflows/` 仅 `release.yml` + `run_tests.yml`（精简 CI，禁止 `cargo xtask workflows` 覆盖）。
- `README.md` 顶部 `> [!IMPORTANT]` 与 `> Remove this line ...` 两行完好（.rules 硬规则，勿删）。

## 关联页面

- [2026-10 合并上游 main：冲突消解决策与编译失败链](../queries/upstream-merge-2026-10.md) — 本轮合并的决策、clippy 失败链、历史丢失去向与本地校验清单。
- [msvc_spectre_libs 构建失败](../queries/msvc-spectre-libs-build-failure.md) — 本地编译 repl/zed 的环境前置。
- [ACP 面板自动压缩上下文不生效](../queries/acp-auto-compact-external-agents.md) — fork 在 acp_thread 层的自有特性；同批修复 fs `watched_paths` merge 遗留。
