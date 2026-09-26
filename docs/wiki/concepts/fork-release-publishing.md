---
title: fork 发布 release 的流程与版本号策略
type: concept
tags: [release, tag, workflow, fork, versioning]
created: 2026-09-26
updated: 2026-09-26
status: active
---

# fork 发布 release 的流程与版本号策略

## 定义

本 fork 通过推送 `v*` 标签发布汉化版安装包，由仓库内精简版 `.github/workflows/release.yml` 完成三平台构建并创建 GitHub Release。

## 要点

- **触发方式**：推送 `v*` 标签（或 workflow_dispatch 手动触发）。release job 依赖 `bundle_linux_x86_64` / `bundle_windows_x86_64` / `bundle_macos_aarch64` 三个构建 job（单 job 超时 360 分钟），产物由 `softprops/action-gh-release@v2` 上传，`make_latest: true`。
- **版本号来源**：`script/get-crate-version zed` 读 `crates/zed/Cargo.toml` 的 `version`，用于 Release 标题 `Zed {version} (汉化版)` 与发布说明头部"基于 Zed v{version}"；tag 触发时标签名即 `v{...}`。改版本需同步 `Cargo.lock` 中 `name = "zed"` 条目，否则 lock 不一致。
- **发布通道**：仓库内 `crates/zed/RELEASE_CHANNEL` 保持 `dev`（本地开发共存），各 bundle job 检出后统一覆盖为 `stable` 再构建，无需修改仓库文件。
- **版本号策略（2026-09-26 决策）**：取**上游已发布的最新 release 版本号**（当时上游 release 最新为 v1.21.0），而不是上游 main 的开发版本号（Cargo.toml 里是 1.23.0，系 1.22/1.23 切分支后的预升号，上游尚未发布）。这样 tag、Release 标题、二进制自报版本三者一致。代价：fork 的 `crates/zed/Cargo.toml` 与 upstream/main 长期存在一行版本差异，合并时该行必冲突，取 fork 侧数值即可。
- **推送**：SSH :22 被墙，标签与 main 一律走 HTTPS（经 git-proxy）：`git push https://github.com/ReachForStar/zed-ZH_CN.git main v1.21.0`；推后 `git update-ref refs/remotes/fork/main <sha>` 同步跟踪引用。
- **轻量标签**：沿用既有惯例，直接在 main 提交上打轻量附注外标签（`git tag vX.Y.Z`，不单独建 release 分支）。

## 验证与遗留

- 2026-09-26 发布 v1.21.0：标签推送触发 release（三平台构建）与 run_tests；run_tests 的**代码风格检查 job 通过**，确认 typos 配置路径修复生效（此前该 job 均在拼写检查一步 1 分钟内失败，见 [fork-upstream-sync](fork-upstream-sync.md)）。
- 遗留：`script/lib/workspace.ps1` 的 `cargo metadata --no-deps --offline` 未带 `--format-version=1`，新版 cargo 每次构建告警；无害、经用户决定**留到下个版本再改**。

## 关联页面

- [汉化 fork 同步 upstream 的合并模式与特性保留清单](fork-upstream-sync.md) — 合并时版本号行的处理。
