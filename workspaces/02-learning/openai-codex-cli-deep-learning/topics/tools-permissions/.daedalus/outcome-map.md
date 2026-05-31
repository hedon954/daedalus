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
| [`notes/question-roadmap.md`](../notes/question-roadmap.md) | 递进问题路线图 | 已验证 |
| [`notes/runbook.md`](../notes/runbook.md) | 本地运行与调试证据 | 已验证 |
| [`notes/architecture.md`](../notes/architecture.md) | 架构入口，指向 Codex agent loop 架构笔记 | 已验证 |
| [`notes/06-code-reader/README.md`](../notes/06-code-reader/README.md) | 核心源码阅读阶段入口，聚焦 auth/approval/sandbox | 已验证 |
| [`demo/design.md`](../demo/design.md) | mini demo 设计蓝图，包含 Phase 1/Phase 2、状态机、事件协议和验收测试 | 已验证 |
| [`guides/08-demo-coder/README.md`](../guides/08-demo-coder/README.md) | 08 阶段编码子地图，按 slice 推进 Phase 1 demo | 已验证 |
| [`demo/README.md`](../demo/README.md) | mini demo 实现、运行和验收说明 | 待填 |
| [`notes/09-biz-solver/README.md`](../notes/09-biz-solver/README.md) | 将权限/沙箱模式迁移到业务问题 | 待填 |
| knowledge-base entry | 已验证知识归档 | 待填 |

## Current Position

- 当前阶段：`08-demo-coder`。
- 当前目标：带用户按 08 子地图实现 Phase 1 mini demo。
- 当前障碍：Slice 6 Agent Orchestrator 进行中；真实 LLM -> 多 tool call -> tool result observation -> 下一轮 LLM 的 ReAct 主链路已跑通，ReAct hardening 已补齐 `max_turns`、`ToolCallFinished` 事件透出、fake LLM deterministic tests 和 OpenAI-compatible parser fixture tests；`ToolRuntime` pure function path 已接入并补齐单测；剩余缺口是把 command tool 的 approval / sandbox / retry 接入 ReAct loop。
- 当前动作服务的产物：`demo/src/`、后续 `demo/README.md`。

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
- [x] Slice 4 SimulatedSandboxRunner：用户已实现规则表驱动的模拟 runner，使用 `request.argv + ExecutionAttempt` 匹配规则，区分 `Success`、`CommandFailed`、`SandboxDenied`；Agent 补充 5 个 runner unit tests，`cargo test` 通过 16 个测试。
- [x] Slice 5 Retry Gate：用户已实现 `decide_retry`，区分 `CommandFailed`、`SandboxDenied`、`already_retried`、`ApprovalPolicy` 与 `NetworkPolicy`；Agent 补充 8 个 retry unit tests，`cargo test` 通过 24 个测试。
- [ ] Slice 6 Agent Orchestrator：已新增 `agent::llm`、`OpenAiCompatibleLlm`、`agent::react` 和顶层 `tool` module；当前 `tool/function.rs` 承载 `add/sub` pure tools，`tool/runtime.rs` 已承接 `ToolRuntime::run(call)` 的 pure function path 并补齐 add/sub/invalid args/unknown tool 单测，`tool/shell.rs` 是 command tool 预留入口。真实 LLM ReAct 主链路和 ReAct hardening 已完成，仍需通过 command path 串起 approval / runner / retry。

### Slice 6 当前待解决问题

- [x] `react.rs` 已补 `max_turns`，超限时返回错误而不是伪装成 `Completed`。
- [x] `ToolCallFinished` 已透出到外部事件流，保持“模型决策”和“runtime 执行”分层可观察。
- [x] fake LLM deterministic tests 已覆盖 final answer without tool、一轮多个 tool call、多轮 tool call、tool failure observation 和 max turns exceeded。
- [x] `take_sse_events` / `map_chunk` 已补 fixture tests，锁定 DeepSeek / OpenAI-compatible `data: ...` stream、`[DONE]`、tool args 分片聚合和单 chunk 多 tool calls 边界。
- [x] `tool/runtime.rs` 的 `ToolRuntime::run(call)` pure function 边界已稳定，`react.rs` 不再直接调用 pure function runner。
- [ ] 下一步：把 `shell` command tool path 接入 `ToolRuntime`，并串起 `decide_approval`、`SimulatedSandboxRunner`、`decide_retry`。
- `OpenAiCompatibleLlm::default()` 缺少 env 时 panic 作为 demo 约束暂时接受；Phase 2 或库化时再考虑 `from_env()` / `try_from_env()`。

## Why This Step Matters

当前步骤只服务于实现 Phase 1 demo。每次编码都必须对应 `guides/08-demo-coder/README.md` 中的 slice 和 `demo/design.md` 中的验收测试。

## Stop Rules

- 不继续扩展 Windows sandbox 细节，除非它直接改变 demo 的跨平台抽象。
- 不继续扩展 MCP elicitation 细节，除非它直接改变 approval request 的最小接口。
- 不继续扩展完整 TUI UI 细节，当前 demo 只需要 CLI 层面的授权交互。
- 不把“多段命令更危险”的工程直觉写成源码事实；必须沿用源码里的概念区分，例如 command segment 和 complex parsing fallback。
