# 08 Demo Coder Stage Map

本阶段子地图用于把 `demo/design.md` 定稿后的架构落成 Phase 1 mini demo。08 的目标不是做完整 Codex，也不是直接攻克 OS sandbox，而是先实现可验证的安全本地命令执行闭环。

## North Star

实现一个 Phase 1 mini demo：

```text
real OpenAI-compatible Chat Completions streaming loop
  -> capability registry
  -> command request
  -> approval decision
  -> simulated execution runner sandbox-first attempt
  -> controlled retry
  -> event stream
  -> final output
```

完成后，`demo/README.md` 应能说明如何运行 demo、如何运行验收测试、Phase 1 简化了什么、Phase 2 如何替换为 `OsExecutionRunner`。

## Stage Inputs

- [`demo/design.md`](../../demo/design.md)：07 阶段定稿的设计蓝图。
- [`notes/06-code-reader/README.md`](../../notes/06-code-reader/README.md)：auth / approval / sandbox 的源码证据入口。
- [`notes/06-code-reader/01-runtime-request-assembly.md`](../../notes/06-code-reader/01-runtime-request-assembly.md)：`CommandRequest` 字段来源依据。
- [`notes/06-code-reader/02-orchestrator-retry.md`](../../notes/06-code-reader/02-orchestrator-retry.md)：sandbox denied 后 retry 状态机依据。

## Build Slices

### Slice 0: Project Skeleton

目标：建立 demo 工程骨架和测试入口。

产出：

- demo 源码目录。
- 测试命令。
- `demo/README.md` 初稿。

验收：

- 可以运行空测试。
- README 能说明 Phase 1 / Phase 2 边界。

### Slice 1: Domain Models

目标：先写领域模型，不写执行逻辑。

实现：

```text
CapabilityKind
CapabilityDescriptor
CapabilityPolicy
CommandRequest
ApprovalRequirement
ApprovalScope
ApprovalPersistence
SandboxProfile
NetworkPolicy
ExecutionAttempt
ExecutionResult
ExecutionFailure
RetryDecision
AgentEvent
AgentStatus
```

验收：

- 模型能表达 `safe-read`、`safe-test`、`network-install`、`dangerous-shell` 四类能力。
- 事件模型复用领域枚举，不重复发明 `tool_denied/tool_skip/tool_run_success`。

### Slice 2: Capability Registry

目标：实现内置能力注册表。

实现：

```text
load_builtin_capabilities() -> Vec<CapabilityDescriptor>
match_capability(argv) -> CapabilityKind
```

验收：

- 通过 AT-01。
- 未知或危险命令能归入 `dangerous-shell` 或显式 unsupported path。

### Slice 3: Approval Decision

目标：实现权限判断，不执行命令。

实现：

```text
decide_approval(request, capability_policy) -> ApprovalRequirement
approval store / session scope matching
```

验收：

- 通过 AT-02、AT-03、AT-04、AT-05、AT-12、AT-13。
- `Skip { bypass_sandbox: false }` 与 `Skip { bypass_sandbox: true }` 在类型和值上明确区分。

### Slice 4: SimulatedExecutionRunner

目标：实现 Phase 1 runner，让 sandbox denied 由可解释规则产生。

实现：

```text
ExecutionRunner.run(request, attempt) -> ExecutionResult
SimulatedExecutionRunner
```

验收：

- 通过 AT-06、AT-08、AT-09。
- `CommandFailed` 不进入 retry gate。
- `SandboxDenied` 必须携带可展示原因。

### Slice 5: Retry Gate

目标：实现 sandbox denied 后的受控 retry。

实现：

```text
decide_retry(request, requirement, failure, approval_state) -> RetryDecision
```

验收：

- 通过 AT-10、AT-11。
- 单个 tool call 最多一次 `SandboxFirst` 和一次 `NoSandboxRetry`。

### Slice 6: Agent Orchestrator

目标：把真实 OpenAI streaming model、tool approval、runner、retry 和 event stream 串成最小 ReAct loop。

实现：

```text
OpenAI-compatible Chat Completions stream
  -> ModelStreamEvent
  -> tool call
  -> approval / sandbox / retry
  -> tool output
  -> final answer
```

验收：

- 通过 AT-07、AT-14。
- 工具结果和命令失败都作为 observation 回灌给下一轮 model。
- `max_turns` 能阻止无限循环。

实现前阅读：

- [`01-rust-openai-integration.md`](01-rust-openai-integration.md)：Rust 中使用 DeepSeek / OpenAI-compatible Chat Completions、streaming 和 function calling 的最小接入方案。

当前代码结构：

- [`../../demo/src/agent/react.rs`](../../demo/src/agent/react.rs)：ReAct loop、tool call 收集和 message 回灌。
- [`../../demo/src/tool/function.rs`](../../demo/src/tool/function.rs)：`add/sub` pure function tools。
- [`../../demo/src/tool/runtime.rs`](../../demo/src/tool/runtime.rs)：`ToolRuntime` 已覆盖 pure function path 和 `run_command` command path。
- [`../../demo/src/tool/shell/`](../../demo/src/tool/shell)：`run_command` command tool 的 registry / approval / retry / orchestration 入口。
- [`../../demo/src/tool/shell/execution/`](../../demo/src/tool/shell/execution)：`ExecutionRunner` 与 `SimulatedExecutionRunner`，统一表达 sandbox / no-sandbox attempt。

下一步行动：

- [`02-slice-6-hardening.md`](02-slice-6-hardening.md)：在 live LLM 主链路跑通后，把 Slice 6 收敛到可验收状态。
- [`03-tool-runtime-data-relationships.md`](03-tool-runtime-data-relationships.md)：解释 `ToolCallFinished -> ToolDefinition -> CommandRequest -> ApprovalRequirement -> ExecutionResult` 的数据关系，帮助实现 `ToolRuntime::run`。
- [`04-slice-6-approval-sandbox-retry-integration.md`](04-slice-6-approval-sandbox-retry-integration.md)：把 command tool 的 approval / sandbox / retry 接入 ReAct tool execution path。
- [`05-slice-6-command-runtime-review.md`](05-slice-6-command-runtime-review.md)：记录当前 `ToolRuntime` / `tool::shell` review 结论，明确 `Fail` / `Deny` / `Denied` / `Skipped` 边界和 `run_shell_command` 下一步实现路径。
- [`06-slice-6-closeout-and-next-slices.md`](06-slice-6-closeout-and-next-slices.md)：基于当前代码和测试重新规划 Slice 6 收口与后续 slice。

### Slice 7: Retry Policy And Denial Semantics

目标：让 sandbox denied 后的 retry 尊重 capability-level `RetryPolicy`，补齐 `safe-read` denied 不应 retry 的安全边界。

状态：已完成。

详解：[`07-slice-7-policy-composition.md`](07-slice-7-policy-composition.md) 用流程图说明 `RetryPolicy`、`ApprovalPolicy` 和 `NetworkPolicy` 如何合成 retry decision。

验收：

- `safe-read` sandbox denied -> `DoNotRetry`。
- `safe-test` sandbox denied -> `RetryWithApproval`。
- `network-install` network prompt -> `RetryWithApproval`。
- 单个 tool call 仍最多一次 sandbox-first 和一次 no-sandbox retry。

### Slice 8: Event Protocol Hardening

目标：把 approval、execution attempt、retry decision、denied/skipped 等关键节点透出为可观察事件。

行动指南：[`08-slice-8-event-protocol-hardening.md`](08-slice-8-event-protocol-hardening.md)。

验收：

- 外部 stream 能看见 approval resolved / denied / execution started / retry evaluated。
- README 可以用 event trace 解释一次命令如何走完安全链路。

### Slice 9: Multi-Tool Hard-Deny And Skipped Semantics

目标：定义同一轮多个 tool call 中，一个安全拒绝发生后，后续 tool 是否执行以及如何回灌 observation。

验收：

- 安全拒绝后，后续依赖性 tool call 不盲跑。
- `ToolRuntimeResult::Skipped` 有真实生产路径或被明确删除。

### Slice 10: Approval Persistence

目标：实现 session approval 复用与 scope mismatch 失效。

验收：

- 相同 command prefix、cwd、sandbox、network scope 可以复用 session approval。
- cwd 或 network policy 变化时不能复用旧 approval。

### Slice 11: Demo README And Runbook

目标：让 demo 可运行、可讲解、可迁移。

产出：

- `demo/README.md`
- 运行命令。
- 验收测试命令。
- Phase 1 / Phase 2 差异说明。

验收：

- 用户可以按 README 运行 Phase 1 demo。
- README 能说明哪些设计来自 Codex，哪些是为了 mini demo 做的简化。

## Test Gates

每个 slice 完成后必须运行对应测试。进入 08 完成态前，至少满足：

```text
Phase 1 minimal gate:
AT-01, AT-02, AT-03, AT-04, AT-06, AT-08, AT-09, AT-11, AT-14

Phase 1 full gate:
AT-01 ... AT-14
```

## Stop Rules

- Phase 1 接真实 OpenAI streaming model，但默认测试不依赖真实 API；联网验收放到 runbook。
- Phase 1 不做真实 OS sandbox，用 `SimulatedExecutionRunner` 验证编排逻辑。
- Phase 1 不实现完整 shell parser，只支持验收用例需要的命令形态。
- Phase 1 不做完整 TUI，不引入与核心状态机无关的 UI 框架。
- 如果实现时发现设计不足，先回到 `demo/design.md` 更新架构，再继续写代码。

## Phase 2 Parking Lot

Phase 2 目标是接入 `OsExecutionRunner`，让 mini demo 从“可解释模型”走向“可用工具”。但它必须复用 Phase 1 的：

- `CommandRequest`
- `ApprovalRequirement`
- `RetryDecision`
- `AgentEvent`
- orchestrator 状态机

Phase 2 只替换 runner，不重写 approval / retry / event 协议。
