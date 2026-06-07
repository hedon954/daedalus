# Slice 9 Multi-Tool Independent Policy

> Status: policy implemented in first version. See [`09-slice-9-parallel-run-tools-review.md`](09-slice-9-parallel-run-tools-review.md) for review and current refactor target.

## Learning Navigation

- Final artifact: `demo/README.md`
- Current stage: `08-demo-coder`
- Current slice: Slice 9 Multi-Tool Independent Execution
- Current gap: 调度语义已定稿并落地第一版；下一步是结构收口。
- After this: 抽出 batch boundary，让 ReAct loop 回到轮次和 transcript 职责。

## First-Principles Framing

同一批 `tool_calls` 一般不应该有值依赖：模型在同一个 assistant response 中已经给出了每个 call 的 arguments，后一个 call 不能等前一个 call 的输出再决定参数。

但这不等于完全没有依赖。它们可能存在：

- 副作用依赖：一个工具写文件、安装依赖、改变环境，另一个工具读取或执行后续动作。
- 安全依赖：一个工具被拒绝，可能说明模型的当前计划有风险。
- 协议依赖：每个 `tool_call_id` 最终都需要对应一个 tool observation。

因此多工具策略不是简单“失败就全部停”或“永远全并发”，而是要回答：

```text
现实目标：让 agent 尽快获得完整 observation，并能一次性纠错。
约束：不要破坏 tool_call_id -> tool result 的协议完整性。
设计选择：同批 tool calls 互不影响；允许并发执行；每个 call 独立返回结果。
代价：有副作用的工具需要额外并发能力声明，否则并发可能放大环境竞争。
```

## Source Calibration

Codex 本地源码体现的是 capability-aware parallelism，而不是 batch-level fail-fast：

- `session/turn.rs` 使用 `FuturesOrdered` 收集 in-flight tool futures，sampling 完成后统一 drain 回 history。
- `stream_events_utils.rs` 在收到 completed tool item 后记录 tool call，再创建 runtime future。
- `tools/parallel.rs` 根据 `router.tool_supports_parallel(&call)` 选择读锁或写锁：支持并发的工具可以并行，不支持并发的工具会串行化。
- tool 失败通常变成模型可见的 failure output；没有看到“一个失败后取消同批剩余 tool call”的主路径。

公开资料层面：

- Cursor 的公开技术报告只确认 agent action 可以包含多个 tool calls，并鼓励高效/并行行为；没有公开同批失败传播策略。
- Claude Code 公开文档强调权限与 hooks，未公开同一 assistant response 内多个 tool call 的失败传播调度细节。

结论：demo 不应把“hard-deny 后跳过后续”当作来自 Codex/Cursor/Claude Code 的事实。我们需要从自身 demo 目标出发决策。

## Decision

Slice 9 采用 independent observation policy：

```text
同批 tool_calls 视为互相独立
能并发的就并发执行
每个 tool_call 独立返回 Finished / Failed / Denied
不因为某一个 tool call 失败或被拒而取消其他 tool call
下一轮让 agent 基于完整 observation 一次性修正
```

实现要求：

- 执行可以并发，但写回 `messages` 的 tool observations 必须按原始 `index` 排序，保证 transcript 和测试稳定。
- 每个 `tool_call_id` 都必须得到 observation，包括 `Failed` 和 `Denied`。
- `Denied` 是当前 call 的结果，不传播成 batch-level hard stop。
- `ToolRuntimeResult::Skipped` 当前已删除；后续只有出现显式 scheduler skip 场景时才重新引入。

## Why This Beats Hard-Deny Skipped For This Demo

hard-deny skipped 的优点是节省资源、风险保守；但它有一个体验问题：agent 下一轮纠正后，还得重新请求那些被强制跳过的工具。如果这些工具本身也失败，就会产生多轮串行纠错。

independent observation 的优点是：

- agent 一次看到多个失败点，可以一次性修复。
- 协议更自然：每个 call 都由自己的 runtime result 负责。
- 测试更清晰：不需要制造 skipped observation 和 batch state。
- 更贴近 Codex 的能力模型：并发由 tool capability 控制，失败作为 observation 交给模型。

## Implementation Shape

```text
run_tools(tool_calls)
  -> sort by index
  -> create execution futures for calls that support parallel execution
  -> collect ToolRuntimeResult with original call metadata
  -> sort results by index
  -> append role=tool observations in stable order
  -> next LLM turn
```

第一版已经保持所有当前 demo tools 可并发，因为 `add/sub` 是纯函数，`run_command` 使用 simulated execution runner。随后再加 `supports_parallel_tool_calls`，让 mutating / shell-like tool 可以声明串行。

## Acceptance Tests

- 同一轮 `add` 和 `sub` 都成功：两条 observations 按 index 写回。
- 同一轮一个 unknown tool 失败、另一个 pure tool 成功：成功工具不受失败工具影响。
- 同一轮 `run_command` 被 `Denied`、另一个 pure tool 成功：两个 observations 都写回，且不会生成 skipped。
- 如果执行层并发，最终 message 顺序仍按原始 index，而不是完成先后。
- 当前不保留 `ToolRuntimeResult::Skipped`；测试应继续证明 batch-level hard-deny 不会自动产生 skipped observation。

## Open Questions

- `run_command` 是否默认支持 parallel？Phase 1 simulated runner 可以支持；Phase 2 `OsExecutionRunner` 前需要重新评估。
- 是否需要显式 `ToolDefinition::supports_parallel`？当前第一版已经并发执行；Phase 2 前这个字段会成为自然扩展点。
