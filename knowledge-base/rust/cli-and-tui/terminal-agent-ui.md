---
status = "verified"
source = "workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions"
---

# 终端 Agent UI 的事件循环

这次 demo 的 TUI 学习不是“怎么把界面画漂亮”，而是理解终端 Agent UI 的底层结构：用户输入和 Agent 流式事件都要进入同一个事件循环，再归约成状态，最后渲染。

```text
key event + agent stream event
-> event loop
-> app state
-> draw
```

## 从第一性原理看

终端 UI 没有浏览器 DOM，也没有系统自动帮你维护组件状态。它更接近一个循环：

1. 读取输入或外部事件。
2. 更新应用状态。
3. 根据当前状态重绘屏幕。

Ratatui 官方 FAQ 明确说明：Ratatui 不处理输入；用户自己维护 event loop、application state，并在每次迭代重绘 UI。输入通常由 Crossterm 这类库提供。

Crossterm 的 `read` 会阻塞直到有事件；`poll` 可以先检查事件是否可读，避免阻塞。这解释了 demo 早期遇到的问题：如果 UI loop 只在用户输入后才继续处理，就会导致 Agent 事件没有持续刷新。

## 底层原理

Agent TUI 至少有三类事件源：

- 用户输入：字符、Enter、Backspace、滚动、approval 选择。
- Agent stream：thinking、assistant text、tool event、approval request、execution event。
- terminal event：resize、tick、redraw。

如果这些事件不进入统一循环，就容易出现：

- Agent 卡住，必须敲键才继续刷新。
- UI state 和 Agent state 不一致。
- approval request 出来了，但输入模式没有切换。
- text delta 和 thinking delta 不能合并。
- transcript 滚动和新事件追加互相打架。

所以 demo 的分层是：

- `event_loop.rs`：合并 key event 和 Agent stream event。
- `app.rs`：维护 transcript、input、approval pending、running state、scroll offset。
- `view.rs`：只负责根据 state 画 header、transcript、prompt/approval、footer。

## 在 Codex demo 中的体现

相关实现：

- [`cli/event_loop.rs`](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/cli/event_loop.rs)
- [`cli/app.rs`](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/cli/app.rs)
- [`cli/view.rs`](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/cli/view.rs)
- [Slice 13 TUI closeout note](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/notes/08-demo-coder/13-slice-13-codex-like-cli-closeout.md)

这次学到的关键不是 Ratatui API，而是 UI 架构边界：

```text
Agent 不直接 print
UI 不直接跑工具
事件循环负责汇合
状态层负责归约
渲染层负责表达
```

## 现实工程取舍

这种分层比直接 print 麻烦，但它带来几个能力：

- 支持流式输出。
- 支持审批交互。
- 支持滚动和历史 transcript。
- 支持 Agent running/ready 状态。
- 支持以后替换渲染层或接入更完整的 CLI。

如果只是脚本工具，直接 stdout 足够。如果是 coding agent CLI，事件循环几乎不可避免，因为用户输入、模型输出、工具执行和审批结果都在竞争同一个终端界面。

## 关联

- [Rust 流式 Agent 与 Tokio 运行时](../async-runtime/streaming-agent.md)
- [ReAct 工具运行时](../../ai-agents/tool-use/react-tool-runtime.md)
- 外部资料：
  - [Ratatui FAQ](https://ratatui.rs/faq/)
  - [Crossterm event module](https://docs.rs/crossterm/latest/crossterm/event/index.html)
  - [Crossterm `read`](https://docs.rs/crossterm/latest/crossterm/event/fn.read.html)
