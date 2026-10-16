本版本将 Zed 汉化版从 v1.21.0 更新到基于 **Zed v1.24.2**，并持续完善汉化覆盖与发布链路。

### Zed 上游更新 (v1.21.0 → v1.24.2)

- 合入 upstream/main（开发基点 `28d0a3547f`，8 个提交、39 个文件，零冲突），保留全部 fork 特性与汉化。
- 涵盖 ACP v2 收据提交链、`language`/`language_core` 高亮缓存、`languages/*` LSP 适配、`gpui_wgpu` 升级、`gpui_ios` 示例改为 Cargo example 目标等。

完整上游对比：https://github.com/zed-industries/zed/compare/v1.21.0...v1.24.2

### 汉化 (本 fork)

- 补齐 previously missed 14 个可恢复消息相关文案（recoverable submissions 状态、Copy/Restore/Discard 按钮与 tooltip、附件占位符），en/zh-CN 双侧同步。
- 上游新增文案 `Message accepted; agent connection lost`、`Connection lost; message acceptance is unknown` 已按约定补键并转为 `t!`。
- 汉化键完整性检查 `scripts/i18n/check-missing-keys.py` 已接入 CI（`check_style` job）。
- 全量 `cargo clippy --workspace --all-targets -- --deny warnings` 通过。

### 构建与发布链路

- `run_tests.yml` 改为仅手动触发（`workflow_dispatch`），避免每次文案提交拉起全量 clippy；需要时手动运行。
- 平台产物：Linux x86_64、Windows x86_64、macOS aarch64（未签名）。

### 已知说明

- 本构建未经代码签名：Windows 可能触发 SmartScreen；macOS 可能被 Gatekeeper 拦截（右键「打开」或执行 `xattr -dr com.apple.quarantine /Applications/Zed.app`）。
- 详细变更列表见上方「Full Changelog」链接。
