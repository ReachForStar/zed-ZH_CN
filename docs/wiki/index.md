---
type: index
updated: 2026-09-26
---

# Wiki 索引

> 人工/Agent 显式查阅用的完整目录；会话中的导航由 wiki-memory hook 注入的目录树提供。
> 格式：`[页面标题](相对路径) — 一行摘要`。

## 实体 entities

## 概念 concepts

- [汉化 fork 同步 upstream 的合并模式与特性保留清单](concepts/fork-upstream-sync.md) — fetch 走 HTTPS、diff 判读陷阱、i18n×上游重构冲突解法、特性保留清单
- [fork 发布 release 的流程与版本号策略](concepts/fork-release-publishing.md) — v* 标签触发 release.yml、版本号取上游已发布 release、Cargo.toml/Cargo.lock 同步改法

## 源总结 sources

## 决策 decisions

## 查询沉淀 queries

- [msvc_spectre_libs 构建失败：缺 Spectre 缓解库](queries/msvc-spectre-libs-build-failure.md) — pet 硬编码 error feature + cc 选最新工具集缺 spectre 库，装 14.51 Spectre 组件解决
- [ACP 面板自动压缩上下文不生效](queries/acp-auto-compact-external-agents.md) — 外部 ACP 智能体无 client→agent 压缩请求，auto_compact 被静默忽略；AcpThread 客户端触发智能体 /compact 命令修复
