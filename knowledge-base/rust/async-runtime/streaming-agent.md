---
status = "verified"
source = "workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions"
---

# Rust 流式 Agent 与 Tokio 运行时

这次 demo 里，Rust async 不是旁路知识，而是 Agent 能力的基础设施：LLM streaming、tool event、approval response、TUI 刷新都依赖异步任务和流式事件。

## 从第一性原理看

异步解决的问题不是“让代码更快”，而是：

```text
当任务在等待 I/O、网络、用户输入或子进程时，不要阻塞整个系统
```

Rust async 的基本抽象是 `Future`：一个可能暂时无法完成、需要之后再被 poll 的计算。`Stream` 则类似异步版 Iterator，可以在完成前多次产出值。Rust Async Book 对 Stream 的描述正好对应 LLM token delta 和 Agent event stream。

在 Agent 中，流式输出天然是多次产出：

- LLM text delta。
- reasoning / thinking delta。
- tool call delta。
- tool started / finished。
- approval request / approval result。
- final answer。

## 底层原理

Tokio runtime 负责调度 async task。任务等待 I/O 时返回 pending，并把 waker 注册给 runtime；当 socket、timer、channel 或其它事件源就绪时，runtime 再唤醒任务继续 poll。

`mpsc` channel 是这次 demo 中最关键的组合工具。Tokio 文档把它定义为多生产者、单消费者队列；bounded channel 还提供 backpressure。对 Agent 来说，它可以把内部复杂任务拆成：

```text
后台任务负责跑 Agent loop
channel 负责把事件送到外部 Stream / TUI
外部消费者按自己的节奏读取事件
```

这比把所有逻辑塞进一个 `try_stream!` 更清晰。`try_stream!` 很适合小逻辑，但在 ReAct loop 里会让语法提示差、错误处理和生命周期管理都变复杂。

## `async move` 和 `tokio::spawn(async move)`

这次并发 tool calls 时最容易混淆的是：

```text
async move 只是创建一个捕获变量的 Future
tokio::spawn(async move { ... }) 会把 Future 交给 runtime 独立调度
```

`async move` 不要求 `'static`，因为它可以仍然被当前函数拥有和 await。

`tokio::spawn` 通常要求 future 满足 `Send + 'static`，因为任务被放进 runtime 后，可能在当前调用栈结束后继续运行，也可能在线程间移动。Tokio 官方教程也强调，spawn 出来的 task 必须满足 `Send`，让 runtime 可以在 `.await` 挂起点之间移动任务。

这解释了为什么在 demo 里直接 `spawn` 会碰到借用 `self`、引用生命周期和 JoinHandle 错误。解决方向不是硬凑生命周期，而是决定任务到底需不需要脱离当前作用域：

- 只需要并发等待：可以用 `join_all` / `FuturesUnordered`。
- 需要后台长期运行：用 `spawn`，并把数据改成 owned / `Arc`。
- 需要流式向外吐事件：常用 `spawn + mpsc channel`。

## OpenAI 兼容流式协议

OpenAI streaming 文档说明，HTTP streaming 使用 Server-Sent Events。实现时不要把 response 一次性读完，而要逐 chunk / line 处理 `data:` 事件。

这次 demo 的关键经验是：

- 原始 stream 要先能 debug 打印。
- SSE line parsing 和 JSON parsing 要分层。
- delta 要合并，尤其是 text、thinking、tool call arguments。
- tool call 完整后再交给 runtime 执行。
- 流式事件要进入统一的 Agent event stream，不能只 print。

## 在 Codex demo 中的体现

- [`agent/llm/openai.rs`](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/agent/llm/openai.rs)：解析 OpenAI-compatible streaming response。
- [`agent/react.rs`](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/agent/react.rs)：把 LLM stream、tool calls 和 observations 串成 ReAct loop。
- [`tool/runtime.rs`](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/tool/runtime.rs)：并发执行多个 tool call。
- [`cli/event_loop.rs`](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/cli/event_loop.rs)：把 Agent stream 和 UI event loop 对接。

## 关联

- [ReAct 工具运行时](../../ai-agents/tool-use/react-tool-runtime.md)
- 外部资料：
  - [Rust Async Book](https://rust-lang.github.io/async-book/)
  - [Rust Async Book: Streams](https://rust-lang.github.io/async-book/05_streams/01_chapter.html)
  - [Tokio `spawn`](https://docs.rs/tokio/latest/tokio/task/fn.spawn.html)
  - [Tokio spawning tutorial](https://tokio.rs/tokio/tutorial/spawning)
  - [Tokio mpsc](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html)
  - [OpenAI Streaming Responses](https://developers.openai.com/api/docs/guides/streaming-responses)
