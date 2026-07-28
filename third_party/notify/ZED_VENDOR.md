# Vendored notify（zed-industries fork + 本地补丁）

## 来源

- 上游仓库：https://github.com/zed-industries/notify
- 基线 rev：`0890bbb8ca40a4b5d1f67031698dd7918b37d991`
  （即 Zed 主仓库原先通过 `[patch.crates-io]` 引用的版本，
  其中已包含 zed 的 fsevent 修复）
- notify 版本：9.0.0-rc.4，notify-types 版本：2.1.0
- 许可：CC0-1.0（见 LICENSE-CC0）

## 本地改动（相对基线 rev）

仅 `notify/src/windows.rs`：

1. `BUF_SIZE` 16KB → 64KB：增大单次完成传输容量，降低溢出频率。
2. 新增 `ERROR_NOTIFY_ENUM_DIR`（1022，缓冲区溢出）处理分支：
   重新发起异步读取 + 上报带 `Flag::Rescan` 的事件，
   而不是落入默认分支 `log::error! + unwatch` 静默永久移除监听。
3. `ERROR_SUCCESS` + 零字节同样按溢出处理：部分 Windows 版本（如
   Windows 11）溢出不上报 1022，而是以成功 + 零字节呈现，
   原代码解析全零缓冲区后静默丢弃事件；现改为上报 rescan。

## 根因背景

基线版本中，溢出错误码 1022 落入 `handle_event` 的兜底 `_` 分支，
导致 OS 层 watch 被静默撤销且上层（Zed `GlobalWatcher`）毫不知情，
表现为文件树与 git 面板冻结、重开项目才恢复；而在 Windows 11 上
溢出以 `ERROR_SUCCESS` + 零字节呈现，事件被静默丢弃但 watch 存活，
表现为文件树悄悄过期。大型仓库（如根目录下存在 cargo `target/`）
编译时极易触发。

## 维护须知

- 根 `Cargo.toml` 的 `[patch.crates-io]` 已将 `notify` / `notify-types`
  指向本目录（path 依赖）。
- 当上游 Zed 升级 notify rev 或 notify 发布新版时，需重新同步本目录
  并重新应用上述两处补丁；建议同时向 zed-industries/notify 或
  notify-rs/notify 提交 PR 推动上游修复，修复合入后即可移除 vendor。
- 本目录是独立的 cargo workspace（notify 自身的 workspace），
  不参与 Zed workspace 的成员列表。
