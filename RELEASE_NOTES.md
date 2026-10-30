本版本把 Zed 汉化版从已发布的 v1.24.2 更新到基于 **Zed v1.28.0**（版本号与上游 `main` 同步），并新增 Ubuntu / Debian 的 `.deb` 安装包。

### Zed 上游更新 (v1.24.2 → v1.28.0)

合入 upstream/main 五次同步累积 220 个上游提交，保留全部 fork 特性与汉化：

- **ACP 会话间消息**：新增 `AgentThreadEntry::SessionMessage`，父会话与子智能体之间的委派消息在对话记录中独立成行（`To <接收方>` / `From <发送方>`），参与者名称可点击跳转到对应会话。
- **子智能体卡片重构**：卡片标题与状态文案迁到 `subagent_card.rs`，新增 `SubagentStatus::Unknown`；停止按钮补齐 `debug_selector`，便于自动化与测试定位。
- **git 面板**：历史标签页新增 Refs 列（可隐藏、可调宽窄），打开的提交在历史列表中保持高亮；新增自动拉取（默认分支远端与当前分支上游）及对应设置项。
- **Codeberg**：支持在 Codeberg 上通过界面创建 Pull Request。
- **gpui**：动画改用执行器时钟、arena 用 `Rc` 包裹避免 UB、文本换行改用 UAX #14、继承透明度变化时重绘缓存视图；macOS 动画掉帧与 M5 机型的帧间隔问题修复。
- **依赖与协议**：ACP SDK 升级到 3.6.0（schema 1.13，`AvailableCommand` 字段改为访问器），`livekit-rust-sdks` 升级到使用 `zip` 9.1 的版本（清理三项 zip 安全公告）。
- **移除**：agent checkpoints 及「Restore Checkpoint」按钮（每条消息前的快照在大仓库上拖慢甚至卡死发送）。

完整上游对比：https://github.com/zed-industries/zed/compare/28d0a3547f...422b53cff8

### 汉化 (本 fork)

- 新增 git 面板「推送被拒绝」提示、历史标签页 Refs 列、子智能体状态未知的 tooltip，以及可恢复消息各状态行的明细 tooltip。
- 新增 Git 自动拉取两个设置项的标题与描述（`git_auto_fetch_*`），en / zh-CN 双侧同步。
- 移除 checkpoints 后清理失效的 `restore_checkpoint*` 键；汉化键完整性检查 `scripts/i18n/check-missing-keys.py` 保持 0 问题。

### 修复 (本 fork)

- **长会话内存回收**：上下文压缩完成后释放被摘要覆盖的消息与工具输出；修正丢弃区间会跨过「进行中」压缩行导致历史错乱的问题。
- **自动压缩钩子**：跟进 ACP SDK 3.6 的 `AvailableCommand::name()` 访问器。
- **自更新通道**：上游移除 `http_client::github` 后改用 `github_releases` crate，GitHub Releases 检查逻辑保持可用。
- `script/lib/workspace.ps1` 的 `cargo metadata` 补 `--format-version=1`，消除每次构建的告警。

### 安装包与发布链路

- **Ubuntu / Debian**：新增 `zed-zh-cn_<版本>_amd64.deb`，安装到 `/usr/lib/zed`，注册 `/usr/bin/zed`、桌面项与图标；`sudo apt install ./zed-zh-cn_<版本>_amd64.deb` 会自动装依赖。
- **Windows**：`Zed-x86_64.exe`（Inno Setup 安装器）；**macOS**：`Zed-aarch64.dmg`。
- **版本号**：与上游 `main` 一致，`scripts/sync-upstream-version.py` 负责同步，发布前由 `verify_version` job 校验，避免标签、Release 标题与二进制自报版本错位。
- `run_tests.yml` 仅手动触发（`workflow_dispatch`），`check_style` 含汉化键完整性检查。

### 已知说明

- 本构建未经代码签名：Windows 可能触发 SmartScreen；macOS 可能被 Gatekeeper 拦截（右键「打开」或执行 `xattr -dr com.apple.quarantine /Applications/Zed.app`）。
- 详细变更列表见下方自动生成的变更记录。
