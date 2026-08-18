# Zed 汉化版 (ZH_CN)

[![Zed](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/zed-industries/zed/main/assets/badge/v0.json)](https://zed.dev)
[![CI](https://github.com/zed-industries/zed/actions/workflows/run_tests.yml/badge.svg)](https://github.com/zed-industries/zed/actions/workflows/run_tests.yml)

本仓库是 [Zed](https://github.com/zed-industries/zed) 的**简体中文汉化分支**，在保持与上游同步的基础上，提供完整的中文界面翻译。

Zed 是一款高性能、多人协作代码编辑器，由 [Atom](https://github.com/atom/atom) 和 [Tree-sitter](https://github.com/tree-sitter/tree-sitter) 的创建者开发。

---

## 汉化说明

### 覆盖范围

汉化通过 `crates/zed_i18n` 实现，当前已翻译 **52 个模块**的界面文本，涵盖：

- 编辑器核心 UI（命令面板、文件查找、诊断、项目面板、搜索、大纲、终端等）
- AI / Agent 面板（agent_ui、ai_onboarding 等）
- 调试器界面（debugger_ui、debugger_tools）
- Git 相关（git_ui、collab_ui）
- 文档与媒体预览（markdown_preview、image_viewer、office_preview、csv_preview、svg_preview）
- 设置与选择器面板（settings_ui、keymap_editor、主题/语言/编码选择器等）
- 扩展管理、反馈、自动更新等

### 使用方式

在 Zed 设置中添加：

```json
{
  "ui_language": "zh-CN"
}
```

即可切换为中文界面。设为 `"en"` 或不配置则使用英文。

### 翻译文件结构

```
crates/zed_i18n/locales/
├── en/            # 英文（上游原始文本）
├── zh-CN/         # 简体中文翻译（52 个 .toml 文件）
├── en.toml
└── zh-CN.toml
```

每个 crate 对应一个 `<crate_name>.zh-CN.toml` 文件，通过 `t!` 宏在运行时按 key 查找译文。

### 参与翻译

1. 在 `crates/zed_i18n/locales/zh-CN/` 下找到对应模块的 `.toml` 文件
2. 补全或修正翻译 key
3. 确保与 `en/` 目录下的 key 保持一一对应
4. 提交 PR 到本仓库

---

### 安装

macOS、Linux 和 Windows 可直接[下载 Zed](https://zed.dev/download)，或通过包管理器安装（[macOS](https://zed.dev/docs/installation#macos)/[Linux](https://zed.dev/docs/linux#installing-via-a-package-manager)/[Windows](https://zed.dev/docs/windows#package-managers)）。

> 本汉化版需自行从源码构建，构建方式见下方开发指南。

其他平台暂不可用：

- Web（[跟踪讨论](https://github.com/zed-industries/zed/discussions/26195)）

### 开发 Zed

- [macOS 构建指南](./docs/src/development/macos.md)
- [Linux 构建指南](./docs/src/development/linux.md)
- [Windows 构建指南](./docs/src/development/windows.md)

### 贡献

参见 [CONTRIBUTING.md](./CONTRIBUTING.md) 了解贡献方式。

汉化相关贡献请直接提交 PR 至本仓库（[MindFlowLab/zed-ZH_CN](https://github.com/MindFlowLab/zed-ZH_CN)）。

### 许可证

Zed 源代码主要采用 GPL-3.0-or-later 许可，标注部分采用 Apache-2.0。

第三方依赖的许可信息必须正确提供以通过 CI。

我们使用 [`cargo-about`](https://github.com/EmbarkStudios/cargo-about) 自动合规开源许可。如果 CI 失败，检查以下情况：

- 你创建的 crate 报 `no license specified`？在 Cargo.toml 的 `[package]` 下添加 `publish = false`。
- 依赖报 `failed to satisfy license requirements`？确认该项目的许可证及合规方式，然后将 SPDX 标识符添加到 `script/licenses/zed-licenses.toml` 的 `accepted` 数组。
- `cargo-about` 找不到依赖的许可证？在 `script/licenses/zed-licenses.toml` 末尾添加 clarification 字段，参见 [cargo-about 文档](https://embarkstudios.github.io/cargo-about/cli/generate/config.html#crate-configuration)。

## 赞助

Zed 由 **Zed Industries, Inc.** 开发。如希望支持上游项目，可通过其 GitHub Sponsors 赞助，资金直接进入 Zed Industries，不附带任何特权。

本汉化版由 **ReachForStar** 独立维护，与 Zed Industries 无隶属关系。如果你认可这份本地化工作，欢迎扫码支持本仓库的持续维护：

| 微信支付 | 支付宝 |
|:---:|:---:|
| <img src="assets/donate/wechat.png" alt="微信支付收款码" width="220"> | <img src="assets/donate/alipay.png" alt="支付宝收款码" width="220"> |

---

## 关键词 / Keywords

**中文**
Zed 汉化、Zed 汉化版、Zed 中文、Zed 中文版、Zed 简体中文、Zed 中文设置、Zed 中文教程、Zed 改中文、Zed 语言设置、Zed 界面汉化、代码编辑器汉化、中文代码编辑器、中文 AI 编辑器、编辑器国际化、i18n 中文翻译、Zed 翻译补丁、高性能编辑器中文、协作编辑器汉化

**English**
Zed Chinese localization, Zed zh-CN, Zed Simplified Chinese, Zed Chinese translation, Zed language pack, Zed i18n, Zed localization patch, Zed editor Chinese, Chinese code editor, Chinese AI code editor, Rust editor internationalization, GPUI i18n, Zed UI translation, how to change Zed to Chinese

**相关技术 / Related**
Zed, Zed editor, Rust, GPUI, rust-i18n, i18n, internationalization, localization, 国际化, 本地化, 多语言, multi-language, code editor, 代码编辑器, AI editor, collaborative editing, 协作编辑

