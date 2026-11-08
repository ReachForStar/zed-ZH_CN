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
   来源是 `AcpThread::resolve_location`：智能体上报的工具调用位置指向工作区根目录时，代码走
   `Project::open_local_buffer`，而该路径是目录，`worktree::decode_file_text_to_rope` 读字节时返回 EISDIR。
   属于上游既有行为（`git show upstream/main:crates/acp_thread/src/acp_thread.rs` 与本地一致），不是本次合并引入；
   影响是刷屏日志与无效 buffer 打开，单次开销小。

## 解法

- 主因无需改代码：等下发完成即可；版本变化后第一次开会话慢属预期。如需避免，可预置
  `~/.local/share/zed/external_agents/registry/<id>/v_<版本>_<哈希>/` 或改用已安装的自定义智能体。
- 目录误读如需消除，可在 `resolve_location` 打开前判断路径类型（`fs.metadata().is_dir()`）并跳过；
  当前未改，避免为日志噪声扩大上游代码改动面。

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
