# 08 Demo Coder Stage Map

本阶段子地图用于把 `demo/design.md` 定稿后的架构落成 Phase 1 mini demo。08 的目标不是做完整 Codex，也不是直接攻克 OS sandbox，而是先实现可验证的安全本地命令执行闭环。

## North Star

实现一个 Phase 1 mini demo：

```text
real OpenAI-compatible Chat Completions streaming loop
  -> capability registry
  -> command request
  -> approval decision
  -> simulated sandbox first attempt
  -> controlled retry
  -> event stream
  -> final output
```

完成后，`demo/README.md` 应能说明如何运行 demo、如何运行验收测试、Phase 1 简化了什么、Phase 2 如何替换为 `OsSandboxRunner`。

## Stage Inputs

- [`demo/design.md`](../../demo/design.md)：07 阶段定稿的设计蓝图。
- [`notes/06-code-reader/README.md`](../../notes/06-code-reader/README.md)：auth / approval / sandbox 的源码证据入口。
- [`notes/06-code-reader/runtime-request-assembly.md`](../../notes/06-code-reader/runtime-request-assembly.md)：`CommandRequest` 字段来源依据。
- [`notes/06-code-reader/orchestrator-retry.md`](../../notes/06-code-reader/orchestrator-retry.md)：sandbox denied 后 retry 状态机依据。

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

### Slice 4: SimulatedSandboxRunner

目标：实现 Phase 1 runner，让 sandbox denied 由可解释规则产生。

实现：

```text
SandboxRunner.run(request, attempt) -> ExecutionResult
SimulatedSandboxRunner
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

- [`rust-openai-integration.md`](rust-openai-integration.md)：Rust 中使用 DeepSeek / OpenAI-compatible Chat Completions、streaming 和 function calling 的最小接入方案。

当前代码结构：

- [`../../demo/src/agent/react.rs`](../../demo/src/agent/react.rs)：ReAct loop、tool call 收集和 message 回灌。
- [`../../demo/src/tool/function.rs`](../../demo/src/tool/function.rs)：`add/sub` pure function tools。
- [`../../demo/src/tool/runtime.rs`](../../demo/src/tool/runtime.rs)：`ToolRuntime` WIP；pure function path 已接入，下一步补 command path。
- [`../../demo/src/tool/shell.rs`](../../demo/src/tool/shell.rs)：command tool 预留入口。

下一步行动：

- [`slice-6-hardening.md`](slice-6-hardening.md)：在 live LLM 主链路跑通后，把 Slice 6 收敛到可验收状态。
- [`slice-6-approval-sandbox-retry-integration.md`](slice-6-approval-sandbox-retry-integration.md)：把 command tool 的 approval / sandbox / retry 接入 ReAct tool execution path。
- [`tool-runtime-data-relationships.md`](tool-runtime-data-relationships.md)：解释 `ToolCallFinished -> ToolDefinition -> CommandRequest -> ApprovalRequirement -> ExecutionResult` 的数据关系，帮助实现 `ToolRuntime::run`。

### Slice 7: Demo README And Runbook

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
- Phase 1 不做真实 OS sandbox，用 `SimulatedSandboxRunner` 验证编排逻辑。
- Phase 1 不实现完整 shell parser，只支持验收用例需要的命令形态。
- Phase 1 不做完整 TUI，不引入与核心状态机无关的 UI 框架。
- 如果实现时发现设计不足，先回到 `demo/design.md` 更新架构，再继续写代码。

## Phase 2 Parking Lot

Phase 2 目标是接入 `OsSandboxRunner`，让 mini demo 从“可解释模型”走向“可用工具”。但它必须复用 Phase 1 的：

- `CommandRequest`
- `ApprovalRequirement`
- `RetryDecision`
- `AgentEvent`
- orchestrator 状态机

Phase 2 只替换 runner，不重写 approval / retry / event 协议。
