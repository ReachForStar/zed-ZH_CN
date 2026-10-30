---
title: macOS 打包要求 SDK 26：macos-14 runner 上 bundle-mac 失败
type: query
tags: [release, ci, macos, xcode, sdk, workflow]
created: 2026-10-30
updated: 2026-10-30
status: active
---

# macOS 打包要求 SDK 26：macos-14 runner 上 bundle-mac 失败

## 问题

推送 `v1.27.0` 标签触发的 release run 里，macOS 构建 job 在「打包 macOS（未签名）」这一步失败：

```
macOS SDK 26 or newer is required; set SDKROOT to a valid macOS SDK directory
(selected: /Applications/Xcode_16.2.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX15.2.sdk).
```

Linux 与 Windows 两个 job 正常完成，release job 因 `needs` 未满足被跳过，本轮没有产生 Release。

## 根因

两层原因叠加：

1. 上游已要求 macOS 12 与 **macOS SDK 26**（上游提交 `f14592c4a5` "Require macOS 12 and SDK 26"），`script/bundle-mac` 会校验 `SDKROOT`/SDK 版本，低于 26 直接退出。
2. fork 的 macOS job 固定在 `runs-on: macos-14`。该镜像已标记 deprecated，自带的 Xcode 最高为 16.x，只装 macOS 15.x SDK；job 里「选择 Xcode（跳过不兼容的 15.4）」选出的 Xcode_16.2 只能提供 `MacOSX15.2.sdk`。上游对应 job 用的是 `namespace-profile-mac-large-xcode26`（自带 Xcode 26 的 runners）。

## 解法

- job 改为 `runs-on: macos-26`（Apple Silicon，默认 Xcode 26.6，含 macOS 26.5 SDK；`macos-14` 已废弃）。
- 删掉「选择 Xcode 跳过 15.4」步骤，改为显式校验步骤：打印 `xcodebuild -version`、`xcrun --sdk macosx --show-sdk-version/-path`，SDK 主版本低于 26 就失败并提示换 runner。
- 保留「仅保留当前 Xcode」的磁盘清理步骤（macos-26 镜像带 26.0～26.6 多份副本，每份十 GB 量级）；删改后的清理逻辑仍以 `xcode-select -p` 为准，保留含 metal 的完整工具链。

## 验证

- 修复提交 `94b50ea89c`，用 `workflow_dispatch`（`dry_run=false`）重跑 release run `41066083715`：`校验版本号与上游一致`、Linux、Windows、macOS 四个 job 全绿，release job 创建 Release v1.27.0。
- 产物 7 个：`Zed-aarch64.dmg`、`Zed-x86_64.exe`、`zed-linux-x86_64.tar.gz`、`zed-zh-cn_1.27.0_amd64.deb`、三个 `zed-remote-server-*`。

## 复发预防

1. runner 标签不要长期钉在会被弃用的版本上；`macos-14` 已 deprecated，镜像与 SDK 都落后于上游要求（见 actions/runner-images 的 Available Images 表）。
2. 打包前显式校验工具链（SDK 版本、`xcodebuild -version`），不要假定 runner 默认值满足要求；校验失败直接失败并给出修复提示。
3. 上游打包脚本的环境要求会随上游提交变化（例如要求 SDK 26），合并上游后对 release.yml 的平台环境做一次核对。
4. 标签触发失败后需要重跑：本次用 `workflow_dispatch` 从 main 重跑，因此 Release 的 `target_commitish` 是 main HEAD（`94b50ea89c`），而标签 `v1.27.0` 仍指向更早的 `077d25fa39`（差异仅为 wiki 与 workflow 改动）。若要三者完全对齐，需要在修复提交上重打标签。

## 涉及模块

- `.github/workflows/release.yml`（macOS job 的 runner 与工具链校验）
- `script/bundle-mac`（SDK 校验来源，上游维护）

## 关联页面

- [fork 发布 release 的流程与版本号策略](../concepts/fork-release-publishing.md)
- [汉化 fork 同步 upstream 的合并模式与特性保留清单](../concepts/fork-upstream-sync.md)