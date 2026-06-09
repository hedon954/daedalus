# Slice 13 Ratatui Agent CLI REPL

> Status: current step-by-step guide. 这份 guide 的目标不是告诉你“最终应该长什么样”，而是带你一步步把一个不懂 `ratatui` 的状态推进到可运行的 Agent CLI REPL。

## Learning Navigation

- Final artifact: [`../../demo/README.md`](../../demo/README.md)
- Current stage: `08-demo-coder`
- Current phase: Phase 2B
- Current slice: Slice 13 `ratatui` Agent CLI REPL
- Read first: [`17-ratatui-first-principles.md`](17-ratatui-first-principles.md)
- Current gap: Phase 2A 已证明真实 `OsExecutionRunner` 可用，但 approval 仍由测试 / example 自动回传；还没有真人可操作的 CLI session。
- After this: demo 可以从“可验证库 + examples”升级为“可用 mini Agent CLI”。

## North Star

这一阶段只替换交互层：

```text
test-driven approval responder
  -> ratatui Agent CLI REPL
```

保持不变：

```text
ReActAgent
  -> ToolRuntime
  -> run_shell_command
  -> ApprovalGateway
  -> OsExecutionRunner / SimulatedExecutionRunner
```

TUI 只负责：

- 接收 prompt。
- 展示 `StreamEvent`。
- 在出现 approval request 时让用户按键选择。
- 把 approval result 发回已有的 approval 通道。

TUI 不负责：

- 判断命令危险性。
- 决定 retry。
- 比较 approval scope。
- 运行 sandbox。

## Step 0: 建立最小模块边界

先保持当前目录结构：

```text
demo/src/cli/
  mod.rs
  app.rs
  event_loop.rs
  view.rs
```

三个文件的职责必须分开：

```text
app.rs
  -> UI state and state transitions

view.rs
  -> ratatui draw functions only

event_loop.rs
  -> terminal lifecycle, keyboard events, agent stream events
```

这一步的验收很简单：

```text
lib.rs or main.rs 能 pub mod cli;
三个文件能被编译器看到。
```

不要急着接 agent。先让空 UI 能跑起来。

## Step 1: 在 `app.rs` 定义 UI 状态

先写状态，不写 `ratatui`。UI 的第一性原理是：

```text
event -> state update -> draw state
```

建议先定义：

```rust
pub struct CliState {
    pub mode: UiMode,
    pub input: String,
    pub lines: Vec<UiLine>,
    pub pending_approval: Option<PendingApprovalView>,
    pub should_quit: bool,
}

pub enum UiMode {
    EditingPrompt,
    RunningAgent,
    PendingApproval,
}

pub enum UiLine {
    User(String),
    Model(String),
    Tool(String),
    Command(String),
    Approval(String),
    Error(String),
}

pub struct PendingApprovalView {
    pub approval_id: String,
    pub reason: String,
    pub scope_summary: String,
}
```

为什么先这么写：

- `mode` 决定按键含义。`a/s/r` 只应该在 `PendingApproval` 下生效。
- `input` 是底部输入框内容。
- `lines` 是中间日志区。
- `pending_approval` 是 approval panel 的数据来源。
- `should_quit` 让 event loop 不需要猜用户是否退出。

接着写几个小方法：

```rust
impl CliState {
    pub fn new() -> Self { ... }

    pub fn push_char(&mut self, c: char) { ... }

    pub fn backspace(&mut self) { ... }

    pub fn take_prompt(&mut self) -> Option<String> { ... }

    pub fn push_line(&mut self, line: UiLine) { ... }
}
```

这一步不要碰 `StreamEvent`。先把纯 UI 输入行为跑通。

验收：

- 输入字符会进入 `input`。
- backspace 会删除最后一个字符。
- Enter 后 `take_prompt()` 返回 prompt，并清空 `input`。
- 空 prompt 不启动 agent。

## Step 2: 在 `view.rs` 画静态 Ratatui 页面

现在接 `ratatui` 的最小绘制。

目标布局：

```text
┌ header: model / cwd / status ┐
│ event log                    │
│                              │
├ input or approval panel       │
└ footer: keys                  ┘
```

先写一个入口函数：

```rust
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph};

use super::app::CliState;

pub fn draw(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(area);

    draw_header(frame, chunks[0], state);
    draw_log(frame, chunks[1], state);
    draw_input_or_approval(frame, chunks[2], state);
    draw_footer(frame, chunks[3], state);
}
```

第一版不要追求美观，只要信息完整：

- header 显示当前 `UiMode`。
- log 显示最近若干条 `UiLine`。
- input panel 在 `EditingPrompt` 下显示 prompt。
- approval panel 在 `PendingApproval` 下显示 reason / scope / a/s/r。
- footer 显示快捷键。

关键心智模型：

```text
ratatui draw 不修改状态。
draw 只读 CliState，把状态翻译成 widgets。
```

验收：

- 不接 agent，也能启动一个空 TUI。
- 输入框、日志框、footer 都能显示。
- 改 `CliState.mode` 后，底部面板能从 input 变成 approval。

## Step 3: 在 `event_loop.rs` 管理终端生命周期

现在写真正的 TUI loop。

核心结构：

```rust
pub fn run_cli(mut state: CliState) -> anyhow::Result<()> {
    setup_terminal()?;

    let result = run_event_loop(&mut state);

    restore_terminal()?;
    result
}
```

实际写的时候要注意：即使 loop 中途出错，也要恢复 terminal。第一版可以先照 `daedalus-tui` 的方式手动 cleanup；后续再抽 `TerminalGuard`。

最小 event loop：

```rust
loop {
    terminal.draw(|frame| {
        crate::cli::view::draw(frame, frame.area(), &state);
    })?;

    if state.should_quit {
        break;
    }

    if crossterm::event::poll(Duration::from_millis(50))? {
        if let Event::Key(key) = crossterm::event::read()? {
            handle_key(&mut state, key);
        }
    }
}
```

先支持这些键：

```text
q / Esc -> quit
char    -> append to input
backspace -> delete char
Enter -> submit prompt
```

这里的 Enter 先不要接 agent，可以先：

```text
input "hello"
  -> log append User("hello")
  -> mode = RunningAgent
  -> log append Model("agent integration pending")
  -> mode = EditingPrompt
```

验收：

- `cargo run` 能打开全屏 TUI。
- 可以输入文本。
- Enter 后文本进入 log。
- `q` 或 `Esc` 退出后 shell 恢复正常。

## Step 4: 把按键处理从 event loop 抽成纯函数

不要让 `event_loop.rs` 很快变成一坨 match。建议抽：

```rust
pub enum UiCommand {
    None,
    Quit,
    SubmitPrompt(String),
    ApproveOnce(String),
    ApproveSession(String),
    Reject(String),
}

pub fn handle_key(state: &mut CliState, key: KeyEvent) -> UiCommand {
    match state.mode {
        UiMode::EditingPrompt => handle_prompt_key(state, key),
        UiMode::RunningAgent => handle_running_key(state, key),
        UiMode::PendingApproval => handle_approval_key(state, key),
    }
}
```

为什么要返回 `UiCommand`：

```text
key handling 负责解释用户意图；
event loop 负责把意图接到 agent / approval gateway。
```

这样后续测试也更容易：

- `EditingPrompt + Enter` -> `SubmitPrompt(prompt)`。
- `PendingApproval + a` -> `ApproveOnce(approval_id)`。
- `PendingApproval + s` -> `ApproveSession(approval_id)`。
- `PendingApproval + r` -> `Reject(approval_id)`。

验收：

- key handling 可以写单元测试。
- `event_loop.rs` 只处理 `UiCommand`，不关心 UI 细节。

## Step 5: 接入 Agent Stream，但先只读事件

现在才接 `ReActAgent`。

当收到 `SubmitPrompt(prompt)`：

```text
state.mode = RunningAgent
spawn async agent run
agent stream event -> ui_event_tx
```

建议 event loop 内部使用一个 channel：

```rust
enum UiEvent {
    Key(KeyEvent),
    Agent(StreamEvent),
    AgentFinished,
    AgentFailed(String),
    Tick,
}
```

第一版可以保持 keyboard poll 在主 loop，agent task 通过 `tokio::sync::mpsc` 把事件送回来：

```text
ReActAgent::run(prompt)
  -> StreamEvent
  -> tx.send(UiEvent::Agent(event))
  -> TUI loop receives
  -> state.apply_agent_event(event)
  -> redraw
```

在 `app.rs` 写：

```rust
impl CliState {
    pub fn apply_agent_event(&mut self, event: StreamEvent) {
        match event {
            StreamEvent::TextDelta { delta, .. } => {
                self.push_line(UiLine::Model(delta));
            }
            StreamEvent::ToolRunStarted { name, .. } => {
                self.push_line(UiLine::Tool(format!("tool started: {name}")));
            }
            _ => {
                self.push_line(UiLine::Tool(format!("{event:?}")));
            }
        }
    }
}
```

注意：这里可以先粗糙地展示 debug，后续再精修每类事件。第一目标是证明 stream 能实时进入 TUI。

验收：

- Enter 后 UI 不冻结。
- 模型 streaming text 能逐步进入 log。
- tool events 能进入 log。
- agent 完成后 `mode` 回到 `EditingPrompt`。

## Step 6: 处理 `CommandNeedsApproval`

现在加入 approval panel。

当 `apply_agent_event` 收到 `CommandNeedsApproval`：

```text
state.mode = PendingApproval
state.pending_approval = Some(PendingApprovalView { ... })
log append Approval("needs approval")
```

UI 显示：

```text
Approval required
reason: ...
scope: ...

[a] approve once   [s] approve session   [r] reject
```

此时键盘含义改变：

```text
a -> UiCommand::ApproveOnce(approval_id)
s -> UiCommand::ApproveSession(approval_id)
r -> UiCommand::Reject(approval_id)
```

event loop 收到命令后，发送 approval result：

```rust
ToolApprovalResult {
    approval_id,
    decision: UserApprovalDecision::Approved {
        persistence: ApprovalPersistence::Once,
    },
}
```

或 session / rejected。

然后：

```text
state.pending_approval = None
state.mode = RunningAgent
log append Approval("approved once/session/rejected")
```

验收：

- approval request 出现时，底部从 input 切成 approval panel。
- 按 `a/s/r` 不会进入 prompt input，而是发送 approval decision。
- approval decision 后 agent 能继续执行或终止。

## Step 7: 验证 Session Approval

这一步不是 UI 美化，而是验证 Phase 2B 的核心价值。

流程：

```text
run #1:
  prompt requests a command that needs approval
  press s
  command continues

run #2:
  same command / same cwd / same sandbox / same network
  should not ask approval again
```

这要求：

- 多次 ReAct run 共享同一个 `ApprovalGateway`。
- TUI 只创建一个 CLI session，不要每次 prompt 都重建 gateway。
- `ApprovalPersistence::Session` 写入的是 gateway 内部 session store。

验收：

- 第一次 prompt 出现 approval panel。
- 按 `s` 后执行通过。
- 第二次同 scope 不再出现 approval panel。
- 换 cwd / sandbox / network 后不能复用旧 approval。

## Step 8: 最后再做可读性

功能跑通后，再处理体验：

- 日志滚动。
- 不同 `UiLine` 使用不同颜色。
- model text delta 合并成段落，而不是每个 delta 一行。
- command attempt 显示 `SandboxFirst` / `NoSandboxRetry`。
- failed / denied / retry 用醒目的颜色。
- footer 根据 `UiMode` 显示不同快捷键。

不要太早做这些，因为过早美化会掩盖真正的状态机问题。

## Tests

优先测试纯逻辑，不测试 terminal 像素：

- `handle_key`：
  - prompt mode 下字符输入、backspace、enter。
  - pending approval 下 `a/s/r`。
- `apply_agent_event`：
  - text delta 进入 log。
  - command approval 进入 pending mode。
  - final event 让 mode 回到 editing。
- approval command mapping：
  - `ApproveOnce` -> `ApprovalPersistence::Once`。
  - `ApproveSession` -> `ApprovalPersistence::Session`。
  - `Reject` -> `UserApprovalDecision::Rejected`。

手动验收：

```bash
cargo run
```

输入一个会触发写入的命令请求，观察：

```text
run_command mkdir approved-dir
  -> sandbox first failed
  -> approval prompt
  -> approve once/session
  -> no-sandbox retry
  -> final answer
```

## Critical Lens

这里最容易犯的错误是把 TUI 做成“另一个 orchestrator”。不要这样。

TUI 是人机交互层，不是安全策略层。它应该相信并展示 runtime 给出的事实：

- 为什么要审批。
- 审批范围是什么。
- 当前 attempt 是否在 sandbox。
- retry decision 是什么。
- 最终工具结果是什么。

如果 UI 自己开始重新判断命令危险性，就会出现两套安全逻辑，后续很难保证一致。

## Stop Rules

- 不做 plain stdout 过渡版。
- 不做复杂主题皮肤。
- 不做完整历史持久化。
- 不做 mouse support。
- 不在 UI 层复制 approval / retry / sandbox 逻辑。
