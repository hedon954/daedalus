# Outcome Map

Outcome Map 是当前 Codex 学习任务的导航仪表盘。它回答：最终要交付什么、当前在哪里、正在补哪个缺口、哪些源码细节停止扩展。

## North Star

- 最终要获得的能力：读懂生产级 Agent CLI 如何把一次用户请求安全地推进到 agent loop、上下文管理、工具调用、权限审批、沙箱执行和结果回流，并能把这些设计迁移到自己的 Agent/CLI 系统。
- 最小可验证 demo：实现一个最小 Agent CLI，保留 `CommandRequest -> ApprovalRequirement -> sandbox first execution -> controlled escalation/retry -> tool result` 的核心权限/沙箱不变量。
- 现实问题中的迁移目标：为自有 Agent/CLI 设计一套不会把本地命令执行简化成字符串黑白名单的权限与沙箱策略。

## Final Artifacts

| 产物 | 用途 | 状态 |
| --- | --- | --- |
| [`.daedalus/task-card.md`](task-card.md) | 学习目标与验收标准 | 已验证 |
| [`notes/02-question-roadmap.md`](../notes/02-question-roadmap.md) | 递进问题路线图 | 已验证 |
| [`notes/03-runbook.md`](../notes/03-runbook.md) | 本地运行与调试证据 | 已验证 |
| [`notes/04-architecture.md`](../notes/04-architecture.md) | 架构入口，指向 Codex agent loop 架构笔记 | 已验证 |
| [`notes/06-code-reader/README.md`](../notes/06-code-reader/README.md) | 核心源码阅读阶段入口，聚焦 auth/approval/sandbox | 已验证 |
| [`demo/design.md`](../demo/design.md) | mini demo 设计蓝图，包含 Phase 1/Phase 2、状态机、事件协议和验收测试 | 已验证 |
| [`guides/08-demo-coder/README.md`](../guides/08-demo-coder/README.md) | 08 阶段编码子地图，按 slice 推进 Phase 1 demo | 已验证 |
| [`demo/README.md`](../demo/README.md) | mini demo 实现、运行和验收说明 | 待填 |
| [`notes/09-biz-solver/README.md`](../notes/09-biz-solver/README.md) | 将权限/沙箱模式迁移到业务问题 | 待填 |
| knowledge-base entry | 已验证知识归档 | 待填 |

## Current Position

- 当前阶段：`08-demo-coder`。
- 当前目标：带用户按 08 子地图实现 Phase 1 mini demo。
- 当前障碍：Slice 9 策略已定稿：同一轮多个 tool calls 互不影响，失败或拒绝不传播成 batch-level skip；下一步是把 `agent/react.rs` 从顺序 loop 改成独立结果收集，并按原始 index 稳定回灌 observations。
- 当前动作服务的产物：`demo/src/`、后续 `demo/README.md`。
- 当前光标：Slice 9 multi-tool independent execution；先从 `agent/react.rs` 的多 tool call 调度、并发收集和 `ToolRuntimeResult::Skipped` 是否保留开始。

## Artifact Dependency Graph

```text
question-roadmap
  -> runbook
  -> architecture / context code-reading
  -> auth-approval-sandbox code-reading
  -> demo/design
  -> demo/README
  -> business-application
  -> knowledge-base entry
```

## Open Gaps

- [x] Decision 合成：已确认多段命令通过最严格 `Decision` 聚合，并映射为 demo 的 `approval_requirement_for_command(CommandRequest, Vec<CommandSegment>) -> ApprovalRequirement`。
- [x] Runtime request assembly：已确认 shell / unified exec 的 request 上下文字段、字段来源，以及 approval key 从 request 派生。
- [x] Orchestrator retry：已确认 sandbox denied 后何时结束、何时请求 no-sandbox approval、何时直接 retry、何时执行第二次 no-sandbox attempt。
- [x] Demo architecture finalization：已将 `demo/design.md` 从草稿收敛为 `08-demo-coder` 可直接实现的蓝图。
- [x] Stage transition confirmation：用户已确认，`07-demo-architecture` 已完成并进入 `08-demo-coder`。
- [x] Slice 0 Project Skeleton：用户已创建 demo crate、library/binary 入口、Makefile，并运行空测试。
- [x] Slice 1 Domain Models：用户已实现领域模型，作为后续 registry / approval / runner / event 的共同语言。
- [x] Slice 2 Capability Registry：用户已实现内置能力加载与 prefix-based 命令匹配，覆盖 AT-01。
- [x] Slice 3 Approval Decision：用户已实现 `decide_approval`，覆盖 capability mismatch fail closed、`Prompt + Never -> Forbidden`、approval scope 绑定 request context、`Skip != bypass sandbox`，`cargo test` 通过 11 个测试。
- [x] Slice 4 SimulatedExecutionRunner：用户已实现规则表驱动的模拟 runner，使用 `request.argv + ExecutionAttempt` 匹配规则，区分 `Success`、`CommandFailed`、`SandboxDenied`；Agent 补充 5 个 runner unit tests，`cargo test` 通过 16 个测试。
- [x] Slice 5 Retry Gate：用户已实现 `decide_retry`，区分 `CommandFailed`、`SandboxDenied`、`already_retried`、`ApprovalPolicy` 与 `NetworkPolicy`；Agent 补充 8 个 retry unit tests，`cargo test` 通过 24 个测试。
- [x] Slice 6 Agent Orchestrator：已新增 `agent::llm`、`OpenAiCompatibleLlm`、`agent::react` 和顶层 `tool` module；当前 `tool/function.rs` 承载 `add/sub` pure tools，`tool/runtime.rs` 已承接 `ToolRuntime::run(call)` 的 pure function path 和 `run_command` command path，并补齐 add/sub/invalid args/unknown tool、run_command safe-read/network-install/command-failed/denied/invalid-json/unmatched-capability 单测。真实 LLM ReAct 主链路和 ReAct hardening 已完成，关键类型/函数注释和 TODO 已补齐，`ExecutionRunner` 边界已取代顶层 sandbox module；`run_shell_command` 单命令编排测试已补齐，ToolRuntime / ReAct 层 command path observation 已覆盖 Finished / Failed / Denied。
- [x] Slice 7 Retry Policy And Denial Semantics：`decide_retry` 已接收并尊重 `RetryPolicy`；`RetryPolicy::Never` 优先阻止 retry，`RetryPolicy::WithApproval` 仍受 `ApprovalPolicy` 和 `NetworkPolicy` 约束，`RetryPolicy::WithoutApproval` 只免除非网络 sandbox denied 的 retry approval，不能绕过 network deny / network prompt。

## Critical Lens

Critical Lens 用来防止把 Codex 当成唯一事实。当前 demo 要先忠实模仿 Codex 的核心安全链路，感受它为什么把 approval / sandbox / retry / event 拆成多个层次；然后再判断哪些复杂度来自 Codex 的生产约束，哪些不适合业务迁移时照搬。

- 当前素材中可能被过度神化的设计：Codex 的权限 / 沙箱 / retry 组合是成熟 CLI Agent 的生产级折中，不是所有 Agent demo 或业务系统都必须照搬的完整复杂度。
- 当前 demo 需要忠实模仿的核心机制：tool call 不能直接执行；必须经过 capability match、approval requirement、sandbox-first execution、controlled retry 和 observation/event 回流。
- 当前 demo 不应无意识照抄的设计：跨平台 sandbox 细节、完整 TUI/MCP elicitation、Codex 所有 approval policy 组合和长期 session policy 存储。
- 当前已决策但待实现的 demo 假设：多 tool call 中单个失败或拒绝不应导致后续 tool 被 skipped；更好的纠错体验是回传完整 observations，让 agent 一次性修正多个问题。
- 当前可以尝试简化、改进或丢弃的部分：Phase 1 先用清晰的 event protocol 表达安全链路，不急着复刻 Codex 的全部 UI/streaming 事件细节。
- 当前迁移到业务场景前必须重新验证的约束：业务是否真的需要 no-sandbox retry、session approval 复用、网络 host 级审批和多工具独立执行策略。

### Slice 6 当前待解决问题

- [x] `react.rs` 已补 `max_turns`，超限时返回错误而不是伪装成 `Completed`。
- [x] `ToolCallFinished` 已透出到外部事件流，保持“模型决策”和“runtime 执行”分层可观察。
- [x] fake LLM deterministic tests 已覆盖 final answer without tool、一轮多个 tool call、多轮 tool call、tool failure observation 和 max turns exceeded。
- [x] `take_sse_events` / `map_chunk` 已补 fixture tests，锁定 DeepSeek / OpenAI-compatible `data: ...` stream、`[DONE]`、tool args 分片聚合和单 chunk 多 tool calls 边界。
- [x] `tool/runtime.rs` 的 `ToolRuntime::run(call)` pure function 边界已稳定，`react.rs` 不再直接调用 pure function runner。
- [x] `run_shell_command` 编排测试：已确认 `resolve_approval_requirement`、`ExecutionRunner`、`decide_retry` 在单命令链路中正确串联。
- [x] `run_shell_command` 前置检查：已验证 `Skip -> SandboxFirst`、`NeedsApproval approved -> SandboxFirst`、`NoSandboxRetry` 走 execution runner attempt，并处理 `RetryWithApproval` approve/reject 两条路径。
- [x] `ToolRuntime::run(run_command)` 直接测试：已覆盖 safe read 成功、network install retry 成功、command failure、dangerous shell denied、invalid JSON、unmatched capability。
- [x] ReAct 层 `run_command` observation 验收：已确认 shell runtime 的 Finished / Failed / Denied 能正确转为下一轮 LLM 可见的 tool observation。
- [x] Slice 6 closeout：已同步旧 guides、todo、outcome-map 和 design 中的旧命名和旧规划，冻结 non-goals。
- [x] Slice 7 closeout：`cargo test` 通过 61 个默认测试，3 个 live LLM 测试 ignored；当前 Slice 8 review 后最新 demo 测试为 63 passed、3 ignored。
- [x] Slice 8 event outlet：`ApprovalGateway + PendingApproval` 已取代旧 `ApprovalController / ApprovalBroker` 事件绑定；`CommandNeedsApproval` 已通过当前 run stream 透出，send failure fail closed，pending 会清理；`CommandEventEmitter` 已抽取，集中填充 command event 上下文；`run_execution_attempt` 已实现，让每个 attempt 的 terminal event 不依赖开发者手动记住。
- `OpenAiCompatibleLlm::default()` 缺少 env 时 panic 作为 demo 约束暂时接受；Phase 2 或库化时再考虑 `from_env()` / `try_from_env()`。

## Why This Step Matters

当前步骤只服务于实现 Phase 1 demo。每次编码都必须对应 `guides/08-demo-coder/README.md` 中的 slice 和 `demo/design.md` 中的验收测试。

## Stop Rules

- 不继续扩展 Windows sandbox 细节，除非它直接改变 demo 的跨平台抽象。
- 不继续扩展 MCP elicitation 细节，除非它直接改变 approval request 的最小接口。
- 不继续扩展完整 TUI UI 细节，当前 demo 只需要 CLI 层面的授权交互。
- 不把“多段命令更危险”的工程直觉写成源码事实；必须沿用源码里的概念区分，例如 command segment 和 complex parsing fallback。
