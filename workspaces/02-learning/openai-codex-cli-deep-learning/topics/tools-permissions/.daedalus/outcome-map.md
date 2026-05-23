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
- 当前障碍：Slice 6 Agent Orchestrator 进行中；LLM streaming adapter 已收敛为 DeepSeek / OpenAI-compatible Chat Completions 口径，但 parser fixture tests、env 初始化和 orchestrator 串联仍待完成。
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
- [ ] Slice 6 Agent Orchestrator：已新增 `agent::llm`、`OpenAiCompatibleLlm` 和 Rust/OpenAI integration guide；当前能以 DeepSeek / OpenAI-compatible Chat Completions 形态将流式 chunk 映射为 `StreamEvent`，并已同步 guide 口径；但尚未串起 approval / runner / retry，也未完成 parser fixture tests。

### Slice 6 当前待解决问题

- `take_sse_events` 当前按 DeepSeek / OpenAI-compatible Chat Completions 的常见 `data: ...` stream 处理；这不是 blocker，但需要用 recorded fixture 单测锁定边界。
- `OpenAiCompatibleLlm::default()` 仍会在缺少 `DEEPSEEK_API_KEY` 时 panic；后续应改为 `from_env() -> anyhow::Result<Self>`。
- `map_chunk` / `take_sse_events` / `map_openai_event` 缺少默认 fixture 单测；当前 `cargo test` 只证明编译通过和 ignored 联网测试存在。

## Why This Step Matters

当前步骤只服务于实现 Phase 1 demo。每次编码都必须对应 `guides/08-demo-coder/README.md` 中的 slice 和 `demo/design.md` 中的验收测试。

## Stop Rules

- 不继续扩展 Windows sandbox 细节，除非它直接改变 demo 的跨平台抽象。
- 不继续扩展 MCP elicitation 细节，除非它直接改变 approval request 的最小接口。
- 不继续扩展完整 TUI UI 细节，当前 demo 只需要 CLI 层面的授权交互。
- 不把“多段命令更危险”的工程直觉写成源码事实；必须沿用源码里的概念区分，例如 command segment 和 complex parsing fallback。
