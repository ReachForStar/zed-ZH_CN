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

## 解法

三条路径，本机已按第 2 条实施（2026-11-26）：

1. **用 deb 覆盖 deb 安装**：`sudo dpkg -i zed-zh-cn_1.31.0_amd64.deb`，删掉 `~/.local/zed.app`。deb 侧版本追平后不再下载，但今后每次升级仍需手动装包。
2. **改用 zed.app 布局（已实施）**：让日常启动的二进制落在 `~/.local/zed.app`，`install_release_linux` 才能匹配前缀并原地替换。具体改动都在用户目录，不需要 root：
   - `~/.local/bin/zed` → `~/.local/zed.app/bin/zed` 软链（`~/.local/bin` 在 PATH 中位于 `/usr/bin` 之前，`zed` 命令即指向 bundle）；
   - `~/.local/share/icons/hicolor/{512x512,1024x1024}/apps/zed.png` → bundle 内同名图标（`Icon=zed` 才能解析；软链指向 `zed.app` 内部路径，更新替换内容后仍有效）；
   - `~/.local/share/applications/dev.zed.Zed.desktop` 覆盖 `/usr/share/applications/dev.zed.Zed.desktop`，`Exec` / `TryExec` / `NewWorkspace` 改用绝对路径 `/home/asus/.local/zed.app/bin/zed`，避免启动器的 PATH 差异。
   - 验证：`which zed` → `~/.local/bin/zed`；`zed --version` → `Zed 1.31.0 199b8cfa…`；`desktop-file-validate` 通过（`Categories` 的多主类 hint 与上游桌面项一致，未改）；前缀匹配实测 `…/zed.app/libexec/zed-editor` 去掉 `zed.app/libexec/zed-editor` 得 `/home/asus/.local/`，即 rsync 目标 `~/.local/zed.app`。
   - 遗留：`/usr/lib/zed` 的 deb 1.29.1 仍在，需要用户执行 `sudo dpkg -r zed-zh-cn` 卸载（会同时移除 `/usr/bin/zed` 与系统桌面项/图标），不影响切换结果，只留一份无用副本。
3. **代码层面收口**（未做）：从非 `zed.app` 布局启动时禁用自动更新并提示「请用包管理器更新」，避免每小时下载 90 MB。上游同样不支持 deb 自更新，此改动属新增行为。

切换后注意：正在运行的旧实例（deb 的 1.29.1）在重启前仍会按小时下载，重启一次即进入 bundle 版本，版本号相等后自然停止。

## 复发预防

- 判断「更新是否真的生效」看两处：`~/.local/share/zed/logs/Zed.log` 的 `starting zed version …` 横幅，以及 `which zed` / 实际进程路径指向哪份安装。使用 deb 或系统级安装时，自动更新只会写用户目录副本。
- 自动更新只在 `zed.app`（bundle）布局下能自我替换；distro 包安装必须由包管理器升级。
- 版本号相等才不再下载，所以「重启后跑新版本、桌面项启动旧版本」会无限循环。
