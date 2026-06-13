---
kind = "skill"
slug = "rust-async-streaming-agent"
status = "verified"
level = "draft"
target_level = "can-design-and-debug"
source = "workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/guides/08-demo-coder"
created_at = "2026-06-13"
---

# Rust 异步流式 Agent 实现

## 回忆钩子

如果想让 Agent 一边接收模型流、一边执行工具、一边更新 UI，就绕不开 Stream、channel、task 和 runtime。

## 现实问题

真实 Agent 不是同步函数调用。LLM 会流式返回 thinking/text/tool call delta，工具可能并发执行，审批结果可能从 UI 回来，TUI 还要持续刷新。Rust 中这些能力通常要用 async runtime、channel 和 Stream 串起来。

## 第一性原理

异步的本质是：任务在等待 I/O 或外部事件时让出执行权，runtime 在事件就绪后再唤醒任务。它解决的是“等待期间不要阻塞整个系统”。

## 底层原理

- `Future` 表示一次最终完成的异步计算。
- `Stream` 表示多次产出的异步序列，适合 LLM delta 和 Agent event。
- `tokio::sync::mpsc` 适合把后台任务产出的事件送到 UI 或外层 stream；bounded channel 可以形成背压，unbounded channel 更方便但更容易积压内存。
- `async move` 只是创建 future；`tokio::spawn(async move { ... })` 会把 future 交给 runtime 调度，spawn task 可能跨线程移动，因此通常要满足 `Send` 和 `'static` 边界。
- `join_all` / task join 可以并发等待多个工具结果，但要明确结果顺序和错误传播策略。
- OpenAI-compatible streaming tool call 不是一次性拿到完整参数，而是持续接收 delta，再聚合 tool name、call id 和 arguments。

## 关键不变量

- 不要在 async 任务里做长时间阻塞操作。
- UI 事件流要能持续 drain，不能只有用户输入时才继续拉取。
- LLM streaming 需要增量合并 text、thinking 和 tool call arguments。
- 并发 tool call 的 observation 要按 call id/index 回灌，不能丢对应关系。

## 取舍

直接在一个 `try_stream!` 里写完整逻辑很直观，但语法提示差、代码容易膨胀。用 channel 把内部任务和外部 Stream 解耦，代码更清晰，但要管理任务生命周期和错误传播。

## 不要照搬

不要把所有 async 逻辑都 `tokio::spawn`。spawn 会引入 `'static`、所有权和取消边界；能在当前任务内并发等待的，不一定要交给 runtime 独立调度。

## 迁移方式

实现流式 Agent 时，可以先拆成三层：

```text
LLM stream parser -> Agent event producer -> UI/event consumer
```

每层用明确的 enum 表达事件，不要让 raw JSON、UI 状态和模型消息互相穿透。

## 证据来源

- [agent/react.rs](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/agent/react.rs)
- [agent/llm/openai.rs](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/agent/llm/openai.rs)
- [cli/event_loop.rs](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/cli/event_loop.rs)
- [Tokio runtime scheduling guide](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/guides/08-demo-coder/11-tokio-runtime-scheduling.md)
- [Rust Async Book: Streams](https://rust-lang.github.io/async-book/05_streams/01_chapter.html)
- [Tokio channels tutorial](https://tokio.rs/tokio/tutorial/channels)
- [Tokio spawning tutorial](https://tokio.rs/tokio/tutorial/spawning)
- [Tokio mpsc docs](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html)
- [OpenAI function calling streaming](https://developers.openai.com/api/docs/guides/function-calling)

## 复习练习

不用看代码，解释 `async move` 和 `tokio::spawn(async move)` 的区别。然后设计一个函数，把 `mpsc::Receiver<StreamEvent>` 转成 `impl Stream<Item = StreamEvent>`。

## 薄弱点

需要继续补强：waker 与 OS 多路复用的关系、task cancellation、spawn 后错误处理、bounded channel 背压。
