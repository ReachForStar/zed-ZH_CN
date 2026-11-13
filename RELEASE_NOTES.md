本版本发布为 **v1.29.1**，代码基线是 upstream/main 的 `b5cd310b3b5`（与 v1.29.0 相同）。本次为 ACP 面板的统计与自动压缩修复，不含上游新提交。

### 修复 (本 fork)

- **ACP 面板每轮 token 统计**：智能体在回合响应（`PromptResponse.usage`）里上报的 input/output 用量原先被 `acp-beta` 特性门禁丢弃，导致面板底部只显示耗时、没有 token 数。现在无论是否启用 beta 都写入统计，opencode、Claude Code、Codex 等上报 response usage 的智能体可正常显示；pi 只在 `usage_update` 上报 used/size，其每轮 token 仍无法拆分。
- **外部智能体自动压缩的触发链路**：空闲判定从「内部回合状态为空」改为「智能体上报的前台活动非运行/非等待操作」，并排除 `AgentOwned` 子会话（Zed 无权向其发 prompt）；同时在 agent 上报空闲状态时补一次阈值检查，覆盖「回合结束才在 Idle 里报用量」的 ACP v2 智能体。
- **自动压缩诊断日志**：达到阈值但智能体未公布 `compact`/`compress`、已触发过且用量尚未回落、用量回落到阈值以下重新武装，三种情况各有一条日志，`RUST_LOG=info` 可直接定位不触发的原因。
- **上下文圆环显示**：智能体从未上报窗口大小（`max_tokens` 为 0）时不再渲染永远为空、显示 0% 的圆环；占比在超过窗口时按 100% 封顶。
- 承接 v1.29.0 的修复：工具调用位置指向目录时不再错误按文件读取；上下文压缩完成后释放被摘要覆盖的条目；外部智能体自动压缩保持可用。

### 汉化 (本 fork)

- 本次无新增汉化键；汉化键完整性检查 `scripts/i18n/check-missing-keys.py` 保持 0 问题。

### 安装包与发布链路

- **Ubuntu / Debian**：`zed-zh-cn_1.29.1_amd64.deb`，安装到 `/usr/lib/zed`，注册 `/usr/bin/zed`、桌面项与图标；`Depends` 由 `ldd` 实测推导，并对 Ubuntu 24.04 的 t64 改名写备选，同时适用于 22.04 与 24.04。
- **Windows**：`Zed-x86_64.exe`（Inno Setup 安装器）；**macOS**：`Zed-aarch64.dmg`（`macos-26` runner，显式校验 SDK ≥ 26）。
- **版本号**：本次发布版本号 `1.29.1` 由发布者指定；`release.yml` 校验其不低于上游最新正式 release、与 `Cargo.lock` 一致，且标签名与 `crates/zed/Cargo.toml` 一致。
- `run_tests.yml` 仅手动触发（`workflow_dispatch`），`check_style` 含汉化键完整性检查。

### 已知说明

- 本构建未经代码签名：Windows 可能触发 SmartScreen；macOS 可能被 Gatekeeper 拦截（右键「打开」或执行 `xattr -dr com.apple.quarantine /Applications/Zed.app`）。
- 详细变更列表见下方自动生成的变更记录。