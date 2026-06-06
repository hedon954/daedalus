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

### Decision 4: 多 tool call 采用能力约束下的并发执行

本节已被 Slice 9 的最新决策更新：Phase 1 不再把多 tool call 固定为顺序执行。当前目标是让同批 tool calls 在工具能力允许时并发执行，但最终写回 `messages` 时仍按原始 `index` 排序，保证 transcript 稳定。

这比纯顺序执行更贴近模型一次返回多个 tool calls 的语义：这些 call 的参数已经全部确定，默认应该作为独立 observation 收集。真正需要串行化的是 mutating / shell-like tool 的副作用边界，而不是 batch 本身。

### Decision 5: 单个 tool 失败或被拒不传播为 batch-level skip

本节也已被 Slice 9 的最新决策更新：不采用“一个安全拒绝后，后续 tool call 全部 skipped”的策略。

当前策略是：

```text
same-batch tool_calls
  -> run independently, preferably concurrently when supported
  -> each call returns Finished / Failed / Denied
  -> collect all results
  -> append observations by original index
  -> next LLM turn repairs with complete evidence
```

也就是说：

- 单个 `Denied` 只说明当前 call 被拒，不自动取消后续 call。
- 单个 `Failed` 只说明当前 call 执行失败，不自动取消后续 call。
- 协议上：每个 `tool_call_id` 仍然必须有自己的 `role=tool` observation。
- 学习上：模型下一轮一次性看到完整成功/失败/拒绝证据，减少多轮串行纠错。

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
- 多 tool call 中一个失败或被拒，不影响其他 call 执行；最终 observations 按原始 index 写回。

## Open Question

Phase 1 是否还需要 `ToolRuntimeResult::Skipped`？

在当前 independent observation policy 下，`Skipped` 不再是 multi-tool hard-deny 的主路径。它只有在未来出现明确“调度器主动不运行某个 call，但仍要回灌 observation”的场景时才有价值。

```text
Skipped
  -> 调度器明确选择不运行某个 call
  -> 但这个选择不是由同批其他 call 的失败自动传播而来
```

如果 Slice 9 实现后仍找不到真实生产路径，可以删除 `ToolRuntimeResult::Skipped`，减少状态枚举噪声。
