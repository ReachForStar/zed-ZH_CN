---
title: git bash 调用 pwsh 脚本的两类失败：chcp 包装与反斜杠路径
type: query
tags: [windows, git-bash, pwsh, powershell, 编码, git-proxy]
created: 2026-09-26
updated: 2026-09-26
status: active
---

# git bash 调用 pwsh 脚本的两类失败：chcp 包装与反斜杠路径

## 问题

发布 v1.21.0 时 `git push` 被重置，触发 `git-proxy` skill 开关代理。skill 文档写的调用形式是
`cmd //c "chcp 65001 >nul && pwsh -ExecutionPolicy Bypass -File <路径>\git-proxy.ps1" <命令>`，
按此执行连续失败；改用直接调用与改正斜杠路径后才成功。两类失败的报错都不含有效线索，需要记录避免复踩。

## 根因与解法

在 `D:\file\Zed` 的 git bash（MSYS）下逐条实测，退出码均为本次实跑结果：

| 写法 | 结果 | 判定 |
|---|---|---|
| `cmd //c "chcp 65001 >nul && echo hi"` | 退出码 1，`系统找不到指定的路径`（GBK 字节，按 UTF-8 读为乱码） | `chcp` 这一步就失败，与 pwsh、脚本路径无关 |
| `cmd //c "pwsh -File C:/正斜杠/git-proxy.ps1 help"` | 退出码 0 | 去掉 `chcp` 后 cmd 包装本身可用 |
| `pwsh -File "C:/正斜杠/git-proxy.ps1" status` | 退出码 0 | 推荐写法 |
| `pwsh -File C:\反斜杠\git-proxy.ps1 status` | 退出码 64，报 `'C:Usersasus...ps1' 无法识别为脚本文件` | MSYS 把参数中的反斜杠剥掉了 |
| `cmd //c "echo hi"` | 退出码 0 | 排除 cmd 包装整体不可用 |

- **失败一：`chcp` 包装**。给 pwsh 切 UTF-8 码页的 `cmd //c "chcp 65001 && ..."` 在 git bash 里执行 `chcp`
  即报错退出，整条命令到不了 pwsh。不要用这个包装；要解决输出编码问题应在脚本内设置编码（见下），
  而不是在 shell 层切码页。
- **失败二：裸反斜杠 Windows 路径**。MSYS 对参数做路径转换，反斜杠被吞，pwsh 收到拼在一起的
  `C:Usersasus.qoder-cnskills...ps1` 并判定不是脚本文件。写成引号包裹的正斜杠绝对路径即可，
  例如 `-File "C:/Users/<user>/.qoder-cn/skills/git-proxy/scripts/git-proxy.ps1"`。

## 附带结论：Write-Host 在管道下按 GBK 写字节

改前所有成功调用的中文输出仍是乱码（`代理状态` 写出 `B4 EA C0 E5...`，即码页 936 字节），
但代理地址、开/关状态是 ASCII，仍可判读，所以不影响结果、只是可读性差。原因是脚本用 `Write-Host`
输出，输出被重定向到管道时 .NET 按系统 OEM 码页写字节；而 `Write-Output` 走的是 UTF-8。

在脚本入口加一行即可让重定向输出变为 UTF-8 字节（实测对比 `od -c`）：

```powershell
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
```

未改脚本时的补救：管道接 `| iconv -f GBK -t UTF-8`（git bash 自带 `/usr/bin/iconv`）。
注意 `pwsh -Command "[Console]::OutputEncoding.WebName"` 在无控制台时返回 `utf-8`，与实际写出的
GBK 字节不一致，不能用它判断编码问题。

## 涉及模块

- 用户 skill `git-proxy`（`~/.qoder-cn/skills/git-proxy/`）：SKILL.md 的调用方式已改为直接 pwsh 调用，
  `scripts/git-proxy.ps1` 入口加了编码设置。
- 本仓库代码未受影响；踩坑发生在 push 阶段（`Recv failure: Connection was reset` 之后的代理开关），
  发布流程见 [fork 发布 release 的流程与版本号策略](../concepts/fork-release-publishing.md)。

## 复发预防

- 在 git bash 里调用 `.ps1`：一律 `pwsh -NoProfile -ExecutionPolicy Bypass -File "<正斜杠绝对路径>"`，
  不加 `chcp` 包装、不传裸反斜杠路径。
- 见到"路径无法识别"类乱码报错，先用 `od -c` 或直接把路径写成正斜杠试一次，判断是编码显示问题还是
  参数真被改写。
- 中文脚本要让 Agent 稳定读输出，在脚本入口设 `[Console]::OutputEncoding`，不要指望调用侧切码页。
