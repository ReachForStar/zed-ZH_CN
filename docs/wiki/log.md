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
