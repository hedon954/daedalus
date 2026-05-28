# Slice 6 Hardening Guide

这份 guide 服务下一轮 `08-demo-coder`：在 `react.rs` 已经 live 跑通真实 LLM 多轮 tool call 后，把 Agent Orchestrator 收敛成可验收、可复习、可迁移的 Phase 1 主链路。

## Learning Navigation

- Final artifact: [`../../demo/README.md`](../../demo/README.md)
- Current stage: `08-demo-coder`
- Current slice: Slice 6 Agent Orchestrator
- Current gap: ReAct hardening 已完成；下一步进入 approval / sandbox / retry integration
- Evidence already available: [`../../notes/08-demo-coder/react-loop-implementation-issues.md`](../../notes/08-demo-coder/react-loop-implementation-issues.md)
- After this: 可以把 approval / sandbox / retry 接入 ReAct loop，然后进入 Slice 7 README / Runbook

## North Star

下一轮不是继续“让 demo 更聪明”，而是让已经跑通的 ReAct loop 具备最小工程边界：

```text
real/fake LLM stream
  -> tool call decision event
  -> tool runtime event
  -> observation feedback
  -> bounded next turn
```

## Action Card

### 1. Add `max_turns`

目标：防止模型持续 tool call 时无限循环。

建议先做最小设计：

```text
ReActAgent {
  llm,
  max_turns,
}
```

验收：

- fake LLM 连续返回 tool call 超过上限时，stream 输出错误或返回 `Err`。
- 正常 2-3 轮 tool call 不被误杀。

### 2. Emit `ToolCallFinished`

目标：让模型决策和 runtime 执行都可观察。

保留这个事件边界的理由：

- 解释未执行的 tool call：模型可能提出了调用，但被参数解析、capability、approval、用户拒绝或 `max_turns` 拦截。
- 对齐 approval 输入：approval 判断发生在“模型提出 tool call”之后、“runtime 真实执行”之前。
- 验证 streaming parser：`ToolCallFinished` 能单独证明分片 arguments 已聚合成完整 tool call。
- 区分决策和副作用：它表达 model tool choice；`ToolRunStarted/Finished/Failed` 才表达真实执行。

普通用户视图可以只展示 runtime 事件；debug / audit / learning 视图建议保留 `ToolCallFinished`。

当前问题：

```text
ToolCallFinished
  -> 被 react.rs 收进 pending_tool_calls
  -> 但没有继续发给外部 stream
```

期望事件顺序：

```text
ToolCallFinished
ToolRunStarted
ToolRunFinished / ToolRunFailed
```

验收：

- fake LLM 单测能观察到 `ToolCallFinished`。
- `ToolCallFinished` 不替代 `ToolRunFinished`。

### 3. Add Fake LLM Tests

目标：让 Slice 6 不依赖真实 API 也能验证主状态机。

建议覆盖：

- 一轮多个 tool call。
- 多轮 tool call。
- tool failed 后以 `role=tool` observation 回灌。
- final answer without tool call。
- max turns exceeded。

注意：

- live LLM test 保持 `#[ignore]`。
- fake LLM 可以直接返回预设 `StreamEvent` 序列，不需要模拟 SSE。

### 4. Add Parser Fixture Tests

目标：锁定 OpenAI-compatible streaming adapter 的协议边界。

建议覆盖：

- `data: {...}\n\n` 正常切块。
- `[DONE]` 被忽略或终止。
- 单个 chunk 内多个 `tool_calls`。
- tool call arguments 分多片 delta 聚合。

### 5. Decide What To Defer

本轮不要做：

- 真正 OS sandbox。
- 真实命令审批 UI。
- 并发执行带副作用的 tool。
- 多 provider message 抽象。

这些属于 Phase 1 后半段或 Phase 2，不要抢当前 slice 的注意力。

## Completion Criteria

- [x] `cargo test` 默认不访问网络。
- [x] fake LLM tests 覆盖 ReAct loop 关键路径。
- [x] live LLM test 仍可手动运行。
- [x] event stream 同时能观察 model decision 和 runtime execution。
- [x] `.daedalus/outcome-map.md` 和 `.daedalus/todo.md` 同步 Slice 6 状态。
