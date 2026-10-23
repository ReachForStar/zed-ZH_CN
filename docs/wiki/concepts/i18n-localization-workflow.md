---
title: 汉化口径与批量替换工作流
type: concept
tags: [i18n, 汉化, rust-i18n, locale, workflow]
created: 2026-10-16
updated: 2026-10-23
status: active
---

# 汉化口径与批量替换工作流

## 定义

汉化指把 Zed 界面文本改为 `t!("<键>")` 调用，译文写入 `crates/zed_i18n/locales/{en,zh-CN}/<crate>.*.toml` 分片；运行期语言由 `ui_language` 设置决定（`zed_i18n::init()` 只在 `crates/zed/src/main.rs` 调用一次）。新增键必须同时补 en 与 zh-CN 两侧，不留英文硬编码。

## 汉化口径

**汉化**：菜单项、按钮与标签、tooltip、输入框占位符、弹窗标题与正文、toast 文案、空状态提示、无障碍 `aria_label`，以及所有用户能看到的英文句子。

**不汉化**（判定后整体跳过，避免噪音改动）：

- 元素 id：`Button::new(id, ..)`、`ButtonLike::new(id)`、`IconButton::new(id, ..)`、`PopoverMenu::new(id)`、`ListItem::new(id)`、`CopyButton::new(id, ..)` 的首参；`key_context("..")`。
- 非界面语义串：`#[action(name = "..")]` 属性值、telemetry 事件名、HTTP 头与认证字面量（`Accept`/`Authorization`/`Bearer`）、`expect()`/`context()` 消息、`log::` 与 `anyhow!` 内部消息、模型可见的工具输出。
- 品牌与产品名（OpenAI、Anthropic、Ollama、OpenRouter、LM Studio、GitHub Copilot、DeepSeek、Mistral 等）、编辑器与键位预设名（VS Code、JetBrains、Sublime Text、Emacs、Vim…）、语言与语法名。
- component preview 画廊与 `ui`/`gpui` examples 的示例串、文档注释里的示例、`#[cfg(feature = "dev-tools")]` 开发面板、`Panel::persistent_name()` 等 trait 要求 `&'static str` 的项。

## 批量替换流程

1. **收集候选**：对每个 crate 抓 `"首字母大写的英文串"` 出现在 UI 位置的行，产出 `tmp/i18n-cand/<crate>.txt`（含出现次数与行预览），逐条判定是否属于汉化口径。
2. **写计划表**：`tmp/i18n-in/<批次>.tsv`，每行 `相对路径:行号<TAB>key<TAB>译文[<TAB>次数[<TAB>行限定]]`；`last` 表示取该行最后一个字面量，`<行号>` 表示只替换该行，`allbut:<行号>` 表示排除该行。用 `scripts/i18n/i18n-mkplan.py` 从源码取准确字面量并生成计划表。
3. **按 crate 拆表并应用**：把批次表按 `crates/<crate>/` 拆成 `tmp/i18n-plan/<crate>.tsv`，再 `python3 scripts/i18n/i18n-apply.py <crate>`。脚本会校验字面量出现次数、替换为 `t!`、补 `use zed_i18n::t;`、给 crate 补 `zed_i18n` 依赖、写两侧 locale 并回读校验。次数不符时整体拒绝写入并一次性报出全部不符项。
4. **修类型**：按下文「类型适配」逐处修编译错误。
5. **校验并提交**：`cargo fmt --all` → `python3 scripts/normalize-crlf.py` 还原 CRLF → `cargo clippy --workspace --all-targets -- --deny warnings`（与 CI 同款门禁）→ 只暂存具体文件提交。

## 类型适配

`t!("key")` 返回 `Cow<'_, str>`，`t!("key", 命名参数 = 值)` 返回 `String`（rust-i18n 3.1.5 实测）。据此：

- 形参是 `&str`：传 `&t!("key")`。多行调用（如 `window.prompt(level, 标题, Some(详情), answers, cx)`）里标题与详情都是 `&str`，两处都要加 `&`。
- 目标类型是 `SharedString`：直接给 `t!("key")`（有 `From<Cow<'_, str>>`）；`Label::new` 等取 `impl Into<SharedString>` 的位置同理。
- `match`/`if` 各分支混用有参与无参 `t!`：只在 `Cow` 分支加 `.to_string()`；给命名参数分支（已是 `String`）再加会触发 `clippy::redundant_clone`，`-D warnings` 下直接编译失败。
- 闭包需要 `Fn`（可能被多次调用）：`Cow`/`String` 不是 `Copy`，捕获后要在闭包内 `.clone()`，或把形参改成 `impl Into<SharedString>` 后在函数体开头 `let x = x.into();`（见 `crates/search/src/search.rs` 的 `render_action_button`）。
- `let` 声明了 `&'static str` 再在两个分支赋值的写法：改成 `Cow<'static, str>`。
- 弹窗按钮不能写 `&[t!("key")]`（`&[Cow]` 不满足签名），要写 `&[gpui::PromptButton::new(t!("key"))]`。
- 含插值的 `format!` 文案改 `t!("key", 参数 = 值)`，locale 里写 `%{参数}`，例如 `git_action_failed = "git %{action} 失败"`。
- 返回 `impl Iterator<Item = &str>` 的辅助函数（拼接消息片段一类）改为 `Item = Cow<'_, str>`，借用分支写 `Cow::Borrowed(..)`、兜底分支直接给 `t!`（`Cow<'static, str>` 因协变可收窄）；原先 `Itertools::intersperse(.., "\n").flat_map(str::chars)` 的取值方式要改成 `.map(Cow::into_owned).collect::<Vec<_>>().join("\n").chars()`，否则分隔符与 `str::chars` 都对不上类型。

## 验证

- **键完整性检查**：`python3 scripts/i18n/check-missing-keys.py` 扫描 `crates/**/*.rs` 里静态字面量形式的 `t!("键")`（含换行写法，已屏蔽 `//` 注释），比对 en 与 zh-CN 键集合，报「代码引用但 locale 缺失」与「仅单侧有」。退出码非 0 即有问题；已加入 CI 的 `check_style` job。脚本要求 Python 3.11+（用 `tomllib`）。
- 测试不调用 `zed_i18n::init()`，单测下 locale 为 `en`，`t!` 解析回英文原文，因此按标签精确匹配的用例（如 `simulate_prompt_answer("Save All")`）不受影响。
- 全量门禁：`cargo clippy --workspace --all-targets -- --deny warnings`。只跑 `cargo check` 漏掉 test 目标与 lint。
- run_tests 已改为手动触发（`gh workflow run run_tests --ref main`），文案类提交不再自动拉全量 clippy；需要判定时按该 run 结论为准，勿以「本地通过」代替。

## 未覆盖项

- 依赖 `&'static str` 的 trait 项（`Panel::icon_tooltip`、`persistent_name`）与其它静态上下文。
- 品牌名、语言名、`#[action(name = ..)]`、telemetry 事件名、HTTP 头等按口径保留英文的位置。
- component preview 画廊、`dev-tools` 面板与 examples 的示例串。
- 动态拼接的键（`t!(format!(..))`）与 `ts!`/`tr!` 等变体不在键完整性检查范围内。

## 已修复的漏键（2026-10-16）

批次汉化后由 `check-missing-keys.py` 查出 20 处「代码引用但 locale 缺失」（含仅单侧缺失）的键，已全部补齐（缺失时界面会直接显示键名）：

- `agent_ui.thread_view.copy`（上下文菜单「复制」）、`debugger_ui.panel.continue_thread`（en 侧缺，上游原文 "Continue Thread"）、`settings_ui.page_data.agent_prevent_idle_sleep_title` / `_description`（上游原文 "Prevent Idle Sleep" / "Whether to keep the system awake while agent threads are running."）。
- `language_tools.highlights_tree_view.*` 7 个与 `language_tools.key_context_view.*` 9 个（两张表整体缺失，来自批次 4 的手工改造）。

## 二次合并补译（2026-10-16，`28d0a3547f`）

上游二次合并（提交 `30e413d270`）带来 2 条新界面文案，同时暴露出上轮合并引入却始终未汉化的整块界面：

- 上游新增：`agent_ui.thread_view.submission_status_accepted_connection_lost`（原文 "Message accepted; agent connection lost"）、`submission_status_connection_lost_unknown`（"Connection lost; message acceptance is unknown"）——消息因连接断开而状态未知时显示的可恢复消息状态。
- 上轮漏译的「可恢复消息」整块（同函数 `render_recoverable_submissions` / `submission_text_parts`）：`submission_status_sending`（"Sending message…"）、`submission_status_waiting`（"Waiting for message to appear…"）、`submission_failed_to_send`（"Message failed to send: %{error}"）、`submission_status_cancelled`（"Message cancelled"）、`submission_status_fallback`（"Message"）、按钮 `submission_restore` / `submission_discard`（"Restore" / "Discard"）、`submission_restore_tooltip`，以及附件占位符 `submission_attachment_resource` / `_image` / `_audio` / `_unsupported`（`[Resource attachment]` 等）。全部键已补 en/zh-CN，提交 `b3fdd26c6d`。
- 该块里 `handle_thread_error(anyhow!(..))` 的错误消息按既有口径保持英文（同文件另有 4 处同类消息，均未汉化，改动会破坏口径一致性）。

## 三次合并补译（2026-10-23，`91d3b962ae3`）

上游三次合并（120 提交）带来 11 处新的界面文案，并改写了 git 面板确认框的消息结构：

- **新增键**：`sidebar.thread.stop_generation_activity_unconfirmed`、`agent_ui.thread_view.stop_generation_activity_unconfirmed`、`agent_ui.thread_view.activity_unconfirmed` / `activity_unconfirmed_tooltip` / `activity_unknown` / `activity_unknown_tooltip` / `thinking_always_on_at_effort`、`agent_ui.conversation_view.copy_saved_input` / `copy_saved_input_tooltip` / `discard_saved_submissions` / `discard_saved_submissions_tooltip`、`editor.debugger_hover.copy_value` / `copy_as_expression` / `add_to_watch` / `value_actions` / `session_not_paused`、`settings_ui.page_data.display_p3_title` / `display_p3_description`、`git_ui.git_panel.changes` / `unstaged_changes` / `discard_changes_to_count_files` / `trash_untracked_count_files` / `file` / `files`。
- **改模板的既有键**：`git_ui.git_panel.discard_changes_confirm` 加 `%{changes}` 插值、`git_ui.git_panel.discard_trash_count_files` 加 `%{changes}` 插值、`settings_ui.page_data.on_new_window_description` 换成上游新文案；en 与 zh-CN 两侧模板同步改。
- **文案随模块迁移**：上游把子智能体卡片文案迁到 `agent_ui/src/conversation_view/subagent_card.rs`（`SubagentCardState::title`），汉化键随之迁到该文件；`title()` 返回 `SharedString`，`t!` 结果用 `.into()` 转换。
- **枚举值也要汉化**：上游 git 面板用 `file` / `files` 两个新键替代硬编码的 `"File"` / `"Files"`，`file_label` 改为 `t!` 结果后传给 `stage_count_files` / `restore_count_files` 等既有键的 `%{files}` 占位符。

## 四次合并补译（2026-10-23，`604a0ff023`）

上游新增 ACP 会话间消息（父子会话委派）的 transcript 行，引入 6 处界面文案：

- **新增键**（`agent_ui.thread_view` 分节）：`session_message_to`（"To"）、`session_message_from`（"From"）、`session_message_between_sessions`（"Message between sessions"）、`open_session`（"Open Session"，参与者按钮 tooltip）、`unavailable_session`（"Unavailable Session"，会话未知时的占位标签）、`main_agent`（"Main Agent"，根会话在其它会话视图中的名称）。
- **译文口径**：`To` / `From` 作为方向标签译作「发送至」/「来自」（保留单字风格，与界面配合）；`Main Agent` 沿用既有 `subagent_output_tooltip` 里的「主智能体」叫法。
- **顺带补上漏译**：`subagent_label` 的兜底 `"Subagent"` 改为 `t!("agent_ui.thread_view.subagent")`（键早已存在）。
- **单测下的 locale 行为**：测试不调 `zed_i18n::init()`，`t!` 回退链命中 en，因此按英文匹配的用例（如 `Some("Main Agent")`）不受汉化影响——前提是 `zed_i18n` 已按新 locale 重新编译（改 locale 后增量编译不重建 `zed_i18n` 时，`t!` 会回退为键名，表现为断言拿到 `agent_ui.thread_view.main_agent`；`touch crates/zed_i18n/src/zed_i18n.rs` 强制重建即可）。

## 关联页面

- [汉化 fork 同步 upstream 的合并模式与特性保留清单](fork-upstream-sync.md) — 合并期 `t!` × 上游重构冲突的解法与特性保留清单。
- [2026-10 合并上游 main：冲突消解决策与编译失败链](../queries/upstream-merge-2026-10.md) — 本轮合并后汉化补齐与门禁执行记录。
