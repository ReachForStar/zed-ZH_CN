---
title: 2026-11-26 合并 upstream/main（1.32 开发线，11 提交）：单冲突消解与 WatchPaths 结构适配
type: query
tags: [upstream-merge, conflict, fork]
created: 2026-11-26
updated: 2026-11-26
status: active
---

# 2026-11-26 合并 upstream/main

## 范围

- 上游基线：`upstream/main` 的 `a3ce8d5e88`，距上次合并（`cd585371e8`）新增 11 个提交、26 个文件（+1672 / −161）。
- 上游版本号：`552644414c` 把 `crates/zed/Cargo.toml` 与 `Cargo.lock` 升到 `1.32.0`（上游已从 main 切出 1.31 发布分支）。
- fork 版本号：保持 `1.31.0`（用户未指定新版本号，按 [fork 发布 release 的流程与版本号策略](../concepts/fork-release-publishing.md) 不自行改动、不打 tag、不触发发布）。
- 合并落点：本仓库 `main`，`git merge --no-edit upstream/main`。
- 合并提交：`de2370d811`（父 `23f4ffd22a` + `a3ce8d5e88`），已推送 fork `main`（远端 `git ls-remote` 核对等于本地 HEAD）。
- 上游要点：`search` 支持在缓冲区搜索打开时从编辑器替换（`a3ce8d5e88`）、`fs` 的 `watcher_ids_covering` 祖先匹配提速（`aeabc0823d`）、`trusted_worktrees` 路径比较在 Windows 上区分大小写（`dc1a4bfd05`）、`project` 在 buffer 释放时丢弃其 LSP 数据（`32062546d5`）、`file_finder` 匹配目录时跳过当前文件、`terminal_view` 用空格绘制制表单元格、`dev_container` 尊重只读挂载、`google_ai` 缺 `citation_sources` 时给默认值、vi 可视行选择整行搜索、`crash-handler` 子进程挂起修复。

## 冲突与判定

唯一未合并路径：`crates/fs/src/fs_watcher.rs`，一行 `std::collections` 导入两侧冲突：

- fork 侧加了 `HashSet`（`check_watch_health` 用 `os_keys` 记录后端监听路径的精确与折叠两种键）；
- 上游侧加了 `BTreeMap` 与 `btree_map::Entry`（`WatchPaths` 新增 `exact_depths` / `folded_depths` 按深度计数）。

判定：两侧都保留，合并为 `collections::{BTreeMap, BTreeSet, HashMap, HashSet, btree_map::Entry}`。

其余 8 个与 fork 改动重叠的文件（`Cargo.lock`、`file_finder.rs`、`fs.rs`、`fs_watcher/diagnostics.rs`、`buffer_search.rs`、`dock.rs`、`workspace.rs`、`zed/Cargo.toml`）全部自动合并，无需人工介入。

## 编译失败链

1. `fs` E0609 `no field 0 on type WatchPaths`：上游把 `WatchPaths` 从 newtype（`.0` 是 `FxHashMap<WatchKey, PathRegistrationState>`）改成带字段的结构（`entries` / `exact_depths` / `folded_depths`），fork 的 `check_watch_health` 里 `state.paths.0.iter()` 失效 → 改 `state.paths.entries.iter()`。这是 fork 特性直读上游内部状态的典型失效，与 2026-09 那次 `PathRegistrationState.has_os_watcher` → `coverage` 的重构同类。
2. `crates/zed/Cargo.toml` 版本行被上游自动合并成 `1.32.0`（`version = "1.32.0"` 与 fork 的 `1.31.0` 不在相邻行，未产生冲突）→ 手工改回，随后 `cargo metadata` 重裁 `Cargo.lock` 的 `zed` 条目。

## 汉化

- `scripts/i18n/check-missing-keys.py`：代码静态引用 4003 键、en / zh-CN 各 4065 键、0 问题。
- 本轮上游改动无新增可见界面文案：对 `search`、`file_finder`、`terminal_view`、`editor`、`workspace`、`dev_container` 的新增行按 `Label::new("…")` / `Button::new(id, "…")` / `Tooltip::text("…")` / `menu.entry(...)` 检索，唯一命中是 `dev_container` 的 `log::warn!`，不属界面文案。`buffer_search.rs` 新增字面量全是测试用例数据（`"aaa"`、`"a a a b"` 等）。

## 验证

- `python3 scripts/check-upstream-added.py cd585371e8 upstream/main`：仅 1 个文件报缺失，即上面那行导入（已并入两侧），无真实丢失。
- `cargo check --workspace --all-targets`：通过。
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`：通过（2m25s）。
- `cargo fmt --all --check`：通过（提交前 `python3 scripts/normalize-crlf.py` 还原工作区 CRLF）。
- `script/check-todos`、`script/check-keymaps`：通过（CRLF 工作区按惯例用 `git show :script/<名> > tmp/<名>.sh` 取 LF 版本执行）。
- 测试两批 6161 项全通过（`LC_ALL=C LANG=C`）：
  - 批次一 2241 项：acp_thread 321、agent_servers 202、agent_ui 611、auto_update 20、dev_container 134、file_finder 94、fs 37 + 集成 29、git_ui 223、github_releases 10、project_panel 150、search 100、workspace 310。
  - 批次二 3920 项：editor 1236、gpui 589 + 3 + 文档测试 89、language_models 157、markdown 207、project 114 + 集成 444、settings_ui 72、sidebar 171、terminal_view 102、vim 588、zed 148。
  - 未跑：`collab`（需 Postgres）、`sandbox*`（需特权与 Windows 环境），仅由 `--all-targets` 编译覆盖。

## fork 特性核对

- `crates/fs`：`check_watch_health`、`WatchBackend::watched_paths`（`fs_watcher.rs` 两个实现 + `fs.rs` 两个实现）、`Coverage` 模型判定、健康检查线程用 `Weak<OsWatcher>`（`watcher_retains_scheduler_until_teardown` 用例）全部在位。
- `acp_thread`：`resolve_location` 目录跳过、`drop_compacted_entries`、`maybe_trigger_auto_compact` 仍在（321 项测试通过）。
- `crates/auto_update`：自建更新通道 `GITHUB_RELEASE_REPO`（`ReachForStar/zed-ZH_CN`）、`release_tag`、`fork_release_notes` + `select_release_notes`、`release_notes_url` 指 GitHub Release 页面均在（20 项测试通过）。
- `tabular_data_preview`、`settings_ui` 的 `SharedString` 设置项、locale en / zh-CN 各 70 个分片成对、`.github/workflows` 仅 `release.yml` + `run_tests.yml`、`third_party/notify` path vendor、`README.md` 顶部两行全部完好。
- 未运行 `cargo xtask workflows`。

## 复发预防

- 上游把「元组结构体」改成「带字段结构体」时（本轮 `WatchPaths`），fork 侧直读内部字段的代码只在编译期暴露；合并后先对所有「读上游私有字段」的 fork 特性做一次 `grep`（`\.paths`、`\.0\b` 之类），再跑 `cargo check`。
- 版本行不在冲突位置时 git 会静默采用上游值（本轮 `crates/zed/Cargo.toml`），合并后必须显式核对 `grep -m1 '^version' crates/zed/Cargo.toml` 与本仓库既定版本策略是否一致，不能只依赖冲突提示。
- 本轮 26 个文件的改动范围里 8 个与 fork 改动重叠却零冲突，说明「有重叠文件」不等于「有冲突」；核对仍以 `scripts/check-upstream-added.py` 的集合比对为准。

## 关联页面

- [汉化 fork 同步 upstream 的合并模式与特性保留清单](../concepts/fork-upstream-sync.md)
- [2026-11-25 合并 upstream/main（1.31.0 线，68 提交）](upstream-merge-2026-11-25.md) — 上次合并基线 `cd585371e8`，本轮的 `check-upstream-added.py` 与 `LC_ALL=C` 测试环境约定由该轮引入。
- [fork 发布 release 的流程与版本号策略](../concepts/fork-release-publishing.md) — 本轮按「版本号由用户决定」规则停在 `1.31.0`。
