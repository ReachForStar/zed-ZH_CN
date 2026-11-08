---
title: 新建智能体会话慢：外部智能体二进制首次下发与目录路径误读
type: query
tags: [agent, acp, performance, external-agent, registry]
created: 2026-11-08
updated: 2026-11-08
status: active
---

# 新建智能体会话慢：外部智能体二进制首次下发与目录路径误读

## 问题

新建智能体会话（ACP 外部智能体）时，要等待很久才出现可对话界面。

## 根因

1. **主因：外部智能体运行时首次下发**。ACP registry 智能体（此处为 `opencode-v2`）在 `LocalRegistryArchiveAgent::get_command` 里检查
   `~/.local/share/zed/external_agents/registry/<id>/v_<版本>_<哈希>/` 是否存在；不存在就先下载归档、解包、写盘，再返回启动命令。
   日志证据（`~/.local/share/zed/logs/Zed.log`）：
   - `10:25:32` `[github_releases] downloading https://opencode.ai/files/bin/2.2.6/opencode-linux-x64.tar.gz into ".../opencode-v2/v_2.2.6_e3b853e667eb0a54_b5fbe8e5346071a9"`
   - 二进制落盘时间 `10:27:09`，`opencode` 文件 258824368 字节（约 247 MiB），耗时约 97 秒
   - 下载完成后的连接在 `10:27:09` 立刻建立；应用重启后 `10:28:32` 的新连接 1 秒内完成

   结论：慢在**新版本首次下发**，不是每次建会话都慢；版本未变时走缓存，秒开。UI 上此时显示 “Installing 2.2.6…”。
2. **次要问题：目录被当成文件读取**。同一日志另有 2979 条
   `ERROR [crates/acp_thread/src/acp_thread.rs:1897] reading bytes of the file "/data/f/项目/Zed"`。
   来源是 `AcpThread::resolve_location`：智能体上报的工具调用位置指向工作区根目录（或它刚创建的文件夹）时，
   代码直接取出该路径下的 buffer，而该路径是目录，`worktree::decode_file_text_to_rope` 读字节时返回 EISDIR；
   每个上报的目录位置都会失败一次并记一条错误日志，还可能在项目外路径上临时创建工作树。
   上游代码同样如此，不是合并引入。

## 解法

- 主因无需改代码：等下发完成即可；版本变化后第一次开会话慢属预期。如需避免，可预置
  `~/.local/share/zed/external_agents/registry/<id>/v_<版本>_<哈希>/` 或改用已安装的自定义智能体。
- 目录误读已在 `AcpThread::resolve_location` 修复：先用 `project.fs().metadata()` 判断路径类型，
  只对绝对路径判断，目录直接返回 `None`，不再打开 buffer、也不为项目外目录创建工作树。
  回归测试 `acp_thread::tests::test_tool_locations_skip_directories` 用 `FakeFs::read_call_count()`
  断言「4 个位置只有 1 个文件被读取」；为便于断言，`FakeFsState` 新增 `read_call_count`（在
  `load_internal` 计数，`FakeFs::read_call_count()` 读取）。上游 `upstream/main` 尚无该判断，
  后续合并需保留，见 [汉化 fork 同步 upstream 的合并模式与特性保留清单](../concepts/fork-upstream-sync.md)。

## 涉及模块

- `crates/project/src/agent_server_store.rs` — `LocalRegistryArchiveAgent::get_command`（下载与解包）
- `crates/project/src/agent_registry_store.rs` — registry 刷新（`refresh_if_stale`，节流 1 小时）与 `build_registry_agents`
- `crates/agent_servers/src/custom.rs` — 建连前调用 `refresh_if_stale`
- `crates/acp_thread/src/acp_thread.rs` — `resolve_location` / `resolve_locations`
- `crates/worktree/src/worktree.rs` — `decode_file_text_to_rope`（错误文案来源）

## 复发预防

- 判断“会话启动慢”先看 `Zed.log` 是否有 `[github_releases] downloading` 与 `external_agents/registry` 相关行；
  有则是下发，不是 Zed 逻辑问题。
- 排除下发因素后，再查 hang 检测日志（`rank too long` 任务）与 `worktree diagnostics`（条目数过大）。
