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

当前 UI 分成四块，但视觉上从“大框仪表盘”收敛成更接近现代 coding-agent CLI 的 transcript-first 形态：

- 顶部状态行：展示 ready / running / approval 状态，以及当前 focus。
- Transcript：展示 user、assistant、thinking、tool、command、approval、error 等可读事件。
- Prompt / Security Gate：ready 时输入 prompt；approval 时展示 reason / scope，并接收 once / session / reject。
- 底部快捷键：按当前模式显示可用操作。

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

### 6. Transcript 自动跟随必须按视觉行计算

事件进入 UI state 之后，还要保证用户真的能看见尾部。`ratatui::Paragraph::scroll` 使用的是渲染后的视觉行，而不是 `Vec<UiLine>` 的逻辑事件数。

如果用 `state.lines.len() - 1` 作为 tail scroll，长 assistant 文本一换行，最终的 `turn completed` 可能已经在面板下方，但 scroll offset 仍然不够。表现就是：事件其实已经到达，Header 也回到 ready，但 Transcript 底部没有展示完整，直到用户再输入一个字符触发新的状态变化。

当前做法：

```text
UiLine
  -> render_line
  -> estimate terminal visual width
  -> wrapped visual row count
  -> max visual scroll
```

当 `CliState` 仍处于跟随尾部状态时，Transcript 使用最大视觉 scroll；用户手动上滚时则只做边界 clamp，不强行拉回底部。

### 7. UI 形态模仿的是交互原则，不是品牌外观

公开文档里 Codex CLI 的关键交互是 full-screen terminal UI、composer、实时 review action、inline approve / reject。Claude Code 的公开文档则强调状态行、fullscreen transcript viewer、完成通知、主题匹配和 prompt ergonomics。

因此本 demo 不再使用“System Online / Transcript / Prompt / Controls 全部大边框”的仪表盘式布局，而是改成：

```text
compact status line
  -> transcript-first reading area
  -> composer / approval gate
  -> quiet shortcut footer
```

这样更接近真实 coding agent 的使用动线：用户主要在读 agent transcript，安全审批只在需要时打断，状态和快捷键保持低噪声。

### 8. Assistant / thinking 文本使用 Markdown renderer

模型输出天然会包含 Markdown：列表、粗体、inline code、代码块、标题等。如果 terminal UI 直接按普通字符串显示，就会把 `**bold**`、反引号和列表结构原样暴露出来，阅读体验很差。

当前做法是只对两类模型文本启用 Markdown：

```text
UiLine::Thinking
UiLine::Model
```

系统事件、tool event、command event 和 approval event 仍然保持紧凑事件行。渲染使用 `tui-markdown`，并关闭默认代码高亮 feature，避免为 demo 引入 `syntect` 这类重依赖。

`tui-markdown` 当前没有启用 GFM table parse option，表格会被当成普通段落压成一行。因此 view 层对 Markdown table block 做了一个小兜底：检测 `| header |` + `|---|` 结构后，直接输出对齐后的终端行；其他 Markdown 仍交给 renderer。

### 9. CLI 进程内保留短期 messages memory

之前每次输入 prompt 都是一次新的 ReAct run，`ReActAgent::run(prompt)` 会从空 messages 开始。这不符合真实 agent CLI 的基本体验：同一个 CLI session 内，用户自然会连续追问。

当前做法是在 `ReActAgent` 内部维护一个短期 memory：

```text
ReActAgent
  -> Arc<Mutex<Vec<Value>>> memory
  -> run(prompt) starts from recent messages + current user message
  -> run completes, remember latest non-system messages
```

边界：

- 只保留当前进程内短期记忆，退出 CLI 即丢弃。
- 不写磁盘，不做长期 resume。
- 设置最近消息数量上限，避免 demo 对话无限增长。
- memory 复用同一套 OpenAI-compatible messages，不额外发明消息协议。

### 10. Runner 必须有 timeout，复杂 shell 语法必须 fail closed

一次真实 TUI 验证中，模型为了“用 Python 脚本计算”生成了：

```text
cat > /tmp/calc.py << 'EOF'
...
EOF
python /tmp/calc.py
```

这个命令暴露了两个问题：

- `OsExecutionRunner` 没有 timeout，子进程异常等待时 UI 会一直停在 running。
- `run_command` 仍用 `split_whitespace`，复杂 shell 命令可能因为第一个 token 是 `cat` 被误匹配成 `safe-read`。

当前修复：

- `OsExecutionRunner` 使用 `tokio::time::timeout` 包住 command output，并设置 `kill_on_drop(true)`。
- `run_command` 检测 heredoc、重定向、管道、命令串联等 shell control syntax；除明确建模的 `curl | sh` dangerous-shell 外，一律 fail closed，不进入 command execution。

这不是完整 shell parser，而是 demo 阶段必要的安全边界：不理解的复杂 shell 语义不能靠 prefix 白名单放行。

## Verification

确定性测试：

```bash
cargo test --manifest-path workspaces/02-learning/openai-codex-cli-deep-learning/topics/tools-permissions/demo/Cargo.toml -j 2
```

结果：

```text
90 passed; 0 failed; 3 ignored
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
- Transcript tail scroll 已改为按视觉行计算，长文本换行后也能自动露出最终 `turn completed`。
- UI 已从大面板仪表盘重写为 transcript-first：紧凑状态行、轻量 transcript、composer / approval gate 和低噪声 footer。
- Assistant / thinking 内容已接入 `tui-markdown`，支持基础 Markdown 渲染，并对 GFM table 做终端可读兜底。
- ReActAgent 已支持同一 CLI 进程内的短期 messages memory。
- `run_command` 已对复杂 shell syntax fail closed，`OsExecutionRunner` 已增加 timeout 防止 TUI 卡死。

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
