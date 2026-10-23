---
type: index
updated: 2026-10-16
---

# Wiki 索引

> 人工/Agent 显式查阅用的完整目录；会话中的导航由 wiki-memory hook 注入的目录树提供。
> 格式：`[页面标题](相对路径) — 一行摘要`。

## 实体 entities

## 概念 concepts

- [汉化 fork 同步 upstream 的合并模式与特性保留清单](concepts/fork-upstream-sync.md) — fetch 走 HTTPS、diff 判读陷阱、i18n×上游重构冲突解法、内部状态模型重构×fork 特性读旧字段、文案随模块迁移、特性保留清单
- [汉化口径与批量替换工作流](concepts/i18n-localization-workflow.md) — 汉化/不汉化口径、mkplan+apply 批量流程、t! 返回类型适配、全量 clippy 门禁、未覆盖项
- [fork 发布 release 的流程与版本号策略](concepts/fork-release-publishing.md) — v* 标签触发 release.yml、版本号取上游已发布 release、Cargo.toml/Cargo.lock 同步改法

## 源总结 sources

## 决策 decisions

## 查询沉淀 queries

- [msvc_spectre_libs 构建失败：缺 Spectre 缓解库](queries/msvc-spectre-libs-build-failure.md) — pet 硬编码 error feature + cc 选最新工具集缺 spectre 库，装 14.51 Spectre 组件解决
- [ACP 面板自动压缩上下文不生效](queries/acp-auto-compact-external-agents.md) — 外部 ACP 智能体无 client→agent 压缩请求，auto_compact 被静默忽略；AcpThread 客户端触发智能体 /compact 命令修复；压缩完成后丢弃被压缩条目修复内存驻留
- [run_tests Clippy 失败链](queries/run-tests-clippy-failure-chain.md) — typos 路径→dead_code→redundant_clone→useless_format 四层串行根因；本地全量 clippy 复现、webrtc 下载代理坑与无缓存行为
- [git bash 调用 pwsh 脚本的两类失败](queries/git-bash-pwsh-invocation-pitfalls.md) — `chcp` 包装与裸反斜杠路径分别报退出码 1/64；改用正斜杠引号路径直调 pwsh，脚本内设 OutputEncoding 解决 Write-Host 的 GBK 输出
- [内存占用异常排查](queries/zed-memory-usage-audit.md) — 长时间使用单进程占 50G 的驻留结构审计：ACP 消息驻留（已修复）、undo 历史无界、buffer 无卸载、PTY 通道无界；ETW 堆追踪待实测
- [2026-10 合并上游 main：冲突消解决策与编译失败链](queries/upstream-merge-2026-10.md) — 297 提交合并的取舍清单、六轮编译失败链（image_resolver 两参数、SharedString 补 `.into()`、`t!` 分支类型、redundant_clone）、历史丢失检测与本地校验清单；附二次合并（`28d0a3547f`，零冲突）与三次合并（`91d3b962ae3`，17 处冲突）记录
