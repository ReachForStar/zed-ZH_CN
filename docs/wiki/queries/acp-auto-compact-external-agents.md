---
title: ACP 面板自动压缩上下文不生效：外部智能体缺客户端触发
type: query
tags: [acp, agent, auto-compact, 上下文压缩]
created: 2026-09-26
updated: 2026-09-28
status: active
---

# ACP 面板自动压缩上下文不生效

## 问题

ai-acp 面板（基于 `AcpThread` 的智能体面板）中，`agent.auto_compact` 设置（默认
`enabled: true, threshold: "90%"`）对外部 ACP 智能体完全不生效：上下文占满也不压缩。

## 根因

面板下有两类连接，压缩机制完全不同：

- **原生 Zed 智能体**（`NativeAgentConnection`）：压缩在 `agent::Thread::run_turn_internal`
  循环内自触发（`perform_compaction_if_needed` → `compaction_message_target_ix`），门槛是
  `auto_compact.enabled` 且 `input_token_capacity() >= 80_000`（`MIN_COMPACTION_CONTEXT_WINDOW`）。
  该路径完好，agent crate 25 个 compaction 测试全部通过。
- **外部 ACP 智能体**（Claude Code / Gemini CLI 等）：ACP 协议的压缩
  （`unstable_session_compaction`）只有 agent→client 的 `CompactionUpdate` /
  `CompactionSummaryChunk` 通知，**不存在 client→agent 的压缩请求**。Zed 端没有任何
  代码消费 `auto_compact` 设置去触发外部智能体压缩——设置被静默忽略。

外部智能体通过 `session/update` 的 `UsageUpdate { used, size }` 上报用量，
`AcpThread.token_usage` 据此更新面板上下文占比条；部分智能体在
`AvailableCommandsUpdate` 里公布 `compact` / `compress` 命令。

## 解法（fork 特性）

在 `AcpThread` 增加客户端自动压缩（`crates/acp_thread/src/acp_thread.rs`
`maybe_trigger_auto_compact`）：

- 触发点：回合正常结束（completion `Ok(r)` 且未取消）、收到 `UsageUpdate`；
  仅在空闲（`running_turn.is_none()`）时执行。
- 门槛：`auto_compact.enabled`；用量达到阈值（Percentage 按 `used/max`、
  TokensUsed 按绝对值、TokensRemaining 按剩余量；`max_tokens == 0` 时只认 TokensUsed）。
- 动作：在 `available_commands` 里找 `compact` / `compress`（忽略大小写），
  用 `send_command`（不回显用户气泡）发送 `/<命令>` 回合。
- 防循环：`auto_compact_armed` 滞回——触发一次后解除武装，直到用量回落到阈值以下
  才重新武装；压缩回合自身的 completion 再检查时因未武装而不会连发。
- 原生排除：`AgentConnection::is_native()`（connection.rs 新增，默认 false；
  `NativeAgentConnection` 覆写为 true），原生智能体继续走自身回合内压缩，避免双重压缩。

测试：`test_auto_compact_triggers_agent_compact_command`（无命令不发、越阈触发、
滞回防连发、回落后重新武装）、`test_auto_compact_skips_native_agent`。

## 涉及模块

- `acp_thread`（AcpThread、AgentConnection trait）
- `agent`（NativeAgentConnection 覆写 is_native）
- 设置来源：`agent_settings::AutoCompactSettings`（assets/settings/default.json `agent.auto_compact`）

## 后续修复：压缩完成后释放被压缩条目（2026-09-28）

自动压缩只收缩模型上下文，**Zed 端 transcript 仍无限驻留**：`push_context_compaction`
只插入/替换摘要条目，从不删除被压缩的旧消息与工具输出（单条可达 MB 级），
长会话内存持续增长（64G 机器实测可到 50G 卡死）。

修复（`crates/acp_thread/src/acp_thread.rs`）：

- `drop_compacted_entries`：压缩状态变为 `Completed` 时（`upsert_context_compaction_update`
  两处调用点），丢弃上一次完成压缩之后的全部条目，摘要条目保留；失败/取消的压缩
  条目只是会话记录标记，不是分段边界。
- 终端清理复用 rewind 范式：仅当终端不被保留条目引用时才 kill 并从 `terminals`
  移除，防止跨条目复用的长驻终端被杀。
- 事件顺序：先 `EntryUpdated` 再 `EntriesRemoved(range)`，UI（conversation_view）
  已有该事件的删除处理。

测试：

- `test_context_compaction_completion_drops_compacted_entries`：失败压缩保留条目、
  完成压缩丢弃区间（0..4）、二次压缩只丢自己的段（1..2）、`EntriesRemoved` 事件；
- `test_context_compaction_updates_preserve_timeline_and_emit_events` 改为两个压缩均
  `Cancelled` 收尾（原 Completed 语义与丢弃行为冲突），保留时间线位置与事件断言；
- acp_thread 全量 167 测试通过、agent_ui compact 测试通过、CI 语义 clippy 通过。

局限：若外部智能体只公布 compact 命令但**不发送** ACP `CompactionUpdate`，
Zed 端无从得知压缩完成、条目不会释放；此类智能体需在 compact 命令回合结束时
由 Zed 侧自行丢弃（未实现，待观察实际智能体行为）。

## 附带修复

同批发现 merge 遗留：上游新增的两处 test-support 后端未实现 fork `WatchBackend`
trait 新增的 `watched_paths`（fs_watcher 健康检查特性），`cargo test` 编译失败：

- `crates/fs/src/fs.rs` `FakeWatchBackend`：按 `registered_paths` 返回 `Recursive` 模式；
- `crates/fs/src/fs_watcher/diagnostics.rs` 测试 `Backend`：无状态，返回空列表。

## 复发预防

- 上游重构 `WatchBackend` 相关代码后，跑一次 `cargo check -p fs --features test-support`
  （普通 `cargo check` 不编译 test-support 代码，漏检）。
- 判断"自动压缩不生效"先分清连接类型：原生看 `Thread` 门槛（模型 max_input_tokens
  是否 >= 80k，参见上游 issue #64256 的 llama.cpp context_window 未保存问题）；
  外部智能体看本页的客户端触发是否命中（智能体须公布 compact/compress 命令并上报 UsageUpdate）。
