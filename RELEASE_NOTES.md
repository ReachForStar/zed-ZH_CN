本版本将 Zed 汉化版从 v1.24.2 更新到基于 **Zed v1.25.3**，并持续完善汉化覆盖与发布链路。

### Zed 上游更新 (v1.24.2 → v1.25.3)

合入 upstream/main 三次同步累积 127 个上游提交，保留全部 fork 特性与汉化：

- **ACP 会话间消息**：新增 `AgentThreadEntry::SessionMessage`，父会话与子智能体之间的委派消息在对话记录中独立成行（`To <接收方>` / `From <发送方>`），参与者名称可点击跳转到对应会话。
- **流式错误类型化**：OpenAI 兼容传输层把供应商拒绝、解码失败、读取失败保留为 `LanguageModelCompletionError`，不再装箱成 `anyhow`，调用方可以继续按错误类别判断重试；Google 与 OpenAI Responses 流同步跟进。
- **ACP SDK 3.5.0**、`gpui_wgpu` 的 wgpu 升级（修复 Vulkan surface 销毁）、fs watcher 保留子监视与折叠路径归一化、多缓冲区折叠块的词级 diff 修复、Agent 线程数据库读取修复等。

完整上游对比：https://github.com/zed-industries/zed/compare/v1.24.2...v1.25.3

### 汉化 (本 fork)

- 新增「会话间消息」整块界面文案 6 个键：`session_message_to`（发送至）、`session_message_from`（来自）、`session_message_between_sessions`（会话之间的消息）、`open_session`（打开会话）、`unavailable_session`（不可用的会话）、`main_agent`（主智能体）；en / zh-CN 双侧同步。
- 接上此前未接入 locale 的子智能体兜底标签（`agent_ui.thread_view.subagent`），此前该位置固定显示英文 `Subagent`。
- 汉化键完整性检查 `scripts/i18n/check-missing-keys.py` 保持 0 问题；全量 `cargo clippy --workspace --all-targets -- --deny warnings` 通过。

### 修复 (本 fork)

- **长会话内存回收**：上下文压缩完成后释放被摘要覆盖的消息与工具输出。修正丢弃区间会跨过「进行中」压缩行的问题——原先会把尚在生成中的记录整体丢弃，随后的分片又把它们重建成新行、打乱历史顺序。
- **CRLF 行尾还原脚本**移入 `scripts/normalize-crlf.py`：合并后提交前统一工作区行尾，避免 rustfmt 写成 LF 后与仓库其余文件混用。
- `script/lib/workspace.ps1` 的 `cargo metadata` 补 `--format-version=1`，消除每次构建的告警。

### 构建与发布链路

- `run_tests.yml` 仅手动触发（`workflow_dispatch`），避免文案提交拉起全量 clippy；`check_style` 含汉化键完整性检查。
- 平台产物：Linux x86_64、Windows x86_64、macOS aarch64（未签名）。

### 已知说明

- 本构建未经代码签名：Windows 可能触发 SmartScreen；macOS 可能被 Gatekeeper 拦截（右键「打开」或执行 `xattr -dr com.apple.quarantine /Applications/Zed.app`）。
- 详细变更列表见下方自动生成的变更记录。
