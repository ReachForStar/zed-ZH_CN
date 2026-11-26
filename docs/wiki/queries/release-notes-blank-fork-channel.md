---
title: 应用内查看 Release Note 空白：zed.dev 更新说明接口对汉化版版本号返回空正文
type: query
tags: [release-notes, auto_update, fork-channel, i18n]
created: 2026-11-25
updated: 2026-11-25
status: active
---

# 应用内查看 Release Note 空白

## 问题

汉化版 v1.31.0 里打开更新说明（命令 `auto_update: View Release Notes`、更新完成提示里的「查看更新说明」）后，编辑器区域出现一个 Markdown 预览标签页，标题为 `Release Notes: Zed v1.31.x`，正文完全空白，也没有错误提示。

与本轮 upstream 合并无关：`git diff f126950f9d..upstream/main -- crates/auto_update crates/auto_update_ui` 为空，两个 crate 本轮没有被上游改动。

## 根因

`crates/auto_update_ui/src/auto_update_ui.rs::view_release_notes_locally` 固定请求上游站点的更新说明接口，并把响应正文原样写进 buffer，不判空：

```text
GET {base_url}/api/release_notes/v2/stable/1.31.0+stable   → HTTP 200
{"title":"Release Notes: Zed v1.31.x","release_notes":""}
GET {base_url}/api/release_notes/v2/stable/1.29.3+stable   → HTTP 200
{"title":"Release Notes: Zed v1.29.x","release_notes":"# [Zed v1.29.3](https://zed.dev/releases/stable/1.29.3) ..."}
```

zed.dev 只为它自己正式发布过的版本给正文。汉化版版本号按上游 `main` 走（见 [fork 发布 release 的流程与版本号策略](../concepts/fork-release-publishing.md)），会超前于上游正式发布线：本仓库当前是 `1.31.0`，上游 stable 只发布到 `1.29.3`。因此接口给 200 + 空串，代码把空串当正文渲染，得到空白页；`AppVersion` 带 `+stable` 构建元数据也不影响结果（两个版本号形式实测均为空正文）。

对照可用的正文来源：汉化版自建发布通道的 GitHub Release（`MindFlowLab/zed-ZH_CN` → 301 → `ReachForStar/zed-ZH_CN`，tag `v1.31.0`，`github_api_request` 已配置跟随重定向）正文 4018 字符，就是 `RELEASE_NOTES.md` 生成的中文说明。

## 解法

正文按三级取舍，上游正文优先，空则回退自建通道 Release 正文，两边都空则报错而不是打开空白页：

- `crates/github_releases/src/releases.rs`：`GithubRelease` 补 `#[serde(default)] pub body: Option<String>`（旧响应缺该字段时仍解析成功）。
- `crates/auto_update/src/auto_update.rs`：
  - `release_tag(version)` 统一去掉 `pre`/`build` 元数据后拼 tag（`get_release_asset`、`fork_release_notes`、`release_notes_url` 三处共用）。
  - `GITHUB_RELEASE_REPO` 改 `pub`（注释补「仓库改名后 GitHub API 仍会重定向」）；
  - `fork_release_notes(version: &Version, http_client) -> Option<String>`：先把 `pre`/`build` 清空再拼 tag（与 `get_release_asset` 一致），请求失败记 `log::warn!` 并返回 None，正文为空串也返回 None；
  - `select_release_notes(from_zed: &str, from_fork: Option<&str>) -> Option<&str>`：纯判定函数，放这里而不是 `auto_update_ui`，因为该 crate 的 `[lib] test = false` 使 `auto_update_ui` 内的 `#[cfg(test)]` 模块不参与编译。
- `crates/auto_update_ui/src/auto_update_ui.rs`：`view_release_notes_locally` 保留 `AppVersion::global(cx)` 的 `Version`（不再先转字符串），上游正文为空时才请求 `fork_release_notes`，最终正文经 `select_release_notes` 取值；取不到时走已有的 `notify_release_notes_failed_to_show`（错误提示 + 「在浏览器查看」链接）。

顺带修好一处既有缺陷：`crates/auto_update/Cargo.toml` 的 dev-dependencies 没有 `db` 的 `test-support` feature，`db::ThreadSafeConnection::global` 因此走 `panic!("database not initialized")`，fork 自己写的 `test_auto_update_downloads` 在本地从未跑过。补 `db = { workspace = true, features = ["test-support"] }` 后该用例通过。

## 浏览器链接改指汉化版 Release 页面（同日追加）

`release_notes_url` 原先用 `http_client().build_url("/releases/{channel}/{version}")`，指向上游站点的 `https://zed.dev/releases/stable/1.31.0`；该页只对上游发布过的版本存在，汉化版超前版本号同样打不开。现改为：

```text
https://github.com/{GITHUB_RELEASE_REPO}/releases/tag/v1.31.0
```

- `GITHUB_RELEASE_REPO` 由 `MindFlowLab/zed-ZH_CN` 换成现名 `ReachForStar/zed-ZH_CN`：`gh api repos/ReachForStar/zed-ZH_CN/releases?per_page=3` 实测首条即 v1.31.0、`prerelease=false`、7 个资产，`latest_github_release` 的筛选条件不变；旧名只靠 GitHub 的 301 重定向，直接用现名少一次跳转。
- `gh api …/releases/tags/v1.31.0` 的 `html_url` 与上述拼接结果完全一致，页面实测 `http=200`（旧名同样 200，靠重定向）。
- Nightly / Dev 分支保持指向上游 commits 页（汉化版只发 stable 通道）。
- `crates/zed/src/zed.rs` 关于页的 `ZH_CN_REPO_URL`（GPL-3.0 §5 署名链接）同步换成现名，注释标出它与 `auto_update::GITHUB_RELEASE_REPO` 是同一仓库；两处都是字面量（Rust 不能在 `const` 里对另一个 crate 的 `const` 做 `concat!`），改动时一起核对。

## 涉及模块

- `crates/auto_update_ui`：更新说明标签页的创建与渲染。
- `crates/auto_update`：自建 GitHub Releases 更新通道、版本号与 tag 归一化、正文取舍。
- `crates/github_releases`：GitHub Releases API 解析（`extension_host` 的 wasm 转换按字段取值，新增字段不影响）。

## 验证

- `curl` 实测：`1.31.0`/`1.31.0+stable` 空正文、`1.29.3+stable` 有正文；GitHub Release `v1.31.0` 正文 4018 字符（含 301 重定向后一致）。
- 链接可达性实测：`gh api repos/ReachForStar/zed-ZH_CN/releases?per_page=3` 首条 v1.31.0（`prerelease=false`、7 资产）；`gh api …/releases/tags/v1.31.0` 的 `html_url` 与代码拼接结果一致；`curl -L` 对该 URL 与新/旧仓库名页面均 `http=200`。
- `cargo check --workspace --all-targets`：通过。
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`：通过（另单独对三个受影响 crate 复跑，52s）。
- `cargo fmt --all --check`：通过。
- `cargo test -p auto_update`：20 项通过，含新增 `test_select_release_notes_prefers_upstream_then_fork`（上游正文优先、空正文回退、两侧空白返回 None）、`test_fork_release_notes_queries_stripped_version_tag`（用真实 GitHub API 响应片段作 HTTP 替身，断言请求路径为 `/repos/{GITHUB_RELEASE_REPO}/releases/tags/v1.31.0`，即 `+stable` 已剥掉）与 `test_release_notes_url_points_to_fork_release`（断言链接为 `https://github.com/{GITHUB_RELEASE_REPO}/releases/tag/v1.31.0`）。
- `cargo test -p zed`：148 项通过（关于页仓库链接改动）；`-p github_releases`：10 项通过，含新增 `test_release_body_deserialization`。
- `scripts/i18n/check-missing-keys.py`：0 问题（本次未新增文案）。

## 复发预防

- 汉化版版本号超前于上游发布线时，凡依赖 zed.dev 的界面内容都不可用：更新说明接口（已用自建 Release 正文兜底）、`/releases/{channel}/{version}` 网页（已改指汉化版 GitHub Release 页面）。新增依赖上游站点的内容要先想好自建通道的替代。
- 上游站点这类「状态码 200、字段为空」的响应不能只按 `log_err()` 处理，判空要落在业务层；否则错误提示不会出现，用户只看到空白。
- 需要在某 crate 内加单元测试前先看它的 `[lib] test`：`auto_update_ui`、多数 UI crate 为 `test = false`，测试不会编译，判定逻辑应放到启用测试的下游 crate。
