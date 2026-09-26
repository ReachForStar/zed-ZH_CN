---
title: 汉化 fork 同步 upstream 的合并模式与特性保留清单
type: concept
tags: [fork, upstream, merge, i18n, workflow]
created: 2026-09-26
updated: 2026-09-26
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
- **vendor 依赖保留**：fork 把 notify vendor 到 `third_party/notify`（含 Windows 溢出修复），workspace Cargo.toml 用 `path=`；upstream 用 git rev。合并时保留 fork 的 path vendor 版本。
- **fork 给共享 trait 加方法 × 上游新增 test 后端**：fork 给某 trait（如 `fs_watcher::WatchBackend` 的 `watched_paths`）加了方法后，上游新增的实现该 trait 的测试后端（`#[cfg(test)]` / `feature="test-support"`）不会自动带上该方法，导致 **`cargo test` 编译失败但 `cargo check` 通过**（test-only 代码不参与普通 check）。合并后必跑 `cargo test -p <crate> --no-run` 或 `cargo check -p <crate> --features test-support` 才暴露。见 2026-09 的 fs `watched_paths` 遗留（fs.rs `FakeWatchBackend`、fs_watcher/diagnostics.rs `Backend`）。

## 特性保留清单（每次合并后逐项核对）

- `crates/zed_i18n/locales/zh-CN/*` 汉化文件齐全。
- `crates/tabular_data_preview/*`（表格/SQLite 预览）。
- `crates/fs/src/fs_watcher.rs` 的 `check_watch_health`（watch 健康检查）——含 `WatchBackend::watched_paths`；上游新增的 test 后端需补实现（见要点）。
- `.github/workflows/` 仅 `release.yml` + `run_tests.yml`（精简 CI，禁止 `cargo xtask workflows` 覆盖）。
- `README.md` 顶部 `> [!IMPORTANT]` 与 `> Remove this line ...` 两行完好（.rules 硬规则，勿删）。

## 关联页面

- [msvc_spectre_libs 构建失败](../queries/msvc-spectre-libs-build-failure.md) — 本地编译 repl/zed 的环境前置。
- [ACP 面板自动压缩上下文不生效](../queries/acp-auto-compact-external-agents.md) — fork 在 acp_thread 层的自有特性；同批修复 fs `watched_paths` merge 遗留。
