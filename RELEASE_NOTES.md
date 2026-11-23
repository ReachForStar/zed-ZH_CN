本版本发布为 **v1.31.0**，版本号与上游 `main` 的 `crates/zed/Cargo.toml` 一致（上游正式发布线最新为 v1.29.3）。代码基线为 upstream/main 的 `f126950f9d`（上游 `Cargo.toml` 为 1.31.0），本次合并了 135 个上游提交。

### Zed 上游更新（1.30.0 之后）

- **智能体**：ACP SDK 由 5.2.0 升到 5.4.0，跟进 schema 1.18 的规则变更与 v2 会话行为；ACP 配置项整体整理进智能体输入框（overflow 菜单 + 分类快捷键提示）；智能体面板显示外部智能体的授权状态；全局技能支持 SSH / WSL / dev container；新增 Ultrafast 响应速度选择器（`fast_forward_ultrafast` 图标）；`read_file` 接受字符串形式的行号；`always_confirm` 规则会在项目内文件编辑时弹确认；线程保存改为 zstd 流式写入；结构化保留模型拒答详情与未显示文本的 Anthropic thinking；保留 OpenAI Responses 的 prompt caching。
- **沙箱（新 Linux 沙箱）**：`sandbox_linux` 重写，代理事件、TCP 端口与 Unix socket 授权、隐藏路径，`mkdir` 放宽容忍度并去掉超时。
- **git**：提交视图尽快折叠缓冲区、新增提交信息模型选择器、合并 URL 在命令与输出 toast 中统一、一次 git 进程批量加载多个缓冲区的 diff base、失败后重试 diff、修复 blame 大未跟踪文件的 broken pipe。
- **gpui**：macOS 原生视图合成进场景并修焦点/按键等价/鼠标问题、SVG 变换动画不再重渲染视图、keystroke 先匹配再判上下文、Windows 剪贴板行尾规范化、新增 `simulate_window_scale_factor_change` 测试 API。
- **性能**：文件系统 watcher 路径匹配与 `WatchKey` 哈希快路径、worktree 扫描与更新、proto/worktree 更新、语言缓冲区操作线性化，编辑器 shaping 与补全排序减少克隆；大量 crate 的死代码与多余检查清理。
- **界面与编辑**：工作区菜单按 Delta checkout 分组显示、空目录不再显示展开箭头、`./` `../` 相对搜索、TypeScript/TSX 类型高亮修复、Vim `*`/`#` 与 `:s` 行为对齐、`which_key` 弹层置顶、onboarding 滚动时头部固定、toggle button 仅图标变体、通知服务调用移出 UI 线程。
- **其他**：官方 Snap 包、共享仓库走 HTTPS 拉取、Go/Rust 等扩展与安装器条款更新、corgi/Nix 构建支持、ACP registry 智能体在 musl Linux 上优先 musl 构建。

完整上游对比：https://github.com/zed-industries/zed/compare/27bbbc1cc4...f126950f9d

### 修复 (本 fork)

- **删除/回收站确认框**：恢复上游统一的 `ProjectPanel::build_removal_prompt`，未保存改动警告重新并入提示正文（此前 fork 的内联旧实现把它放进了副标题，导致行为与上游测试描述不一致）。
- **项目面板重命名**：恢复上游的 `window.is_window_active()` 守卫，切换键盘布局或切到其他应用后重命名/新建输入不再被取消。
- **ACP 工具位置**：跟随上游 `location()` 新语义（未解析位置也返回原始位置、解析结果在第二项），目录跳过逻辑与回归测试同步调整。

### 保留的 fork 特性

- 上下文压缩后的条目回收（`drop_compacted_entries`）、外部智能体的自动压缩触发与三档日志。
- `fs_watcher` 健康检查（`check_watch_health` + `WatchBackend::watched_paths`，watch 丢失自动重扫）。
- 表格与 SQLite/CSV 预览的内联设置面板（挂载在上游 `TableView`）。
- Markdown 标题内 KaTeX 字号与 `pending_heading_scroll` 滚动。
- `third_party/notify` path vendor（含 Windows 溢出修复）、`README.md` 顶部两行提示。

### 汉化 (本 fork)

- 本次合并新增 6 个键（`settings_ui.skills_setup` 的远程主机两键、`sidebar.new_thread.create_worktree_in`、`agent_ui.config_options.agent_options`、`agent_ui.thread_view.open_plan_file` / `cycle_response_speed`）。
- en / zh-CN 双侧键集合一致（各 4060 键），`scripts/i18n/check-missing-keys.py` 0 问题。

### 安装包与发布链路

- **Ubuntu / Debian**：`zed-zh-cn_1.31.0_amd64.deb`，安装到 `/usr/lib/zed`，注册 `/usr/bin/zed`、桌面项与图标；`Depends` 由 `ldd` 实测推导，并对 Ubuntu 24.04 的 t64 改名写备选。另附 `zed-linux-x86_64.tar.gz`。
- **Windows**：`Zed-x86_64.exe`（Inno Setup 安装器）；**macOS**：`Zed-aarch64.dmg`（`macos-26` runner，显式校验 SDK ≥ 26）。
- **版本号**：`1.31.0` 与上游 `main` 一致，高于上游最新正式 release 1.29.3；`release.yml` 校验其不低于上游正式版、与 `Cargo.lock` 一致，且标签名与 `crates/zed/Cargo.toml` 一致。
- `run_tests.yml` 仅手动触发（`workflow_dispatch`），`check_style` 含汉化键完整性检查。

### 验证

- `cargo check -p zed`、`cargo check --workspace --all-targets`、受影响 crate 全量 `clippy --all-targets -- -D warnings`、`cargo fmt --all --check` 通过。
- 测试：acp_thread 317、agent_servers 203、agent_skills 95、agent_ui 594、git_ui 214、project_panel 150、settings_ui 72、tabular_data_preview 23、language_model_selector 8、fs 63，其余 8 个 crate 合计 1630 项，全部通过。

### 已知说明

- 本构建未经代码签名：Windows 可能触发 SmartScreen；macOS 可能被 Gatekeeper 拦截（右键「打开」或执行 `xattr -dr com.apple.quarantine /Applications/Zed.app`）。
- 详细变更列表见下方自动生成的变更记录。