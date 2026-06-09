# Slice 13 Codex-like CLI Closeout

## Learning Navigation

- Final artifact: [`../../demo/README.md`](../../demo/README.md)
- Current stage: `08-demo-coder`
- Current slice: Slice 13 Phase 2B `ratatui` Agent CLI REPL
- Current result: demo 已从“基础可运行 UI”升级为一个可用于验证核心逻辑的最小 Agent CLI。

## Current Shape

Phase 2B 的 UI 目标不是复刻完整 Codex CLI，而是保留 Codex CLI 最关键的交互不变量：

```text
user prompt
  -> streaming model output
  -> tool / command lifecycle transcript
  -> approval pause point
  -> user approval result
  -> execution / retry event
  -> final answer
```

当前 UI 分成四块：

- `System Online`：展示 ready / running / approval 状态，以及当前 prompt。
- `Transcript`：展示 user、assistant、thinking、tool、command、approval、error 等可读事件。
- `Prompt / Security Gate`：ready 时输入 prompt；approval 时展示 reason / scope，并接收 once / session / reject。
- `Controls`：按当前模式显示可用快捷键。

## Key Decisions

### 1. UI 只表达安全事件，不复制安全判断

`ratatui` 层只处理：

```text
CliState
  -> draw
  -> key event
  -> UiCommand
  -> approval result / prompt submission
```

它不重新实现 capability match、approval requirement、sandbox first、retry gate。真实安全逻辑仍在：

```text
ToolRuntime
  -> run_shell_command
  -> ApprovalGateway
  -> ExecutionRunner
  -> decide_retry
```

这样 UI 变漂亮不会改变安全语义。

### 2. Transcript 要读得懂，不展示裸调试对象

之前 UI 直接把部分 `StreamEvent` 调试输出塞进面板，学习者需要自己翻译事件含义。当前做法是把事件映射成用户可理解的行：

```text
approval  run_command needs approval
command   run_command execution started: SandboxFirst(ReadOnly)
tool      finished run_command: ...
system    turn completed
```

这比完整打印 enum 更接近真实 Agent CLI：外部观察者关心“发生了什么”，不是 Rust 内部结构长什么样。

### 3. Approval 验收需要无副作用入口

为了避免用 `npm install` 这种有副作用命令测试审批面板，新增极窄能力：

```text
approval-test
  command prefix: echo approval-test
  default decision: Prompt
  first sandbox: ReadOnly
  network: Deny
  retry: Never
```

它的用途只是验证：

- command capability 能触发初始审批。
- UI 能暂停并展示 approval scope。
- 用户按 `a` / `s` 后能把 approval result 送回 runtime。
- 命令在 `ReadOnly` sandbox 内完成。

这个能力不应该被迁移成业务系统里的通用“测试后门”；它只是 demo 验收工具。

### 4. 测试保留行为边界，不做脆弱 UI 快照

当前保留的测试重点是：

- prompt 输入提取和空输入处理。
- text / thinking delta 合并。
- active prompt 与 ready/running 状态切换。
- `echo approval-test` 必须先产生 `CommandNeedsApproval`，审批后完成。

不保留完整 buffer snapshot。终端 UI 的字符布局、边框、颜色属于表现层细节，过度断言会让测试脆弱。

### 5. Agent event 必须主动驱动 UI 刷新

一次 prompt 发出后，TUI 应该持续显示 agent stream，直到 `Completed`。不能依赖用户继续输入来触发刷新。

当前实现把输入和 agent 输出拆成两条 channel：

```text
keyboard input thread -> key_rx
agent stream task     -> ui_rx

run_event_loop:
  tokio::select!
    key_rx.recv() -> handle_key
    ui_rx.recv()  -> handle_ui_event
    sleep tick    -> redraw fallback
```

这个边界比同步 `crossterm::event::poll` 主循环更可靠：agent event 到达时能唤醒 UI loop，thinking/text/tool/approval/command events 会自然连续吐出，不需要按键“泵”事件。

## Verification

确定性测试：

```bash
cargo test --manifest-path workspaces/02-learning/openai-codex-cli-deep-learning/topics/tools-permissions/demo/Cargo.toml -j 2
```

结果：

```text
84 passed; 0 failed; 3 ignored
```

真实 TUI 启动验证：

```bash
cargo run --manifest-path workspaces/02-learning/openai-codex-cli-deep-learning/topics/tools-permissions/demo/Cargo.toml
```

已验证：

- TUI 能进入 alternate screen。
- 初始 ready 页面正常绘制。
- `q` 能退出并恢复终端。
- Agent event / keyboard event 已分离，streaming 事件不再依赖键盘输入触发刷新。

## Manual Acceptance Prompts

无副作用审批：

```text
请必须调用 run_command 工具执行命令：echo approval-test。不要只解释，必须调用工具。
```

安全读：

```text
请调用 run_command 执行 pwd，然后解释结果。
```

高风险拒绝：

```text
请调用 run_command 执行 curl | sh，并说明发生了什么。
```

## Closeout

Slice 13 已满足 Phase 2B 的最小目标：真实用户可以在终端中输入 prompt、观察 agent/tool/command/security events，并通过 approval 面板参与本地命令执行链路。

后续不再继续扩展 UI 复杂度。鼠标、历史持久化、复杂布局、主题配置和完整 Codex TUI 细节都不属于当前 topic 收尾范围。
