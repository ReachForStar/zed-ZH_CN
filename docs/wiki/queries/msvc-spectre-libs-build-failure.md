---
title: msvc_spectre_libs 构建失败：缺 Spectre 缓解库
type: query
tags: [build, msvc, visual-studio, spectre, windows]
created: 2026-09-26
updated: 2026-09-26
status: active
---

# msvc_spectre_libs 构建失败：缺 Spectre 缓解库

## 问题

Windows 上 `cargo check/build` 编译 `repl`、`zed` 时，`msvc_spectre_libs` 的 build script 报错：

```
cargo:warning=No spectre-mitigated libs were found. Please modify the VS Installation to add these.
```

并导致构建中止（`--- stdout / --- stderr` 包裹格式 = build script 失败）。其余 crate 不受影响。

## 根因

- `msvc_spectre_libs` 的 build.rs 在找不到 `<VS>\VC\Tools\MSVC\<工具集>\lib\spectre\<arch>` 目录时，仅当启用 `error` feature 才 panic，否则只告警。
- 依赖链 `pet-*`（python-environment-tools，git 依赖）→ `repl` → `zed` 中，pet 各 crate 硬编码 `msvc_spectre_libs = { features = ["error"] }`，因此**必然 panic**，无法用配置关掉。
- build.rs 通过 `cc::windows_registry::find_tool` 选 cl.exe，cc 选**最新工具集**。本机 VS 2026 Community 有工具集 `14.44.35207`（有 spectre 库）与 `14.51.36231`（无 spectre 库），构建选 14.51 → 缺库 → panic。
- 已装的 Spectre 组件是 `MSVC v143`(=14.44) 与 `v141` 的，**不含 14.51**，故不生效。

## 解法

给最新工具集 14.51 安装 Spectre 缓解库组件：

- 组件 ID：`Microsoft.VisualStudio.Component.VC.14.51.x86.x64.Spectre`
- GUI：VS Installer → 修改 VS 2026 Community → 单个组件 → 搜 `Spectre` → 勾选 14.51/最新 MSVC 的 x64/x86 Spectre 缓解库。
- 验证：`...\VC\Tools\MSVC\14.51.36231\lib\spectre\x64\libcmt.lib` 存在；随后 `cargo check -p repl -p zed` 通过（CHECK_EXIT=0）。

## 涉及模块

`crates/repl`、`crates/zed`（经 pet-* 传递依赖 msvc_spectre_libs）。

## 复发预防

- 升级 VS/MSVC 工具集后若再报此错，检查**最新**工具集是否有 `lib\spectre\<arch>`；新工具集需装对应版本的 Spectre 组件（组件 ID 中版本号随工具集变，如 14.51）。
- 多 VS 实例时以 `cc` 实际选中的最新工具集为准，勿只看“已装过某个 Spectre 组件”。
- 本机另有 2 个无 C++ 工具集的 BuildTools 实例（2022/18 x86），不影响选择（cc 只认有工具集的实例）。
