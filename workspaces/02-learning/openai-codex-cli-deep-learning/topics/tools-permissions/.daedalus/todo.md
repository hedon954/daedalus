# Todo Path Board

Todo 是动态路径看板。学习证据变化、阶段完成、学习路径需要收窄或扩展时，都要同步更新它和 [`outcome-map.md`](outcome-map.md)。

> Agent 负责拆解、指导、排障和验收；用户负责关键实践、观察和手写笔记。不要把 Agent 自动完成的事项伪装成用户已经掌握。

## North Star

- 最终产物：Codex Agent CLI 的源码理解、mini demo、业务迁移方案和知识归档。
- 最小 demo：一个保留 Codex 权限/沙箱不变量的本地命令执行链路。
- 业务迁移目标：为自有 Agent/CLI 设计可解释、可审批、可回滚的本地命令执行权限模型。

## Current Path

当前位于 `08-demo-coder`。阶段入口是 [`guides/08-demo-coder/README.md`](../guides/08-demo-coder/README.md)，实现产物入口是 [`demo/README.md`](../demo/README.md)。当前目标是由用户按 slice 地图亲手实现 Phase 1 mini demo，Agent 只做引导、校准和验证。

## Now

- 当前问题：Slice 8 Event Protocol Hardening 进行中：`ApprovalBroker/ApprovalController` 已重构为 `ApprovalGateway + PendingApproval`，approval request 由当前 `run_shell_command` 的 `EventSender` 发到当前 run stream，send failure 会 fail closed 并清理 pending；`CommandEventEmitter` 已抽取，command event 的 `index / call_id / name` 已集中填充；retry success trace tests 已显式锁定 `SandboxFirst -> Failed -> RetryEvaluated -> NoSandboxRetry` 顺序。当前剩余结构风险是 attempt lifecycle 还没有被 `run_execution_attempt` 这类 helper 强制保护。
- 为什么现在做它：真实 LLM streaming、tool call、observation 回灌已经可测试；现在需要恢复 Codex 学习的核心不变量：tool call 不能直接执行，必须先经过 capability / approval / sandbox first / controlled retry。
- 完成后解锁：`demo/README.md` 可以展示完整 command trace，证明 `run_command` 经过 approval、sandbox first、retry gate、no-sandbox retry 或 denied，而不是直接裸跑。
- 当前已做：新增 `FakeLlm` test double；`react.rs` 已补 `max_turns`、`ToolCallFinished` 透出、fake LLM deterministic tests；`openai.rs` 已补 SSE / parser fixture tests；`ToolRuntime` 已覆盖 `add/sub` 成功、参数错误、未知工具、`run_command` 安全读成功、网络安装 retry 成功、命令失败、危险命令拒绝、非法 JSON、未匹配 capability；`run_shell_command` 编排测试覆盖 skip、needs approval、command failure、retry without approval、retry with approval approve/reject，以及 safe-read sandbox denied 不 retry 的 registry 接线；ReAct `run_command` observation 测试覆盖 Finished / Failed / Denied 回灌；`RetryPolicy` 已纳入 retry gate；Slice 8 已补 `ToolCallContext`、`ApprovalGateway`、`PendingApproval`、内部 `ToolApprovalResult` 回流、`CommandEventEmitter`、command 专属 event 命名和 command execution / retry 事件透出；`SandboxFirst` failure event 已在 retry decision 前发出并被 shell / ReAct trace tests 锁定；`cargo test` 通过 63 个默认测试，3 个 live LLM 测试保持 ignored。
- 当前待解决：按 [`guides/08-demo-coder/09-slice-8-command-event-emitter-refactor.md`](../guides/08-demo-coder/09-slice-8-command-event-emitter-refactor.md) 继续引入 `run_execution_attempt`，把 `Started(attempt) -> Finished/Failed(attempt)` 从人工约定变成结构性保证；approval request 的 run-scoped stream 绑定和 `CommandEventEmitter` 已闭合。另记录一个后续 approval 语义缺口：当前 `ApprovalPolicy::OnRequest` 只表达“初始 capability prompt 可询问”，还没有建模“调用方显式请求 no-sandbox / escalation”的 request 字段；后续可考虑给 `CommandRequest` 增加 `requested_escalation` 或 `require_no_sandbox`，让 `OnRequest` 语义更贴近 Codex。`Default` 缺少 env 时 panic 作为 demo 约束暂时接受。

## Current Cursor

- Code frontier：Slice 8 进行中，当前代码光标在 `tool/shell/mod.rs` 的 execution attempt lifecycle；approval sender run-scoped 问题已闭合，`CommandEventEmitter` 已抽取，下一步是实现 `run_execution_attempt`。
- Already wired：`run_command` tool name、`RunCommandArgs`、`build_command_request`、capability matching、`ToolRuntimePlan::RunCommand`、`execute_plan -> run_shell_command`、`ExecutionRunner`、`SimulatedExecutionRunner` 已打通，并已通过 `run_shell_command` 编排测试、`ToolRuntime` command path 直接测试和 ReAct observation 测试验证。
- Current open decision：不再争论是否需要 `run_execution_attempt`；用户已确认它有必要，因为 attempt lifecycle 不变量应该由结构保证，而不是靠开发者记忆。下一步实现它并保持现有 trace tests 通过。
- Do not suggest：不要再建议“先把 command path 接入 ToolRuntime”或“补 RetryPolicy”；当前先做 event protocol hardening，再讨论 approval persistence / README。

## Critical Checkpoint

- Source constraint：Codex 需要服务真实 CLI Agent 的安全、本地命令执行、跨平台 sandbox、用户审批体验和事件可观察性；它的复杂度来自生产约束，不是单纯类型设计偏好。
- Faithful imitation：当前 demo 必须先模仿 `tool call -> approval -> sandbox first -> retry decision -> observation/event` 的分层安全链路，不能把工具调用简化成直接执行函数。
- Simplified / improved / discarded：Phase 1 只做清晰可测的事件协议，不复刻完整 TUI/MCP elicitation、跨平台 OS sandbox 和所有 Codex 内部事件细节。
- Transfer risk：业务迁移前要重新验证是否真的需要 no-sandbox retry、session approval、network host 级审批、多工具 hard-deny 语义，不能因为 Codex 有这些复杂度就默认采用。

## Gaps Blocking Next Stage

- [x] 定稿 demo scope：明确 Phase 1 使用 `SimulatedExecutionRunner`，Phase 2 再接 `OsExecutionRunner`。
- [x] 定稿核心数据结构：已收敛为 capability、request、approval、sandbox runner、retry、event、agent status 等实现级模型。
- [x] 定稿主状态机：已对齐最小 ReAct loop，支持 tool result / command failure observation 回灌和受控 retry。
- [x] 定稿验收用例：已形成 AT-01 到 AT-14，并区分 Phase 1 minimal gate 与 full gate。
- [x] 制定 08 阶段子地图：已创建 `guides/08-demo-coder/README.md`。
- [x] 用户确认可以进入 `08-demo-coder`。
- [x] Slice 0 Project Skeleton：用户已创建独立 Rust crate、library/binary 入口、Makefile，并运行空测试。
- [x] Slice 1 Domain Models：用户已实现 capability、request、approval、execution、retry、event 等领域模型，测试通过。
- [x] Slice 2 Capability Registry：用户已实现内置能力加载、默认策略断言和 prefix-based `match_capability`。
- [x] Slice 3 Approval Decision：用户已实现 `decide_approval` 并完成 review；`cargo test` 通过 11 个测试，覆盖 approval scope、forbidden、prompt policy、capability mismatch fail closed。
- [x] Slice 4 SimulatedExecutionRunner：用户已实现规则表驱动 runner，Agent 补充 read success、command failed、sandbox denied、no-sandbox retry success、unknown command failed 五个测试；`cargo test` 通过 16 个测试。
- [x] Slice 5 Retry Gate：用户已实现 `decide_retry`，Agent 补充 command failed、already retried、never/on-request、network prompt/allow/deny、non-network sandbox denied 等 8 个测试；`cargo test` 通过 24 个测试。
- [x] Slice 6 Agent Orchestrator：真实 LLM ReAct 主链路已 live test 跑通；ReAct hardening 已补齐 `max_turns`、`ToolCallFinished`、fake LLM tests 和 parser fixture tests；用户已重构为 `tool/function.rs`、`tool/runtime.rs`、`tool/shell/`，`ToolRuntime` pure function path 和 `run_command` command path 均已接入并补齐单测；command path 的 `run_shell_command` 单命令编排已通过测试，ReAct 层 `run_command` Finished / Failed / Denied observation 回灌已通过测试。
- [x] Slice 6 Closeout：同步旧 guides、todo、outcome-map 和 design，冻结 Slice 6 non-goals。
- [x] Slice 7 Retry Policy And Denial Semantics：让 `decide_retry` 尊重 capability-level `RetryPolicy`，补齐 `safe-read` denied 不 retry、`safe-test` approval retry、network prompt approval retry、network deny 不被 `WithoutApproval` 绕过等边界。
- [ ] Slice 8 Event Protocol Hardening：`ApprovalGateway + PendingApproval`、run-scoped `CommandNeedsApproval`、`CommandEventEmitter`、command attempt / retry event 和 trace tests 已完成；待实现 `run_execution_attempt`，把 attempt lifecycle 从测试约束推进为代码结构约束。
- [ ] Slice 9 Multi-Tool Hard-Deny And Skipped Semantics：决定并实现同轮多工具安全拒绝后的 skipped 行为。
- [ ] Slice 10 Approval Persistence：实现 session approval 复用和 scope mismatch 失效。
- [ ] ApprovalPolicy OnRequest Semantic Hardening：补充显式 escalation request 建模，区分“初始请求提权可询问”和“sandbox failure 自动提权询问”，避免 `OnRequest` 与 `OnFailure` 语义混淆。
- [ ] Slice 11 README / Runbook：补齐运行说明、验收命令和 Phase 2 说明。

## 06 Code Reader Gaps

- [x] Decision 合成：已确认多段命令通过最严格 `Decision` 聚合，并映射为 demo 的 `approval_requirement_for_command` 接口。
- [x] Runtime request assembly：已确认 shell / unified exec 的 request 上下文字段、字段来源，以及 approval key 从 request 派生。
- [x] Orchestrator retry：已确认 sandbox denied 后何时结束、何时请求 no-sandbox approval、何时直接 retry、何时执行第二次 no-sandbox attempt。

## Stage Exit Criteria

- [x] `demo/design.md` 不再标记为 blocked by source evidence。
- [x] demo scope、non-goals 和验收用例足以指导实现。
- [x] 可以明确进入 `08-demo-coder` 的第一个实现切片。
- [x] 用户确认 demo 蓝图后，进入 `08-demo-coder`。
- [ ] Phase 1 minimal gate 通过：AT-01、AT-02、AT-03、AT-04、AT-06、AT-08、AT-09、AT-11、AT-14。
- [ ] `demo/README.md` 能指导用户运行 demo 和验收测试。

## Done

- [x] 完成 `04-debugger-guide`：用户已执行最小 runbook，`codex-exec` 测试通过。
- [x] 完成 `05-arch-analyzer`：形成 `notes/05-codex-agent-loop-architecture.md`，覆盖 agent loop、工具、权限、沙箱和事件回流。
- [x] 完成上下文管理专题：形成 `notes/06-codex-context-and-compaction.md`，覆盖 history、prompt view、rollout、tool result 回灌和 compact。
- [x] 回滚 Agent 代学式 code-reading 记录，并把规则改成用户先形成假设、Agent 再校准。
- [x] 重塑 `notes/07-code-reading.md`，把 auth/approval/sandbox 阅读整理成生产问题、源码证据、不变量、代价和迁移模式。
- [x] 新增 [`outcome-map.md`](outcome-map.md)，明确当前只补 3 个 demo 缺口。
- [x] 新增 [`demo/design.md`](../demo/design.md) 草案，用源码阅读逐步填字段。
- [x] 补齐 Decision 合成缺口：用户已复述 `Forbidden` 聚合、`Decision::Allow != bypass_sandbox`，并抽象出 demo 的 `approval_requirement_for_command` 输入/输出。
- [x] 将 active 阶段迁移为文件夹入口：新增 [`guides/06-code-reader/README.md`](../guides/06-code-reader/README.md) 和 [`notes/06-code-reader/README.md`](../notes/06-code-reader/README.md)，旧单文件保留为 legacy。
- [x] 补齐 Runtime request assembly 缺口：用户已对比 `ShellRequest` / `UnifiedExecRequest`，并完成 `CommandRequest` 字段来源映射。
- [x] 补齐 Orchestrator retry 缺口：用户已复述 sandbox denied 后的 network approval context、approval policy gate、retry approval 和 no-sandbox second attempt 状态机。
- [x] 完成 `07-demo-architecture`：`demo/design.md` 已定稿为 Phase 1/Phase 2 蓝图，`guides/08-demo-coder/README.md` 已提供 08 阶段子地图。
- [x] 用户完成 `08-demo-coder` Slice 0：`demo/` 独立 Rust crate 已创建，`cargo test` 通过 1 个占位测试，binary 输出 `Hello, world!`。
- [x] 用户完成 `08-demo-coder` Slice 1：补齐 `CapabilityDescriptor`、`CapabilityPolicy`、`CommandRequest`、`ExecutionFailure::SandboxDenied`、`ExecutionAttempt`、`RetryDecision` 等领域模型，`cargo test` 通过。
- [x] 用户完成 `08-demo-coder` Slice 2：实现 `CapabilityRegistry`、内置四类 capability、默认策略测试和 prefix-based command matching，`cargo test` 通过。
- [x] 用户完成 `08-demo-coder` Slice 3：实现 `decide_approval`，明确 `DefaultDecision`、`ApprovalPolicy`、request context 的综合优先级；将 capability mismatch 设计为安全边界上的 `Forbidden` fail closed；`cargo test` 通过。
- [x] 用户完成 `08-demo-coder` Slice 4：实现 `SimulatedExecutionRunner`，用 `command_prefix + simulated_attempt + simulated_result` 表达模拟执行规则；区分 `CommandFailed` 与 `SandboxDenied`，并支持 no-sandbox retry success；`cargo test` 通过。
- [x] 用户完成 `08-demo-coder` Slice 5：实现 `Retry Gate`，将 sandbox denied 后的策略收敛为 `DoNotRetry`、`RetryWithoutApproval`、`RetryWithApproval`；`cargo test` 通过。
- [x] 用户推进 `08-demo-coder` Slice 6：实现 `OpenAiCompatibleLlm` 多 tool call 聚合、`tool/function.rs` 示例工具、`react.rs` channel-based ReAct loop；live LLM 手动测试跑通多轮 tool call 和 observation 回灌，相关实现问题已沉淀到 `notes/08-demo-coder/01-react-loop-implementation-issues.md`。
- [x] 完成 Slice 6 ReAct hardening：`max_turns` 超限返回错误、`ToolCallFinished` 对外透出、`FakeLlm` test double 下沉到 `agent::llm::fake`、fake LLM tests 覆盖 ReAct 关键路径、OpenAI-compatible parser fixture tests 覆盖 SSE 与 tool call 聚合边界。

## Canceled

- 开放式继续泛读 Codex 权限系统。
- 现在扩展 Windows sandbox、完整 TUI UI、完整 MCP elicitation。
