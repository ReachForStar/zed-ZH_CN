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
