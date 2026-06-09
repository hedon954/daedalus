# Slice 13 Ratatui REPL UI

## Learning Navigation

- Final artifact: [`../../demo/README.md`](../../demo/README.md)
- Current stage: `08-demo-coder`
- Current slice: Slice 13 Ratatui Agent CLI REPL
- Related guides:
  - [`../../guides/08-demo-coder/16-slice-13-ratatui-agent-cli-repl.md`](../../guides/08-demo-coder/16-slice-13-ratatui-agent-cli-repl.md)
  - [`../../guides/08-demo-coder/17-ratatui-first-principles.md`](../../guides/08-demo-coder/17-ratatui-first-principles.md)

## Current Implementation

Slice 13 已经把 demo 从 examples / tests 推进到真实终端交互：

```text
main.rs
  -> OpenAiCompatibleLlm
  -> ReActAgent
  -> ToolRuntime
  -> OsExecutionRunner
  -> cli::event_loop::run_cli
```

`demo/src/cli/` 当前分三层：

```text
app.rs
  -> CliState / UiMode / UiLine / PendingApprovalView
  -> 输入、日志、滚动、delta 合并这些纯 UI 状态

view.rs
  -> ratatui draw
  -> header / events / prompt-or-approval / footer

event_loop.rs
  -> terminal raw mode / alternate screen
  -> keyboard input
  -> ReActAgent stream consumption
  -> approval result sending
```

这个边界是本轮最重要的设计成果：UI 层只处理交互和展示，不重新实现 approval / retry / sandbox policy。

## Key Decisions

### 1. 直接接 Ratatui，不做 plain stdout 过渡版

用户明确希望 Phase 2B 直接学习和实现 `ratatui`。因此 guide 和实现都从第一步进入真实 TUI：

```text
terminal lifecycle
  -> ratatui draw
  -> keyboard event
  -> CliState update
  -> redraw
```

plain stdout 虽然更简单，但会绕开本阶段真正要学习的 terminal UI 状态模型。

### 2. `UiCommand` 用来隔离“按键解释”和“副作用执行”

同一个按键在不同模式下含义不同：

```text
EditingPrompt:
  a -> 输入字符 a

PendingApproval:
  a -> approve once
```

所以当前设计把按键先解释为 `UiCommand`：

```text
KeyEvent + UiMode
  -> UiCommand
  -> event_loop 执行副作用
```

这避免 `event_loop` 直接散落大量模式判断，也为后续测试 `handle_key` 留出了纯逻辑边界。

### 3. Delta 不是一行一个事件，而是追加到同一段

早期 UI 把每个 `ThinkingDelta` / `TextDelta` 都写成一行，导致日志像这样碎裂：

```text
[thinking]
[thinking] 用户
[thinking] 打了个
[thinking] 招呼
```

当前改为：

```text
TextDelta
  -> append_model_delta
  -> 追加到最后一条 UiLine::Model

ThinkingDelta
  -> append_thinking_delta
  -> 追加到最后一条 UiLine::Thinking
```

这样更符合 streaming text 的真实语义：delta 是同一段内容的增量，而不是独立事件。

### 4. Events 区域支持滚动

Agent stream 会持续增长，固定高度的 event panel 必须支持滚动，否则旧事件会不可见。

当前实现用：

```text
CliState.log_scroll
  -> view.rs Paragraph::scroll((log_scroll, 0))
```

键盘方向键先承担最小滚动能力：

```text
Up   -> scroll_log_up
Down -> scroll_log_down
```

这还不是最终形态；后续可以补 PageUp / PageDown、自动跟随底部和用户手动滚动之间的区别。

### 5. Approval UI 验收以 harmless prompt capability 验证

内置 `npm install` 会触发初始 approval，但用户不希望为了验证 UI 真的安装依赖。

因此更合适的验证方式是给 demo 一个无副作用但 `DefaultDecision::Prompt` 的测试 capability，例如：

```text
echo approval-test
  -> Prompt
  -> ReadOnly sandbox
  -> Deny network
```

这样可以真正走 `CommandNeedsApproval -> approve once/session/reject`，同时批准后只执行 harmless command。

## Verified Behavior

用户已经手动验证：

- TUI 可以启动、输入 prompt、展示 event log。
- prompt 输入、backspace、Enter 提交、退出基础交互可用。
- ReAct Agent stream 能进入 TUI。
- Thinking / text delta 合并后可读性满足基本诉求。
- Events panel 支持滚动。
- Approval UI 基本满足当前 demo 的交互诉求。

## Remaining Work

Slice 13 当前达到“基本 UI 验收 OK”，但还不应该宣称 Phase 2B 全部收口。剩余工程化事项：

- 给 `handle_key` / `UiCommand` / delta 合并补单元测试。
- 补 README 的 Phase 2B 运行说明和手动验收 prompt。
- 根据最终 capability 选择，确认是否保留 harmless approval-test capability。
- 考虑 `try_send` approval result 失败时是否需要在 UI 中显示错误。
- 考虑日志自动滚动和用户手动滚动的状态区分。

## Transferable Pattern

Agent CLI 的 TUI 层可以按这个模式设计：

```text
Domain event stream
  -> UI state reducer
  -> immediate-mode render
  -> keyboard command
  -> domain command / approval result
```

关键不是把 UI 做复杂，而是保证：

- 安全策略仍在 domain/runtime 层。
- TUI 只展示事实和发送用户决策。
- Streaming delta 要按文本段落合并。
- 长事件流必须支持回看。
