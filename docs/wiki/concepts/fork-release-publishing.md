---
title: fork 发布 release 的流程与版本号策略
type: concept
tags: [release, tag, workflow, fork, versioning]
created: 2026-09-26
updated: 2026-10-24
status: active
---

# fork 发布 release 的流程与版本号策略

## 定义

本 fork 通过推送 `v*` 标签发布汉化版安装包，由仓库内精简版 `.github/workflows/release.yml` 完成三平台构建并创建 GitHub Release。

## 要点

- **触发方式**：推送 `v*` 标签（或 workflow_dispatch 手动触发）。release job 依赖 `bundle_linux_x86_64` / `bundle_windows_x86_64` / `bundle_macos_aarch64` 三个构建 job（单 job 超时 360 分钟），产物由 `softprops/action-gh-release@v2` 上传，`make_latest: true`。
- **版本号来源**：`script/get-crate-version zed` 读 `crates/zed/Cargo.toml` 的 `version`，用于 Release 标题 `Zed {version} (汉化版)` 与发布说明头部"基于 Zed v{version}"；tag 触发时标签名即 `v{...}`。改版本需同步 `Cargo.lock` 中 `name = "zed"` 条目，否则 lock 不一致。
- **发布通道**：仓库内 `crates/zed/RELEASE_CHANNEL` 保持 `dev`（本地开发共存），各 bundle job 检出后统一覆盖为 `stable` 再构建，无需修改仓库文件。
- **版本号策略（2026-09-26 决策）**：取**上游已发布的最新 stable release 版本号**，而不是上游 main 的开发版本号。例如 2026-10-24 发版取 v1.25.3（上游 main 当时已预升到 1.27.0）。这样 tag、Release 标题、二进制自报版本三者一致。代价：fork 的 `crates/zed/Cargo.toml` 与 upstream/main 长期存在一行版本差异，合并时该行必冲突，取 fork 侧数值即可。
- **版本号判定要先分清上游两条线**：stable/preview 的 tag 提交、`upstream/main` 是不同分支。`git merge-base --is-ancestor <tag> upstream/main` 对 stable tag 通常是**否**（v1.25.3 的父提交就不在 main 上），不能据此判断"上游没发布”。权威依据是 GitHub Releases API 的 `prerelease=false` 清单，而不是 main 的 Cargo.toml 版本号。
- **fetch 上游 tag 会污染本地标签**：`git fetch <url> refs/tags/v...` 会把上游 tag 直接写进本地 `refs/tags/`，与 fork 的发布标签同名冲突（本次误入 7 个）。fetch 上游 tag 时必须重命名（`refs/tags/vX:+refs/tags/upstream-vX`），用完即删；否则打 fork 标签前先核对 `git rev-parse refs/tags/vX` 指向提交是否在 fork 历史里。
- **推送**：SSH :22 被墙，标签与 main 一律走 HTTPS（经 git-proxy）：`git push https://github.com/ReachForStar/zed-ZH_CN.git main v1.21.0`；推后 `git update-ref refs/remotes/fork/main <sha>` 同步跟踪引用。
- **轻量标签**：沿用既有惯例，直接在 main 提交上打轻量附注外标签（`git tag vX.Y.Z`，不单独建 release 分支）。

## 验证与遗留

- 2026-09-26 发布 v1.21.0：标签推送触发 release（三平台构建）与 run_tests；run_tests 的**代码风格检查 job 通过**，确认 typos 配置路径修复生效（此前该 job 均在拼写检查一步 1 分钟内失败，见 [fork-upstream-sync](fork-upstream-sync.md)）。
- 标签指向的提交随后在 Clippy 阶段连续暴露三层失败（typos 已修 → extensions_ui dead_code → acp_thread redundant_clone），全绿后需删除旧标签、在修复提交上重打 v1.21.0 再触发正式构建；细节见 [run_tests Clippy 失败链](../queries/run-tests-clippy-failure-chain.md)。
- 2026-10-24 发布 v1.25.3（上游 main 开发号为 1.27.0，取上游 stable 最新版）：发版前补齐了此前的遗留——`script/lib/workspace.ps1` 的 `cargo metadata` 补上 `--format-version=1`（实测去掉该参数会打 `please specify --format-version` 告警，补上后无输出）。
- 残留验证：`script/get-crate-version` 在 CRLF 工作区无法直接执行（`#!/usr/bin/env bash\r`），本地核验版本号用 `git show :script/get-crate-version > tmp/x.sh && bash tmp/x.sh zed`，或直接读 `cargo metadata --no-deps --format-version=1`。

## 关联页面

- [汉化 fork 同步 upstream 的合并模式与特性保留清单](fork-upstream-sync.md) — 合并时版本号行的处理。
