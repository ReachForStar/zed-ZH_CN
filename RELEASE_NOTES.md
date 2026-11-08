本版本发布为 **v1.29.0**，代码基线是 upstream/main 的 `b5cd310b3b5`（上游该提交的 `Cargo.toml` 为 1.29.0，上游正式发布线最新为 1.27.0）。

### Zed 上游更新（v1.27.0 之后）

合入 upstream/main 77 个上游提交，保留全部 fork 特性与汉化：

- **ACP v2**：支持 plan 操作与后台工作，子智能体追踪各自的工作目录；`agent-client-protocol` 升级到 5.0.0（模块改为 `agent_client_protocol::v2::schema`）。
- **智能体面板**：自动生成会话标题可以关闭；复制回复后显示反馈；修复压缩被计为非缓存输入 token、云端显式压缩期间的 keepalive、OpenAI 订阅压缩保留缓存前缀。
- **git 面板**：新增日期格式与各视图的日期样式设置；可查看相对基线分支的变更；把文件拖到终端；提交信息视图保留换行；提交视图滚动到被 blame 的文件；JetBrains 键位 `Ctrl-K` 聚焦提交信息。
- **编辑器与终端**：终端和 diff 视图支持鼠标滚轮缩放；终端标签页圆角；隐藏不可切换标签上的只读开关；工作树选择器支持按分支名搜索。
- **Vim**：替换模式支持 `\+` 与 `\{n\}`；修复换行处 visual-block 挂起。
- **调试器**：默认面板尺寸可配置；修复 CodeLLDB 内置推断配置的 schema 校验。
- **键位**：修复 macOS `backspace`/`delete` 的按键等价物与 JetBrains 键位的 `Cmd-O`。
- **Windows**：修复 `path:line` 打开、大写可执行名的 shell 检测、默认 shell 为 Nushell 时的 SSH 命令、原生标签换行，以及 COLR 图层的 CPU 合成。
- **gpui**：支持动画 WebP；修复拖拽开始时鼠标指针回退、子菜单在按键时关闭、`list()` 水平内边距被忽略；子菜单与弹出层在空间充足的一侧打开；Linux 改用 `ext-background-effect` 协议。
- **其他**：修复远程 Markdown 预览中的 SVG 图片；markdown 预览粗体遵循主题；新增 crates.io API 客户端与扩展下载 bzip2 支持。

完整上游对比：https://github.com/zed-industries/zed/compare/422b53cff8...b5cd310b3b5

### 汉化 (本 fork)

- 新增 68 个汉化键：设置页 49 个（新增设置项与分节、日期样式、状态栏编辑预测按钮等）、git 面板 15 个（分支模式、提交图、提交右键菜单、分支选择器）、智能体面板 3 个（后台任务文案）、`git_ui_core` 1 个，en / zh-CN 双侧同步。
- 汉化键完整性检查 `scripts/i18n/check-missing-keys.py` 保持 0 问题（代码静态引用 3987 键，en / zh-CN 各 4048 键）。

### 修复 (本 fork)

- **工具调用位置指向目录时的错误刷屏**：位置解析到目录（例如项目根）时不再按文件读取。原先每次解析都打开一次 buffer 并失败，实测日志一小时出现 2979 条 `reading bytes of the file "<目录>"`。`AcpThread::resolve_location` 现在先按 `metadata` 判断路径类型，目录直接返回 `None`，并新增回归测试 `test_tool_locations_skip_directories`。
- **长会话内存回收**：上下文压缩完成后释放被摘要覆盖的消息与工具输出；修正丢弃区间跨过「进行中」压缩行导致历史错乱的问题（上游无该实现，fork 保留）。
- **外部智能体自动压缩**：fork 钩子跟进 ACP SDK 5.0，自动压缩触发保持可用（上游无该实现）。
- **git 面板**：折叠条目的过滤与 Changes / History 焦点导航仍由 fork 实现提供（上游未实现）。
- **标题滚动**：改用上游的 `pending_heading_scroll` 与 `defer_parsing`，删除 fork 旧逻辑；标题内 KaTeX 字号仍为 fork 自有特性。

### 安装包与发布链路

- **Ubuntu / Debian**：`zed-zh-cn_1.29.0_amd64.deb`，安装到 `/usr/lib/zed`，注册 `/usr/bin/zed`、桌面项与图标；`Depends` 由 `ldd` 实测推导，并对 Ubuntu 24.04 的 t64 改名写备选，同时适用于 22.04 与 24.04。
- **Windows**：`Zed-x86_64.exe`（Inno Setup 安装器）；**macOS**：`Zed-aarch64.dmg`（`macos-26` runner，显式校验 SDK ≥ 26）。
- **版本号**：本次发布版本号 `1.29.0` 由发布者指定，与代码基线 upstream/main 一致；`release.yml` 校验其不低于上游最新正式 release（当前 1.27.0）、与 `Cargo.lock` 一致，且标签名与 `crates/zed/Cargo.toml` 一致。
- `run_tests.yml` 仅手动触发（`workflow_dispatch`），`check_style` 含汉化键完整性检查。

### 已知说明

- 本构建未经代码签名：Windows 可能触发 SmartScreen；macOS 可能被 Gatekeeper 拦截（右键「打开」或执行 `xattr -dr com.apple.quarantine /Applications/Zed.app`）。
- 详细变更列表见下方自动生成的变更记录。
