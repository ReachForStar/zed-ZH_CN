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
- **vendor 依赖保留**：fork 把 notify vendor 到 `third_party/notify`（含 Windows 溢出修复），workspace Cargo.toml 用 `path=`；upstream 用 git rev。合并时保留 fork 的 path vendor 版本。

## 特性保留清单（每次合并后逐项核对）

- `crates/zed_i18n/locales/zh-CN/*` 汉化文件齐全。
- `crates/tabular_data_preview/*`（表格/SQLite 预览）。
- `crates/fs/src/fs_watcher.rs` 的 `check_watch_health`（watch 健康检查）。
- `.github/workflows/` 仅 `release.yml` + `run_tests.yml`（精简 CI，禁止 `cargo xtask workflows` 覆盖）。
- `README.md` 顶部 `> [!IMPORTANT]` 与 `> Remove this line ...` 两行完好（.rules 硬规则，勿删）。

## 关联页面

- [msvc_spectre_libs 构建失败](../queries/msvc-spectre-libs-build-failure.md) — 本地编译 repl/zed 的环境前置。
