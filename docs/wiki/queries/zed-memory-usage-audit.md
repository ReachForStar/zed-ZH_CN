---
title: 内存占用异常排查：长时间使用后单进程可占 50G
type: query
tags: [内存, 泄漏, undo, watcher, ETW]
created: 2026-09-28
updated: 2026-09-28
status: active
---

# 内存占用异常排查：长时间使用后单进程可占 50G

## 问题

Windows 上 Zed（fork 构建）长时间使用后内存持续增长，64G 机器上单进程可占约 50G（总占用 99%）。要求审计代码中是否有异常驻留。

## 排查结论（按证据分级）

### 已确认无界、长期增长必然发生的结构

1. **编辑 undo 历史无界**：`crates/text/src/text.rs` 的 `History`（text.rs:153）中
   `operations: TreeMap<clock::Lamport, Operation>` 只有 `insert`（text.rs:235），
   全仓无删除路径；`undo_stack` 每个事务同样永久保留，含每次编辑的文本
   fragment。大文件反复格式化、批量替换会持续累积。
2. **buffer 无 LRU 卸载**：`Project` 持有 `buffers: Vec<Entity<Buffer>>`
   （crates/project/src/project.rs:276），打开过的文件（文本 + 语法树 + 诊断 +
   高亮缓存）整个会话驻留。
3. **Agent/ACP 线程消息**：压缩发生前，所有消息与工具输出（终端输出、
   文件内容，单条可达 MB 级）全部驻留内存。**用户确认此为主因**（ACP 长会话
   后机器卡死）。2026-09-28 已修复：压缩状态变为 Completed 时丢弃被压缩条目，
   见 [ACP 面板自动压缩上下文不生效](acp-auto-compact-external-agents.md)。
4. **终端 PTY 事件通道无界**：`crates/terminal/src/terminal.rs` 的
   `events_rx: UnboundedReceiver<PtyEvent>`（terminal.rs:979/1018/1216，均
   `unbounded()`）。回滚显示有 100_000 行上限
   （terminal.rs:891 `MAX_SCROLL_HISTORY_LINES`），但积压在通道中尚未消费的
   事件不受限；输出速率超过消费（UI 卡顿或重扫风暴期间）时会堆积。

### 已确认有界（排除）

- LSP 日志：`MAX_STORED_LOG_ENTRIES = 2000`（crates/project/src/lsp_store/log_store.rs:23）
- 终端回滚：`MAX_SCROLL_HISTORY_LINES = 100_000`
- 行包装缓存：按 (字体, 字号) 维度创建，维度有限

### fork 特有改动审计（无泄漏，但有放大效应）

- **vendored notify**（third_party/notify）：Windows 溢出分支改为重读 + 上报
  `Flag::Rescan`，本身不驻留内存；但高频溢出 = 高频全量重扫。
- **fs_watcher 健康检查**（提交 39b2085b39，crates/fs/src/fs_watcher.rs）：
  每 30s 比对 OS 实际监听与注册表，对孤儿注册重建 watch 并合成 rescan 事件，
  触发**全部** worktree 全量重扫。两者叠加：大仓库持续构建时形成 rescan 风暴
  （CPU 占用 + 文件树冻结），扫描请求队列无界
  （crates/worktree/src/worktree.rs:595 `unbounded()`），但扫描内存为瞬态。
- **sqlite_viewer**（提交 4dfbe364ea）：分页查询、无大缓存；打开超大 .db 时
  按需驻留。
- **katex**：仅 markdown 预览渲染，维度有限。

## 定位手段（实测路径，待执行）

- Zed 内置 ETW 堆追踪：命令面板 `Record ETW Trace With Heap Tracing`
  （需管理员）。WPR 采集 CPU.Verbose.Memory / GPU.Light.Memory /
  DiskIO / FileIO + 自定义 ZedHeap 堆快照，WPA 打开 .etl 可看分配栈。
  对应 CLI 隐藏参数：`--record-etw-trace --etw-zed-pid <PID> --etw-socket <path>`
  （crates/zed/src/main.rs:1794-1807；crates/etw_tracing/etw_tracing.rs:448）。
- 粗定位：任务管理器看提交内存曲线；Process Explorer / VMMap 看堆组成。

## 待确认

- ACP 消息驻留（主因）已修复；undo 历史无界、buffer 无卸载、PTY 通道无界
  仍为长会话增长点，是否需要进一步处理待定。
- 外部智能体若不发送 ACP `CompactionUpdate`（只公布 compact 命令），
  Zed 端仍无从释放条目，见 ACP 压缩页「后续修复」的局限说明。
- 50G 的剩余构成可用 ETW 堆快照（命令面板 `Record ETW Trace With Heap Tracing`，
  需管理员）实测确认。

## 涉及模块

- crates/text（undo 历史）
- crates/project（buffer 驻留、LSP 日志、worktree 扫描）
- crates/terminal（PTY 事件通道）
- crates/acp_thread、crates/agent_ui（线程消息）
- crates/fs（fork 补丁：健康检查）、third_party/notify（fork vendor）

## 复发预防

- 拿到 ETW 堆快照后回写本页，把"待确认"收敛为确定根因。
- 关注上游 zed-industries/zed 的内存相关修复；fork 落后上游时优先同步。
