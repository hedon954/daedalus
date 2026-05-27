# ReAct Loop Implementation Issues

本文记录从开始实现 [`react.rs`](../../demo/src/agent/react.rs) 到当前 live test 跑通期间遇到的问题。它不是最终设计定稿，而是用户实践中的问题清单和校准记录。

## Navigation

- Stage: `08-demo-coder`
- Slice: Slice 6 Agent Orchestrator
- Artifact advanced: `demo/src/agent/react.rs`、`demo/src/agent/stream_event.rs`、后续 `demo/README.md`
- Current validation: live LLM test 已跑通一次多轮 tool call，但仍缺 deterministic fake LLM tests。

## 1. `yield syntax is experimental`

### User problem

用户在实现 `react.rs` 时遇到：

```text
yield syntax is experimental
```

### Calibration

Rust 原生 `yield` 仍是实验语法。稳定 Rust 中可以通过 `async_stream::try_stream!` 这类宏使用 `yield`，但 `yield` 必须出现在宏内部。普通 `async fn`、普通 `loop` 或 helper function 里直接写 `yield` 都会触发该错误。

### Design conclusion

如果采用 `try_stream!`：

```rust
let stream = async_stream::try_stream! {
    yield StreamEvent::Started;
};
```

但 `try_stream!` 适合薄外壳，不适合承载完整 agent 状态机。

## 2. `try_stream!` 里的语法提示太差

### User problem

用户发现：把 ReAct loop 写进 `try_stream!` 后，rust-analyzer 的补全、类型提示和错误定位都变差。

### Calibration

这是宏方案的自然代价。`try_stream!` 会把代码展开成状态机，IDE 很难像普通 Rust 函数一样理解内部控制流。

### Design conclusion

第一版思路是把复杂逻辑拆进普通 helper：

```text
try_stream!
  -> run_one_turn(...)
  -> yield returned events
```

但这个方案会牺牲实时性，因为 `run_one_turn` 必须先收集完整一轮结果再返回。

## 3. 实时 yield 与普通函数可维护性冲突

### User problem

用户指出：如果 `run_one_turn` 返回 `Vec<StreamEvent>`，就不能提前 `yield`，无法做到 LLM delta 或 tool result 产生后立刻对外可见。

### Calibration

这是一个真实架构张力：

```text
Vec<StreamEvent> 方案
  -> 普通函数好写
  -> 但事件不实时

try_stream! 内完整 loop
  -> 事件实时
  -> 但宏内代码可维护性差
```

### Current conclusion

改用 `tokio::sync::mpsc` channel：

```text
run()
  -> 创建 tx/rx
  -> tokio::spawn(run_agent_loop(..., tx))
  -> 把 rx 包装成 EventStream 返回
```

这样 `run_agent_loop` 是普通 `async fn`，语法提示正常；同时每个事件可以通过 `tx.send(...)` 立即对外发出。

## 4. `tokio::spawn` + `stream::unfold` 的含义

### User problem

用户追问这段代码的含义：

```rust
tokio::spawn(async move {
    if let Err(err) = run_agent_loop(llm, prompt, tx.clone()).await {
        let _ = tx.send(Err(err)).await;
    }
});

let stream = stream::unfold(rx, |mut rx| async {
    rx.recv().await.map(|event| (event, rx))
});
```

### Calibration

这段代码把普通 async loop 转成 `EventStream`：

```text
tokio::spawn
  -> 后台生产事件

mpsc::Sender
  -> run_agent_loop 每产生一个事件就发送

mpsc::Receiver
  -> 外部 stream 消费事件

stream::unfold
  -> 把 receiver 包装成 futures Stream
```

如果 `run_agent_loop` 出错，后台任务会把 `Err(err)` 发送到 stream。若外部消费者已经 drop 掉 receiver，发送失败被忽略，不再 panic。

## 5. 一个 LLM response 里多个 tool call 怎么处理

### User problem

用户遇到：同一轮 LLM response 可能产生多个 tool call，`run_tool` 应该怎么调用。

### Calibration

不能看到一个 tool call delta 就执行，因为 streaming arguments 可能还没完整。执行点应该是 `ToolCallFinished`。

同一轮里多个 tool call 的协议顺序是：

```text
collect all ToolCallFinished
  -> append assistant message with tool_calls
  -> run tools
  -> append role=tool observations
  -> next model turn
```

### Current conclusion

当前实现采用“本轮收集，按 index 排序，逐个执行，逐个回灌”。

后续可以改成：

```text
ToolCallFinished 后立即 spawn tool task
  -> 等本轮 Completed
  -> 按 index 收集结果
  -> append tool observations
```

但并发 tool execution 只适合无副作用或互不依赖的工具。进入命令执行、文件写入、安装依赖后，需要重新考虑顺序、权限审批和副作用隔离。

## 6. `ToolCallFinished` 与 `ToolRunFinished` 不是同一个事件

### User problem

用户提出：每 run 一个 tool，应该吐一个 tool-result event 出去。

### Calibration

这是正确抽象。事件层至少要区分：

```text
ToolCallFinished
  -> 模型已经完整表达了想调用的工具和参数

ToolRunStarted
  -> runtime 开始实际执行工具

ToolRunFinished / ToolRunFailed
  -> runtime 拿到工具执行结果或失败
```

这能对齐 agent ReAct loop 的观察需求：模型决策、运行时执行、执行结果是三个不同事实。

### Current conclusion

`react.rs` 当前已经对外 emit `ToolRunStarted`、`ToolRunFinished`、`ToolRunFailed`。但 code review 发现 `ToolCallFinished` 被内部消费后没有继续 emit，后续应补上。

## 7. OpenAI-compatible message 回灌顺序

### User problem

实现过程中需要决定 tool result 怎么放回 `messages`。

### Calibration

OpenAI-compatible Chat Completions 的 tool call 回灌顺序应为：

```text
assistant message:
  role = assistant
  content = ...
  tool_calls = [...]

tool message 1:
  role = tool
  tool_call_id = ...
  content = ...

tool message 2:
  role = tool
  tool_call_id = ...
  content = ...
```

不能只追加 tool result，而不追加 assistant 的 `tool_calls` message。否则下一轮模型无法把 tool observation 和自己上一轮的 tool call 对齐。

### Current conclusion

当前 `react.rs` 已经按这个结构回灌 messages。

## 8. Live test 跑通，但不能替代验收测试

### User observation

当前 live test 输出证明：

```text
第一轮：
  add(999, 666) -> 1665
  sub(321, 123) -> 198

第二轮：
  add(1665, 198) -> 1863

第三轮：
  final answer
```

这说明主链路已经跑通：

```text
real LLM stream
  -> multiple tool calls in one turn
  -> tool result feedback
  -> next model turn
  -> final output
```

### Calibration

但 live test 依赖：

```text
DEEPSEEK_API_KEY
network
provider availability
model output stability
```

因此它只能作为手动验收，不应该作为默认 `cargo test` 的稳定测试。

### Next step

- 给 live test 加 `#[ignore = "requires DEEPSEEK_API_KEY and network"]`。
- 增加 fake LLM deterministic tests，覆盖：
  - 一轮多个 tool call。
  - 多轮 tool call。
  - tool failed 后 observation 回灌。
  - final answer without tool call。

## 9. 当前 code review 发现的待补点

### Must fix before Slice 6 done

- `max_turns`：防止模型不断 tool call 导致无限循环。
- `ToolCallFinished` emit：不要只内部消费模型决策事件。
- live LLM test 默认 ignore。
- fake LLM deterministic tests。

### Should improve soon

- `openai.rs` 的 tool call cache 从 `Vec` 改为 `BTreeMap<i64, ToolCallCache>`，避免依赖 index 连续且按序到达。
- assistant message 回灌时尽量只保留 OpenAI-compatible 标准字段：`role`、`content`、`tool_calls`；`reasoning_content` 不宜作为通用输入字段回灌。

## Transferable Pattern

实现 streaming ReAct loop 时，不要把问题简化成“如何返回一个 stream”。真正的设计问题是：

```text
如何在保持实时 event 输出的同时，
让 agent loop 主逻辑仍然是普通可维护代码，
并且严格区分 model decision、runtime execution、observation feedback。
```

当前 demo 的答案是：

```text
mpsc channel as event boundary
  -> run_agent_loop writes events
  -> receiver becomes EventStream
  -> model events and runtime events share one observable stream
```
