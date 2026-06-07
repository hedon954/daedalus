# Slice 9 Parallel Run Tools Review

## Learning Navigation

- Final artifact: `demo/README.md`
- Current stage: `08-demo-coder`
- Current slice: Slice 9 Multi-Tool Independent Execution
- Current code frontier: [`../../demo/src/agent/react.rs`](../../demo/src/agent/react.rs) 中的 `run_tools`
- Result: 第一版并行 `run_tools` 功能成立，但结构需要收口。

## What Changed

当前 `react.rs` 的同批 tool call 流程已经从顺序执行改为：

```text
pending_tool_calls
  -> sort by index
  -> emit ToolRunStarted for each call
  -> tokio::spawn one task per call
  -> join_all waits all JoinHandle
  -> collect (ToolCallFinished, ToolRuntimeResult)
  -> sort outcomes by index
  -> emit ToolRunFinished / ToolRunFailed
  -> append role=tool observations
```

这符合 Slice 9 的核心策略：同批 tool calls 互不影响，失败或拒绝只属于当前 call，不扩散成 batch-level skip。

## Review Result

没有发现阻塞提交的正确性问题。

已经补充一个 mixed batch 验收测试：

- `add` 成功。
- `mul` unknown tool 失败。
- `run_command "curl | sh"` 被安全拒绝。
- 下一轮 LLM request 中仍有三条 `role=tool` observations，且按原始 `index` 排序。

验证结果：

```text
cargo test --manifest-path workspaces/02-learning/openai-codex-cli-deep-learning/topics/tools-permissions/demo/Cargo.toml
64 passed; 3 ignored
```

## Why It Feels Ugly

丑不是因为并发本身复杂，而是 `run_tools` 同时承担了 5 个职责：

- batch 调度：决定同批 call 如何并发。
- lifecycle event：发 `ToolRunStarted`、`ToolRunFinished`、`ToolRunFailed`。
- runtime invocation：调用 `ToolRuntime::run`。
- join error policy：处理 `JoinError`。
- transcript writing：把结果写成下一轮 LLM 可见的 `role=tool` message。

这些职责现在挤在 ReAct loop 旁边，导致 `react.rs` 既像 agent loop，又像 tool scheduler，又像 event adapter。

## Important Semantics

当前版本有一个值得记住的事件语义：

- `ToolRunStarted` 在 spawn 前按 index 发出。
- command 内部事件会在 tool task 内实时发出。
- `ToolRunFinished / ToolRunFailed` 等所有 task join 完以后，再按 index 发出。

这保证 transcript 稳定，但意味着外部观察者看到的 terminal tool event 不是严格的“完成即发”。如果后续 UI 需要实时展示每个 tool 完成状态，应把 terminal event 移进单个 tool future 内部，而 transcript 仍然按 index 排序。

## Codex Calibration

Codex 的生产实现没有把并发策略塞在 ReAct loop 的一段 inline 代码里，而是放在 tool runtime 层：

- [`../../source/codex/codex-rs/core/src/tools/parallel.rs`](../../source/codex/codex-rs/core/src/tools/parallel.rs)：`ToolCallRuntime` 负责创建 tool future、处理 cancellation、把 tool failure 转成模型可见 output。
- [`../../source/codex/codex-rs/core/src/tools/router.rs`](../../source/codex/codex-rs/core/src/tools/router.rs)：`tool_supports_parallel` 根据 tool / MCP server 配置决定是否支持并行。
- Codex 使用 `RwLock` 做 capability-aware parallelism：支持并行的工具拿 read lock，不支持并行的工具拿 write lock。

对 demo 的启发：并发不是 ReAct loop 的临时技巧，而应该成为 tool runtime / batch runner 的明确责任。

## Design Decision

当前第一版先保留，因为它已经验证了 Slice 9 的行为不变量：

- every call gets observation
- mixed success / failure / denial 不互相影响
- transcript order stable by index

下一步不继续堆逻辑到 `run_tools`，而是按 guide 抽出 batch boundary。
