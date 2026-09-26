---
title: run_tests Clippy 失败链：四层根因与本地复现方法
type: query
tags: [ci, clippy, run_tests, fork, workflow]
created: 2026-09-26
updated: 2026-09-26
sources: []
status: active
---

# run_tests Clippy 失败链（v1.21.0 发布期）

## 问题

v1.21.0 发布后 main 分支 run_tests 连续四轮失败，每轮都在不同 job：

1. **代码风格检查（typos）**：`could not read config at ./typos.toml` — 上游提交把根配置移入 `.config/`（typos.toml/lychee.toml/livekit.yaml），fork 手维护的 `.github/workflows/run_tests.yml` 仍指向旧路径。修复：`config: .config/typos.toml`（4c9bff177e）。
2. **Clippy dead_code（extensions_ui）**：`ExtensionVersionSelector is never constructed` 等 4 条。根因是更早一次合并的静默功能丢失（见 [fork-upstream-sync](../concepts/fork-upstream-sync.md)）。修复：恢复上游代际并套回 t! 汉化（d1b9a9c735）。
3. **Clippy redundant_clone（acp_thread）**：fork 自写的 auto-compact 测试里 `request.prompt.clone()` 冗余。此前从未暴露，因为前两个阶段先失败、clippy 阶段刚轮到跑。修复：删 clone（b070c01fe8）。
4. **Clippy useless_format（zed）**：`zed.rs:1964` 的 `error = format!("{err}")`——`MigrationStatus::Failed { error }` 字段本就是 `String`。同文件另两处 `format!("{error}")`（anyhow::Error 类型）**未被报**：该 lint 实际只在 String 参数上触发，逐处扫 grep 结果意义不大，以 CI/本地 clippy 为准。修复：`err.to_string()`（55487ef02c）。

## 根因要点

- CI clippy 命令是 `cargo clippy --workspace --all-targets -- --deny warnings`（run_tests.yml:111），`--all-targets` 覆盖 tests 代码；workspace 根 Cargo.toml `[workspace.lints]` 把 redundant_clone 等定为 warn，`-D warnings` 后变 fatal。
- 本地只跑 `cargo check` 或部分 crate 的默认 clippy **检测不到**这类失败：级别与目标范围都和 CI 不一致。

## 解法（本地复现 CI 语义）

改动涉及某 crate 时，推送前用 CI 同款参数只跑受影响 crate，成本远低于全 workspace：

```bash
cargo clippy -p <crate> --all-targets -- --deny warnings
```

两个本地环境坑（2026-09-26 实测）：

- **webrtc-sys build script 要下 GitHub release**：`livekit_client` 链编译时 build.rs 用 reqwest 直连下载 `webrtc-win-x64-release.zip`，本机直连不通会报 `connection closed before message completed` / `tls handshake eof`；给 cargo 注入 `HTTPS_PROXY=http://127.0.0.1:7890`（git 全局代理不影响 cargo）即可通过。`--workspace` 覆盖全部成员（`default-members=["crates/zed"]` 只影响裸 `cargo build` 的默认目标），一轮全量本地通过即可放心推送。

## 复发预防

- 发布标签推送前确认 main 的 run_tests 已全绿，避免标签指向带红 CI 的提交（v1.21.0 即因此重打，流程见 [fork-release-publishing](../concepts/fork-release-publishing.md)）。
- 逐轮失败要一次收集全部错误再修（本次三层是串行暴露，属各阶段互相掩盖，无法提前预知下一层）。

## 涉及模块

- `.github/workflows/run_tests.yml`、根 `Cargo.toml [workspace.lints]`
- `crates/extensions_ui/`、`crates/acp_thread/`（测试代码）、`crates/zed/src/zed.rs`
