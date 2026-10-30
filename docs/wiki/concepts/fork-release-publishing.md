---
title: fork 发布 release 的流程与版本号策略
type: concept
tags: [release, tag, workflow, fork, versioning]
created: 2026-09-26
updated: 2026-10-30
status: active
---

# fork 发布 release 的流程与版本号策略

## 定义

本 fork 通过推送 `v*` 标签发布汉化版安装包，由仓库内精简版 `.github/workflows/release.yml` 完成三平台构建并创建 GitHub Release。

## 要点

- **触发方式**：推送 `v*` 标签（或 workflow_dispatch 手动触发）。release job 依赖 `bundle_linux_x86_64` / `bundle_windows_x86_64` / `bundle_macos_aarch64` 三个构建 job（单 job 超时 360 分钟），产物由 `softprops/action-gh-release@v2` 上传，`make_latest: true`。
- **版本号来源**：`script/get-crate-version zed` 读 `crates/zed/Cargo.toml` 的 `version`，用于 Release 标题 `Zed {version} (汉化版)` 与发布说明头部"基于 Zed v{version}"；tag 触发时标签名即 `v{...}`。改版本需同步 `Cargo.lock` 中 `name = "zed"` 条目，否则 lock 不一致。
- **发布通道**：仓库内 `crates/zed/RELEASE_CHANNEL` 保持 `dev`（本地开发共存），各 bundle job 检出后统一覆盖为 `stable` 再构建，无需修改仓库文件。
- **版本号策略（2026-10-30 决策，取代 2026-09-26 的「取上游已发布 stable 版本」）**：版本号由发布者指定（`python3 scripts/sync-upstream-version.py --version X.Y.Z`，同时写入本仓库 `crates/zed/Cargo.toml` 与 `Cargo.lock` 的 `zed` 条目），`release.yml` 的 `verify_version` job 在构建前三重校验：
  1. `--check-not-behind stable`：仓库版本不得低于上游最新正式 release（不落后）；
  2. `--check-not-behind` 内部同时校验 `Cargo.lock` 与 `Cargo.toml` 的版本一致；
  3. 标签触发时要求 `v{版本号}` 与标签名完全一致。
  这样合并上游不再为版本行产生冲突，也允许 fork 发布自选版本号（例如基于上游 main 的 `1.27.0`，虽然上游正式发布线当时是 `1.26.2`、main 的 `Cargo.toml` 是 `1.28.0`）。需要与上游完全一致时用 `--check`（配 `--source main|stable`）；`--print` 输出仓库当前版本号供 CI 比对标签。
- **版本号策略（2026-09-26 决策，已被上一条取代）**：取**上游已发布的最新 stable release 版本号**，而不是上游 main 的开发版本号。例如 2026-10-24 发版取 v1.25.3（上游 main 当时已预升到 1.27.0）。这样 tag、Release 标题、二进制自报版本三者一致。代价：fork 的 `crates/zed/Cargo.toml` 与 upstream/main 长期存在一行版本差异，合并时该行必冲突，取 fork 侧数值即可。
- **版本号判定要先分清上游两条线**：stable/preview 的 tag 提交、`upstream/main` 是不同分支。`git merge-base --is-ancestor <tag> upstream/main` 对 stable tag 通常是**否**（v1.25.3 的父提交就不在 main 上），不能据此判断"上游没发布”。权威依据是 GitHub Releases API 的 `prerelease=false` 清单，而不是 main 的 Cargo.toml 版本号。
- **fetch 上游 tag 会污染本地标签**：`git fetch <url> refs/tags/v...` 会把上游 tag 直接写进本地 `refs/tags/`，与 fork 的发布标签同名冲突（本次误入 7 个）。fetch 上游 tag 时必须重命名（`refs/tags/vX:+refs/tags/upstream-vX`），用完即删；否则打 fork 标签前先核对 `git rev-parse refs/tags/vX` 指向提交是否在 fork 历史里。
- **推送**：SSH :22 被墙，标签与 main 一律走 HTTPS（经 git-proxy）：`git push https://github.com/ReachForStar/zed-ZH_CN.git main v1.21.0`；推后 `git update-ref refs/remotes/fork/main <sha>` 同步跟踪引用。
- **标签**：直接在 main 提交上打标签，不单独建 release 分支。早期用轻量标签（`git tag vX.Y.Z`，v1.24.2 及更早），v1.27.0 起改用附注标签（`git tag -a vX.Y.Z -m "Zed 汉化版 vX.Y.Z" <sha>`）以便在 `git tag -n` 里看到版本说明；两种形式 workflow 都按标签名解析版本。
- **三平台安装包（2026-10-30 起）**：Linux 产出 `zed-zh-cn_<版本>_amd64.deb`（`script/bundle-linux-deb`，另附 `zed-linux-x86_64.tar.gz`）、Windows 产出 `Zed-x86_64.exe`（Inno Setup 安装器，`script/bundle-windows.ps1`）、macOS 产出 `Zed-aarch64.dmg`（`script/bundle-mac`）；三平台另附 `zed-remote-server-*` 包。
- **deb 打包要点**：包名 `zed-zh-cn` 并 `Conflicts`/`Replaces` `zed`，装到 `/usr/lib/zed`、软链 `/usr/bin/zed`、桌面项与 hicolor 图标放 `/usr/share`；`share/` 不重复装进 `/usr/lib/zed`。运行时 `Depends` 由 `ldd` 实测推导：跳过包内自带同名库，其余映射到 `dpkg -S` 的包名，并对 Ubuntu 24.04 的 t64 改名（`libasound2`、`libglib2.0-0`、`libblkid1`、`libmount1`、`libselinux1`）写「原名 | t64名」备选，使同一份 `.deb` 同时可装 22.04 与 24.04。
- **验证流水线**：`release.yml` 的 `workflow_dispatch` 带 `dry_run` 布尔输入，为真时只构建三平台安装包、跳过创建 Release，用于不打标签验证打包链路。

## 验证与遗留

- 2026-09-26 发布 v1.21.0：标签推送触发 release（三平台构建）与 run_tests；run_tests 的**代码风格检查 job 通过**，确认 typos 配置路径修复生效（此前该 job 均在拼写检查一步 1 分钟内失败，见 [fork-upstream-sync](fork-upstream-sync.md)）。
- 标签指向的提交随后在 Clippy 阶段连续暴露三层失败（typos 已修 → extensions_ui dead_code → acp_thread redundant_clone），全绿后需删除旧标签、在修复提交上重打 v1.21.0 再触发正式构建；细节见 [run_tests Clippy 失败链](../queries/run-tests-clippy-failure-chain.md)。
- 2026-10-24 准备 v1.25.3（上游 main 开发号为 1.27.0，取上游 stable 最新版）：版本号、`README`、`RELEASE_NOTES.md` 与 `script/lib/workspace.ps1` 的 `cargo metadata --format-version=1` 遗留一并写好，**尚未打标签发布**（GitHub Releases 最新仍是 v1.24.2），准备提交为 `110f5c907a`，`v1.25.3` 需在合并上游后另行打标签触发。
- 2026-10-30 发布 v1.27.0（**已发布**）：提交 `077d25fa39` 上打附注标签并推送触发 run 41062882626，该轮 macOS job 因 `macos-14` 只带 SDK 15 失败（详见 [macOS 打包要求 SDK 26](../queries/macos-bundle-sdk-26-requirement.md)）；修复提交 `94b50ea89c` 把 macOS job 改成 `macos-26` 并显式校验 SDK 后，用 `workflow_dispatch`（`dry_run=false`）重跑 run 41066083715，四个 job 全绿，Release `Zed 1.27.0 (汉化版)` 已创建并标记 Latest，7 个产物：`Zed-aarch64.dmg`、`Zed-x86_64.exe`、`zed-linux-x86_64.tar.gz`、`zed-zh-cn_1.27.0_amd64.deb`、三个 `zed-remote-server-*`。Linux `.deb` 安装包本轮首次上线（见上文「三平台安装包」）。
- 该 Release 的 `target_commitish` 是重跑时的 main HEAD（`94b50ea89c`），标签仍指向 `077d25fa39`：两者差异只有 wiki 与 workflow 改动；若要三者对齐，需在修复提交上重打标签。
- 残留验证：`script/get-crate-version` 在 CRLF 工作区无法直接执行（`#!/usr/bin/env bash\r`），本地核验版本号用 `git show :script/get-crate-version > tmp/x.sh && bash tmp/x.sh zed`，或直接读 `cargo metadata --no-deps --format-version=1`。

## 关联页面

- [汉化 fork 同步 upstream 的合并模式与特性保留清单](fork-upstream-sync.md) — 合并时版本号行的处理。
