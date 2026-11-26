---
title: 更新一直在下载：deb 安装布局与 zed.app 更新布局不匹配
type: query
tags: [auto_update, release, deb, linux, fork]
created: 2026-11-26
updated: 2026-11-26
status: active
---

# 更新一直在下载

## 问题

应用每小时检查一次更新，反复提示下载并「更新完成」，但版本没有变化，下一次启动又继续下载。

## 证据（本机实测）

- 实际运行的是 deb 安装：进程为 `/usr/lib/zed/libexec/zed-editor`，`dpkg -s zed-zh-cn` 报 `Version: 1.29.1`（11-13 安装）；`which zed` → `/usr/bin/zed` → `../lib/zed/bin/zed`，桌面项 `Exec=zed`。
- 自动更新写的是另一份副本：`~/.local/zed.app` 存在，`~/.local/zed.app/bin/zed --version` 报 `Zed 1.31.0 199b8cfa…`；`/usr/lib/zed/bin/zed --version` 报 `1.29.1 4595b847…`。
- `~/.local/share/zed/logs/Zed.log`：3 次 `downloaded update`、3 次 `Restarting process, using app path: "/home/asus/.local/zed.app/libexec/zed-editor"`、2 次 `auto-update check failed`；版本横幅交替出现——13:54:08 起 `1.29.1`、13:54:29 起 `1.31.0`（重启用新副本）、13:55:26 与 13:55:37 又回到 `1.29.1`。
- 轮询间隔 `POLL_INTERVAL = 60 分钟`；每次下载的 `zed-linux-x86_64.tar.gz` 约 90 MB。

## 根因

`crates/auto_update/src/auto_update.rs::install_release_linux` 只认 `script/bundle-linux` 的 `zed.app` 布局：

```rust
let expected_suffix = format!("{}/libexec/zed-editor", app_folder_name); // "zed.app/libexec/zed-editor"
let mut to = home_dir.join(".local");
if let Some(prefix) = running_app_path.to_str().and_then(|s| s.strip_suffix(&expected_suffix)) {
    to = PathBuf::from(prefix);
}
// rsync -av --delete {下载解压出的 zed.app} {to}
```

deb 包安装到 `/usr/lib/zed/{bin,lib,libexec}`，运行路径 `/usr/lib/zed/libexec/zed-editor` **不以** `zed.app/libexec/zed-editor` 结尾 → `strip_suffix` 失败 → `to` 保持 `$HOME/.local`，rsync 把新 `zed.app` 写到 `~/.local/zed.app`。`/usr/lib/zed` 一字未动（且属 root，更新器不走 `dpkg` 也不提权）。

于是形成闭环：从桌面项启动 → deb 的 1.29.1 → 每小时发现 GitHub 最新 release（1.31.0）比 1.29.1 新 → 再下载 90 MB → 写 `~/.local/zed.app` → 「重启」确实跑起 1.31.0，但下次从桌面项启动又回到 1.29.1。

`check_if_fetched_version_is_newer_non_nightly` 只比版本号，无法感知「新版本已经下载过」，因此每轮都会重新下载。

## 解法（三选一，均需用户决定）

1. **用 deb 覆盖 deb 安装（推荐，最简单）**：`sudo dpkg -i zed-zh-cn_1.31.0_amd64.deb`，随后删除 `~/.local/zed.app` 避免两份 1.31.0 混淆。deb 侧变成 1.31.0 后版本号相等，不再下载。
2. **改用 zed.app 布局**：卸载 deb，把桌面项 `Exec` 改成 `~/.local/zed.app/bin/zed`（或 `~/.local/bin/zed`），此后自动更新可以正常替换自身。
3. **代码层面收口**（上游同样不支持 deb 自更新）：从非 `zed.app` 布局启动时禁用自动更新并提示「请用包管理器更新」，避免每小时下载 90 MB。是否需要上游式改动待用户决定。

## 复发预防

- 判断「更新是否真的生效」看两处：`~/.local/share/zed/logs/Zed.log` 的 `starting zed version …` 横幅，以及 `which zed` / 实际进程路径指向哪份安装。使用 deb 或系统级安装时，自动更新只会写用户目录副本。
- 自动更新只在 `zed.app`（bundle）布局下能自我替换；distro 包安装必须由包管理器升级。
- 版本号相等才不再下载，所以「重启后跑新版本、桌面项启动旧版本」会无限循环。
