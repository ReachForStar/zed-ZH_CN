# Wiki 操作日志

> 只追加、不修改历史。可用 `grep "^## \[" docs/wiki/log.md | tail -5` 看最近记录。
> 前缀格式：`## [YYYY-MM-DD] <init|feat|fix|query|lint|chore> | <简述>`

## [2026-09-25] init | wiki-memory hook 自动创建知识库骨架

## [2026-09-26] query | 沉淀 msvc_spectre_libs 构建失败排查（缺 14.51 Spectre 缓解库，装组件解决）

## [2026-09-26] feat | 沉淀汉化 fork 同步 upstream 合并模式与特性保留清单；同步 upstream/main（2 提交，language_model 重构）并保留 fork 特性

## [2026-09-26] fix | 修复 ACP 面板外部智能体自动压缩上下文不生效（客户端触发 /compact，滞回防循环，原生排除）；补 merge 遗留 watched_paths（fs.rs、fs_watcher/diagnostics.rs）

## [2026-09-26] feat | 发布 v1.21.0：版本号对齐上游最新发布（改 Cargo.toml/Cargo.lock），修复 run_tests typos 配置路径（上游移入 .config/）；沉淀 fork-release-publishing 概念页

## [2026-09-26] query | run_tests 验证：代码风格检查（typos）通过，确认 .config/typos.toml 路径修复生效；workspace.ps1 的 cargo metadata --format-version 警告按用户决定留到下个版本处理

## [2026-09-26] fix | 恢复 extensions_ui 上游代际（版本选择器调用链、右键菜单、repository_icon/context_menu builder）并套回 t! 汉化；修复 Clippy dead_code 失败；extension_suggestions 2 个测试在 Windows 本地失败为既有问题（HEAD 复测相同）

## [2026-09-26] query | 沉淀 run_tests Clippy 三层失败链（typos 路径/dead_code/redundant_clone）与本地复现 CI clippy 语义方法；acp_thread 冗余 clone 已修复推送

## [2026-09-26] query | 沉淀 git bash 调用 pwsh 的两类失败（chcp 包装退出码 1、裸反斜杠路径退出码 64）与 Write-Host 管道输出 GBK 的编码修法；同步修正 git-proxy skill 的调用方式

## [2026-09-26] fix | 修复 zed.rs useless_format（String 字段重复 format!），全量本地 clippy 通过；run_tests 36212676836 全绿；query 页扩为四层失败链并补 webrtc 下载代理坑

## [2026-09-28] query | 沉淀内存占用异常排查页：审计确认 undo 历史（text.rs History.operations 只增不删）、Project buffer 驻留、终端 PTY 事件无界通道、ACP 线程消息压缩前驻留为长期增长点；LSP 日志/终端回滚有界排除；fork 的 notify vendor + fs_watcher 健康检查无泄漏但叠加放大为重扫风暴；给出 ETW 堆追踪实测路径，50G 构成待实测

## [2026-09-28] fix | 修复 ACP 长会话内存驻留：压缩状态 Completed 时丢弃被压缩条目（drop_compacted_entries，复用 rewind 的终端清理范式，仅杀不被保留条目引用的终端）；失败/取消压缩条目不是分段边界；acp_thread 全量 167 测试、agent_ui compact 测试、CI 语义 clippy 全过；局限：外部智能体不发送 CompactionUpdate 时 Zed 侧无从释放，待观察


## [2026-09-28] query | 补记 webrtc-sys 构建脚本行为：download_webrtc 每次删除 OUT_DIR 旧解压后整包重下，无本地缓存；LK_CUSTOM_WEBRTC 指向含 include/lib/webrtc.ninja 的目录可跳过下载；os error 10060 与 tls handshake eof 同为代理上游断开，用 curl -x 访 google 探针区分代理死亡与仅 github 阻断

## [2026-09-28] query | 修正 git-proxy 页与脚本现状不符——grep 实测 scripts/git-proxy.ps1 无 OutputEncoding 设置行，改记 iconv 补救为现行方案；SKILL.md 已补探针/瞬断/非 git 进程边界说明

## [2026-10-16] feat | 合并 upstream/main（953ac75529，297 提交）并保留 fork 汉化与特性：冲突消解取上游新 API/新结构与 acp_v1 拆分、恢复 image_resolver 两参数 API、tabular_data_preview 面板迁 TableView、补回 git_panel.starts_open 等三项丢失设置；本地全量 clippy/fmt/typos/check-todos/check-keymaps 通过，合并提交 80851bcbbc 已推送 fork main；沉淀 upstream-merge-2026-10 查询页与 fork-upstream-sync 要点

## [2026-10-16] feat | 合并后汉化补齐：分 8 批处理 30+ crate 界面文案（workspace/ui/editor/language_models/edit_prediction_ui/agent_ui/copilot_ui/language_tools/git_ui/debugger_ui/settings_ui/onboarding/recent_projects/extensions_ui/dev_container/vim/zed/project_panel/picker/outline_panel/collab_ui/command_palette/miniprofiler_ui/acp_tools/auto_update/remote_connection/call_hierarchy/lsp_command_selector/git_ui_core/markdown_preview/agent_servers），en 与 zh-CN 双侧补键；判定整体跳过 project/git_hosting_providers/which_key/languages/gpui 平台 crate 与 component preview、dev-tools 面板；批量脚本落 `scripts/i18n/`，沉淀 concepts/i18n-localization-workflow 页
## [2026-10-16] fix | 修复汉化引入的 clippy redundant_clone（edit_prediction_button.rs 命名参数 t! 后又 to_string），本地全量 clippy 过、批次 6 起 CI 转 success
## [2026-10-16] fix | 合并后逐 hunk 审计发现静默丢失：thread_view.rs 的 sandbox_confusable_findings 该扫 requested 与 canonical_or_requested 两条路径（上游 953ac75529 已改），合并结果退回只扫 requested；恢复上游实现并推送 e26a09420f

## [2026-10-16] chore | run_tests 改为手动触发（workflow_dispatch），去掉 push/PR 自动运行，避免每次文案提交拉起全量 clippy；check_style 新增汉化键完整性检查（scripts/i18n/check-missing-keys.py，比对代码静态 t! 引用与 en/zh-CN 键集合）

## [2026-10-16] feat | 二次合并 upstream/main（28d0a3547f，8 提交 / 39 文件，零冲突）：合并提交 30e413d270，逐个复核 39 文件 t! 计数无下降；补译上游新增 2 条可恢复消息状态文案，并补齐此前漏译的整块「可恢复消息」界面（状态文案 / Copy、Restore、Discard 按钮 / tooltip / 附件占位符，共 14 键，提交 b3fdd26c6d）；fmt --check 与全量 clippy 通过，CI 手动触发 run 38851778563

## [2026-10-23] feat | 三次合并 upstream/main（91d3b962ae3，120 提交 / 360 文件）：17 处冲突消解（workflow 取删除、lsp-types 跟进上游新 rev、版本号取 fork 侧 1.24.2）；markdown 按上游恢复 pending 滚动清理、子智能体卡片文案迁到 subagent_card.rs、git 面板确认框文案带 %{changes} 插值、fs 健康检查按新 coverage 模型重写并改用 Weak 持有 watcher；补上游新增 11 处界面文案汉化（en/zh-CN 双侧）；fmt/clippy/typos/check-todos/check-keymaps/check-missing-keys 全过，fs 单测 33 项全过

## [2026-10-23] feat | 四次合并 upstream/main（604a0ff023，2 提交 / 15 文件，零冲突）：acp_thread 新增 AgentThreadEntry::SessionMessage（父子会话间消息 transcript 行）与 agent_ui 的 render_session_message/session_participant_label；open_ai 流式错误类型化（LanguageModelCompletionError 取代 anyhow 装箱）各 provider 返回类型跟进；补上游新增 6 处界面文案汉化（session_message_to/from、session_message_between_sessions、open_session、unavailable_session、main_agent，en/zh-CN 双侧），顺带接上 subagent_label 的漏译键；fmt/clippy/typos/check-todos/check-keymaps/check-missing-keys 全过；既有失败（非本次引入）：agent_servers 的 test_load_session_replays_notifications_sent_before_response 与 v2_compaction_sdk_chunks_snapshots_and_historical_targeting，回退合并前 HEAD 复现，属二次合并带入的上游测试与 fork drop_compacted_entries 内存回收特性的语义分歧，待用户决定取舍

## [2026-10-24] fix | 按「保留内存回收」消除 agent_servers 两处既有失败：v2_compaction_sdk_chunks_snapshots_and_historical_targeting 暴露特性缺陷——丢弃区间会跨过仍在 InProgress 的压缩行、后续 chunk 把被丢的行重建成新行导致历史错乱，修正为 InProgress 也作为屏障；test_load_session_replays_notifications_sent_before_response 按 fork 语义改期望（回放 Completed 压缩与实时同一路径，最终只剩压缩行）；acp_thread 304 项、agent_servers 181 项全绿，fmt/check-missing-keys/typos 通过；沉淀 acp-auto-compact 页的语义分歧小节

## [2026-10-24] chore | 把 CRLF 行尾还原脚本从 tmp 移入 scripts/normalize-crlf.py（tmp 会被当临时文件清理）：无参数处理 git diff HEAD 的 .rs，给路径则只处理指定文件，跳过二进制与不存在路径、可重复执行；同步更新 fork-upstream-sync / i18n-localization-workflow / upstream-merge-2026-10 三页里的命令；实测全 LF 文件（2267 行）一次还原为 CRLF、二次运行幂等，提交 9b6397259e 已推送

## [2026-10-30] feat | 五次合并 upstream/main（422b53cff8，98 提交 / 314 文件）：11 处冲突消解（workflow 取删除、livekit rev 跟进上游、notify 保留第三方便携版、Cargo.lock 取上游并以 cargo 重裁、zed 版本留 1.25.3）；上游删 http_client::github 后 auto_update 改用 github_releases crate、ACP SDK 3.6 的 AvailableCommand::name() 访问器跟进、上游整体移除 checkpoints 后删 Restore Checkpoint 按钮与两个死键、settings_ui 新增 Auto Fetch 两项改 t! + SharedString、git 图新增 Refs 列（column_refs）；补回被冲突块吞掉的 fullscreen_entry、清掉两处重复闭合分隔符；补汉化 push_rejected / column_refs / subagent_status_unknown / submission_detail_* 共 7 键（en/zh-CN 双侧）；check -p zed、全量 clippy（469 crate 0 警告）、fmt、check-missing-keys（0 问题）、check-todos、check-keymaps 通过，acp_thread 308 / agent_ui 583 / agent_servers 187 / settings_ui 76 等测试通过；已知本地失败：上游新测试 test_history_tab_focus_border_only_for_keyboard_focus（与 fork 改动无关，待上游 CI 核对）；本机环境补丁：webrtc-sys 需 CXXFLAGS=-fpermissive、测试链接用 tmp 符号链接补 libX11-xcb.so；合并提交 1900b573e8 已推送 fork main

## [2026-10-30] chore | 补提交上次遗留的发版准备（`110f5c907a`）：`crates/zed/Cargo.toml` 与 `Cargo.lock` 版本号 1.24.2 → 1.25.3、重写 `RELEASE_NOTES.md`、`README` 分支策略行的当前版本、`script/lib/workspace.ps1` 的 `cargo metadata --format-version=1`、`fork-release-publishing` 页的发版记录；同步把该页「2026-10-24 发布 v1.25.3」更正为「准备但未打标签发布」（GitHub Releases 最新仍为 v1.24.2）

## [2026-10-30] feat | release CI 改为按系统给原生安装包并让版本号跟随上游：新增 script/bundle-linux-deb（tar.gz → .deb，装 /usr/lib/zed + /usr/bin/zed 软链 + 桌面项与 hicolor 图标 + licenses，Depends 由 ldd 推导并给 Ubuntu 24.04 的 t64 改名写备选，包名 zed-zh-cn 且 Conflicts/Replaces zed）；release.yml 加 verify_version job（scripts/sync-upstream-version.py --check）与 deb 构建上传、发布说明按系统写安装方式、workflow_dispatch 增加 dry_run 只构建不发布；版本号同步上游 main 的 1.28.0（Cargo.toml/Cargo.lock/README/RELEASE_NOTES 四处一致），RELEASE_NOTES 按 v1.28.0 重写、上游对比改为 28d0a3547f...422b53cff8；本地用合成 app 目录与 tar.gz 两种输入验证 dpkg-deb 产物（control/软链/桌面项/图标/postinst 语法）与 fakeroot dpkg --no-act，版本脚本写入/幂等/--check 均通过，cargo check -p zed 与 fmt、check-missing-keys 通过；提交 78356823b6、d54909a734 已推送

## [2026-10-30] chore | 发布 v1.27.0：版本号定 1.27.0（Cargo.toml/Cargo.lock/README/RELEASE_NOTES 四处一致），sync-upstream-version.py 增加 --check-not-behind（版本不低于上游最新正式 release、且与 Cargo.lock 一致）、--print（供 CI 比对标签）；release.yml 的 verify_version 改为三重校验（不落后上游 stable、lock 一致、标签 v{版本} 与 Cargo.toml 一致），发布说明首行改为「社区简体中文翻译构建，版本 v{版本}」不再声称对应某个上游正式版本；在提交 077d25fa39 上打附注标签 v1.27.0 并推送，触发 release run 41062882626，verify_version 已通过（1.27.0 ≥ 上游正式版 1.26.2），三平台构建进行中；记录上游版本事实：正式发布线最新 v1.26.2、preview 线 v1.27.0-pre、main 的 Cargo.toml 为 1.28.0
