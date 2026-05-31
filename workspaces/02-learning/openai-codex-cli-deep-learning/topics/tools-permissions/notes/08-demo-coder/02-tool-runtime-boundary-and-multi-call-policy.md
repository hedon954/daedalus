# Tool Runtime Boundary And Multi-Call Policy

本文记录 Slice 6 中把 approval / sandbox / retry 接入 ReAct loop 前的一次设计决策过程。它的目的不是替代实现，而是把“为什么这么接”留下来，避免后面写 `react.rs` 时只剩局部代码选择。

## Navigation

- Stage: `08-demo-coder`
- Slice: Slice 6 Agent Orchestrator
- Artifact advanced: `demo/src/agent/react.rs`、`demo/src/tool/function.rs`、`demo/src/tool/runtime.rs`、`demo/src/tool/shell.rs`、`demo/README.md`
- Current gap: ReAct loop 已经能执行 tool call，但还没有恢复 Codex 的权限 / 沙箱 / retry 不变量。

## Problem

当前 `react.rs` 的工具执行路径仍然接近：

```text
ToolCallFinished
  -> tool::function::run_tool(name, arguments)
  -> ToolRunFinished / ToolRunFailed
  -> role=tool observation
```

这能验证 ReAct message 回灌，但还不能验证我们从 Codex 学到的核心安全链路：

```text
ToolCallFinished
  -> build runtime request
  -> match capability
  -> decide approval
  -> sandbox first attempt
  -> controlled retry
  -> observation feedback
```

因此本轮要解决的不是“怎么再加一个工具”，而是“工具调用必须经过什么 runtime 边界，才能既支持纯函数工具，也支持本地命令工具”。

## User Hypothesis

用户给出的初始判断：

1. `add/sub` 仍然作为纯工具存在，但也应该进入统一权限验证路径；它们当前的权限结果可以是 `Pass` / `Skip`。
2. 新增一个 `shell` 工具承载 command 能力，并让它走真实的 capability / approval / sandbox / retry 链路。
3. `CommandRequest` 组装可以抽到 `ToolRuntime`，避免 `react.rs` 承担过多权限细节。
4. 多 tool call 中如果某个工具被 forbidden 或 rejected，需要参考 Codex / Claude Code 的设计，再决定后续工具是否继续执行。

## Source Calibration: Codex

Codex 的设计可以抽象成三层：

```text
model stream item
  -> ToolRouter / HandleOutput
  -> ToolCallRuntime
  -> concrete tool handler / orchestrator
```

关键观察：

- `session/turn.rs` 中，模型返回的 completed output item 会被处理为 tool future，并放入 `FuturesOrdered`，之后统一 drain 回 history。
- `stream_events_utils.rs` 中，tool call 会先被记录进 conversation history，再创建 runtime future；这保证模型上一轮的 tool call 和下一轮的 tool output 能对齐。
- `tools/parallel.rs` 中，是否并行不是 ReAct loop 任意决定，而是由工具声明 `supports_parallel_tool_calls`。不支持并行的工具会经由锁串行化。
- `tools/orchestrator.rs` 中，shell-like 工具的审批、沙箱选择、sandbox denied 后 retry 都在 orchestrator 内部完成，而不是散落在 agent loop 中。
- Forbidden / rejected / 普通工具失败通常会成为模型可见的 failure observation，让下一轮模型可以自我修复；fatal error 才中断 turn。

对应源码入口：

- [`session/turn.rs`](../../../../source/codex/codex-rs/core/src/session/turn.rs)：看 `try_run_sampling_request` 如何把 completed tool item 放入 in-flight futures，并在 sampling 完成后 drain 回 history。
- [`stream_events_utils.rs`](../../../../source/codex/codex-rs/core/src/stream_events_utils.rs)：看 `handle_output_item_done` 如何先记录 tool call，再创建 runtime future。
- [`tools/parallel.rs`](../../../../source/codex/codex-rs/core/src/tools/parallel.rs)：看 `ToolCallRuntime` 如何根据 `supports_parallel_tool_calls` 决定并行或串行。
- [`tools/orchestrator.rs`](../../../../source/codex/codex-rs/core/src/tools/orchestrator.rs)：看 approval、sandbox first attempt、sandbox denied retry 的主状态机。
- [`tools/handlers/shell.rs`](../../../../source/codex/codex-rs/core/src/tools/handlers/shell.rs)：看 shell handler 如何组装 `ShellRequest` 并交给 orchestrator。
- [`tools/runtimes/shell.rs`](../../../../source/codex/codex-rs/core/src/tools/runtimes/shell.rs)：看 `ShellRuntime` 如何定义 approval key、first-attempt sandbox override 和实际执行。

## Source Calibration: Claude Code

Claude Code 官方文档能确认的点：

- 权限规则包含 `allow` / `ask` / `deny`。
- `PreToolUse` hook 可以返回 `allow` / `deny` / `ask`，其中 `deny` 会阻止工具执行，并把原因反馈给 Claude。
- Claude 模型层支持 parallel tool use，也可以通过配置禁用并行工具使用。

但官方文档没有公开到 Codex 源码这种 runtime 细节。因此，不能把“同一轮多个 tool call 中，一个被拒后后续是否继续执行”当成 Claude Code 的已验证实现事实。这里 Claude Code 只能作为产品权限模型参考，不能作为实现级证据。

参考：

- <https://docs.anthropic.com/en/docs/claude-code/settings>
- <https://docs.anthropic.com/en/docs/claude-code/iam>
- <https://docs.anthropic.com/en/docs/claude-code/hooks>
- <https://docs.anthropic.com/es/docs/agents-and-tools/tool-use/implement-tool-use>

## Design Decisions

### Decision 1: 所有工具进入统一 runtime 边界

`add/sub` 不应该绕过 runtime。它们虽然不是本地命令，也没有沙箱意义，但仍然可以通过统一 runtime 得到：

```text
ToolCallFinished
  -> ToolRuntime
  -> pure tool approval pass
  -> run pure function
  -> ToolRunFinished
```

这样做的价值是：ReAct loop 不需要知道“哪些工具安全、哪些工具危险”。它只负责把 tool call 交给 runtime；runtime 再根据工具类型决定是否需要 command request、approval、sandbox 和 retry。

### Decision 2: 新增命令型工具承载本地命令安全链路

`shell` / `run_command` 是真正验证 Codex 权限模型的工具。它需要把模型参数转成 `CommandRequest`，并走完整链路：

```text
shell tool call
  -> parse argv / cwd
  -> match capability
  -> decide_approval
  -> sandbox first attempt
  -> decide_retry if SandboxDenied
  -> optional no-sandbox retry
  -> observation
```

不要把 `add/sub` 强行改造成命令工具。纯函数工具和本地命令工具都可以共用 runtime 边界，但内部执行协议不同。

### Decision 3: `react.rs` 不直接组装完整权限链路

`react.rs` 应该保留 agent loop 的职责：

```text
LLM stream
  -> collect tool calls
  -> append assistant tool_calls message
  -> ask ToolRuntime to execute
  -> append tool observations
  -> next LLM turn
```

权限、沙箱、retry 的细节应放到 `ToolRuntime` 或等价 helper。这样 demo 会更接近 Codex 的分层：agent loop 负责 turn orchestration，tool runtime 负责 tool execution policy。

### Decision 4: Phase 1 多 tool call 先保持顺序执行

虽然 Codex 支持根据工具声明并行执行，但 Phase 1 demo 的目标不是并发调度，而是把安全链路跑通。为了确定性和可测试性，Phase 1 先按 `index` 顺序执行 tool calls。

并发执行可以作为后续增强，但前提是工具声明自己是否支持并行，并且 mutating / shell-like 工具有明确串行化策略。

### Decision 5: 被拒后停止真实执行后续工具，但补齐 skipped observation

OpenAI-compatible Chat Completions 中，如果 assistant message 包含多个 `tool_calls`，下一轮通常需要为这些 `tool_call_id` 都提供 tool message。否则上一轮 assistant 的 tool call 和下一轮 tool output 容易不匹配。

因此 Phase 1 推荐策略是：

```text
for tool_call in sorted_tool_calls:
  if previous call caused hard denial:
    append skipped observation for this tool_call_id
    continue

  result = ToolRuntime.run(tool_call)

  if result is forbidden / approval rejected:
    append denial observation for current tool_call_id
    mark subsequent calls as skipped
```

也就是说：

- 安全上：拒绝后不继续真实执行后续工具。
- 协议上：仍然为后续 tool call 补充 `role=tool` message。
- 学习上：模型下一轮可以看到“为什么没执行”，并重新规划。

## Resulting Target Shape

下一步实现目标可以收敛为：

```text
agent/react.rs
  -> ToolRuntime::run(call)

tool/runtime.rs
  -> pure tool path
  -> command tool path

pure tool path
  -> approval pass
  -> tool/function.rs run add/sub

command tool path
  -> tool/shell.rs parse argv/cwd
  -> CommandRequest
  -> decide_approval
  -> SimulatedSandboxRunner
  -> decide_retry
```

测试优先覆盖：

- pure tool 仍能正常执行。
- shell safe command sandbox success。
- shell dangerous command forbidden，runner 不执行。
- sandbox denied + retry approved -> no-sandbox success。
- command failed -> no retry。
- 多 tool call 中一个被拒后，后续 call 不真实执行，但有 skipped observation。

## Open Question

Phase 1 是否需要把 `ToolRunSkipped` 做成显式 `StreamEvent`？

最小路径可以先复用 `ToolRunFailed`，但从可观察性看，`Skipped` 与 `Failed` 不是同一个事实：

```text
Failed
  -> 工具尝试执行了，但失败

Skipped
  -> runtime 因安全策略没有执行
```

如果事件模型已有足够空间，建议补一个 `ToolRunSkipped`，否则至少在 `ToolRunFailed.error` 里明确 `skipped because previous tool was denied`。
