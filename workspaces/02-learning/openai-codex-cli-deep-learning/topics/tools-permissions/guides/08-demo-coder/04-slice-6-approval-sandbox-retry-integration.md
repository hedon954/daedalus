# Slice 6 Approval Sandbox Retry Integration

这份 guide 基于当前最新代码结构，服务下一轮 `08-demo-coder`：把已独立完成的 approval、simulated sandbox runner、retry gate 接入 ReAct tool execution path。

## Learning Navigation

- Final artifact: [`../../demo/README.md`](../../demo/README.md)
- Current stage: `08-demo-coder`
- Current slice: Slice 6 Agent Orchestrator
- Current gap: `react.rs` 已通过 `ToolRuntime` 调用 `add/sub` pure function path，但 command tool 尚未接入 approval / sandbox / retry。
- Current code shape:
  - [`agent/react.rs`](../../demo/src/agent/react.rs)：收集 tool calls、回灌 messages、通过 `ToolRuntime` 执行工具。
  - [`tool/function.rs`](../../demo/src/tool/function.rs)：`add/sub` pure function tools。
  - [`tool/runtime.rs`](../../demo/src/tool/runtime.rs)：`ToolRuntime` WIP，下一步核心文件。
  - [`tool/shell/`](../../demo/src/tool/shell)：`run_command` command tool 的内部实现模块。
- After this: Slice 6 达到 Phase 1 主链路验收，可以进入 Slice 7 README / Runbook。

## North Star

下一步不是继续增强 LLM，而是把 tool execution 从“直接运行函数”改成“经 runtime 规划后执行”：

```text
ToolCallFinished
  -> ToolRuntime::run
  -> pure function path or command path
  -> command path: capability / approval / sandbox / retry
  -> ToolRuntimeResult
  -> StreamEvent + role=tool observation
```

## Current Design Decisions

已经确认的决策：

1. `add/sub` 继续是 pure function tools，但也要进入统一 `ToolRuntime` 边界。
2. `run_command` 是模型可见的 command tool，内部由 `tool/shell/` 实现，后续走完整 `CommandRequest -> ApprovalRequirement -> SandboxRunner -> RetryDecision` 链路。
3. `react.rs` 不长期承载 approval / sandbox / retry 细节，只负责 ReAct loop 和 message 回灌。
4. Phase 1 多 tool call 先按 `index` 顺序执行。
5. 如果某个 tool call 被 forbidden / rejected，后续 tool call 不真实执行，但要补 skipped observation，保证每个 `tool_call_id` 都有对应 tool message。

## Action Card

### 1. Stabilize ToolRuntime Boundary

状态：已完成 pure function path。`react.rs` 已不再直接依赖 `tool::function::run_pure_function`。

当前入口：

```rust
pub struct ToolRuntime {}
pub struct ToolRuntimeResult {}

impl ToolRuntime {
    pub fn fun(&self, call: &ToolCallFinished) -> ToolRuntimeResult {
        todo!()
    }
}
```

建议改成语义明确的入口：

```rust
impl ToolRuntime {
    pub fn run(&self, call: &ToolCallFinished) -> ToolRuntimeResult {
        todo!()
    }
}
```

验收：

- `add/sub` 仍能通过 fake LLM tests 跑通。
- 未知 tool 不 panic，返回可回灌给模型的失败或拒绝结果。
- `react.rs` 只关心 `ToolRuntimeResult -> StreamEvent / role=tool message`。
- ToolRuntime pure function 单测只锁定成功输出；错误路径只断言 `Failed` 变体，不锁具体错误文案。

### 2. Add Tool Definition Layer

目标：避免 `ToolCallFinished.name` 直接绑定 `CapabilityDescriptor`。

建议：

```rust
enum ToolKind {
    PureFunction,
    Command,
}

struct ToolDefinition {
    name: &'static str,
    kind: ToolKind,
}
```

当前最小 tool set：

```text
add  -> PureFunction
sub  -> PureFunction
run_command -> Command
```

验收：

- pure function 不需要 `CommandRequest`。
- command tool 才进入 `CapabilityRegistry`。

### 3. Move Pure Function Execution Behind Runtime

目标：先完成最小无风险闭环。

路径：

```text
ToolCallFinished(add/sub)
  -> ToolRuntime.find_tool
  -> ToolRuntimePlan::RunPureFunction
  -> tool/function.rs::run_pure_function
  -> ToolRuntimeResult::Finished or Failed
```

验收：

- 现有 ReAct tests 仍通过。
- `ToolRuntimeResult` 内部区分 `Finished`、`Failed`、`Denied`、`Skipped`，即使事件暂时仍映射到 `ToolRunFinished` / `ToolRunFailed`。

### 4. Add Command Planning Without Running Yet

目标：先把 `run_command` tool call 变成 `CommandRequest`，不要一步到位执行。

当前 `run_command` 参数形态：

```json
{
  "command": "npm test",
  "justification": "为了验证当前 demo"
}
```

路径：

```text
ToolCallFinished(run_command)
  -> parse command/justification
  -> derive argv
  -> CapabilityRegistry.match_capability(argv)
  -> build CommandRequest
  -> ToolRuntimePlan::RunCommand
```

验收：

- 模型不能自报 capability。
- 模型不能自报 cwd / sandbox / network policy。
- `CapabilityKind` 必须由 argv 匹配得出。
- unknown command fail closed。

### 5. Insert Approval Gate

目标：command tool 执行前必须经过 approval decision。

路径：

```text
CommandRequest + MatchedCapability
  -> decide_approval
  -> ApprovalRequirement
```

验收：

- `Forbidden`：runner 不执行，返回 denied observation。
- `NeedsApproval`：Phase 1 可以用 scripted approval state 表达 accepted / rejected。
- `Skip { bypass_sandbox: false }`：继续 sandbox first。
- `Skip { bypass_sandbox: true }`：直接 no-sandbox attempt。

### 6. Insert Sandbox First And Retry

目标：复用 Slice 4 / Slice 5。

路径：

```text
Approval passed
  -> choose ExecutionAttempt
  -> SimulatedSandboxRunner.run
  -> if SandboxDenied: decide_retry
  -> optional NoSandboxRetry
```

验收：

- `CommandFailed` 直接失败，不 retry。
- `SandboxDenied + DoNotRetry` 失败。
- `SandboxDenied + RetryWithApproval rejected` 失败。
- `SandboxDenied + RetryWithApproval accepted` 进行 no-sandbox retry。
- 单个 tool call 最多一次 sandbox first + 一次 no-sandbox retry。

### 7. Extend Fake LLM Tests

目标：默认测试不依赖真实模型和网络。

建议新增：

- model proposes `add/sub` -> pure function path success。
- model proposes `run_command cat package.json` -> approval skip -> sandbox success -> observation。
- model proposes dangerous shell -> forbidden -> runner not called。
- `npm install` sandbox denied -> retry approval accepted -> no-sandbox success。
- command failed -> no retry。
- 多 tool call 中前一个 denied -> 后续 skipped observation。

## Stop Rules

- 不做真实 OS sandbox。
- 不做真实用户审批 UI。
- 不做 tool 并发执行。
- 不把 `add/sub` 改造成 command。
- 不扩展多 provider message 抽象。

## Completion Criteria

- [x] `react.rs` 通过 `ToolRuntime` 执行工具。
- [x] pure function path 仍可用。
- [ ] 至少一个 command tool 走完整 approval / sandbox / retry 链路。
- [ ] fake LLM tests 覆盖 forbidden、sandbox success、sandbox denied retry、command failed no retry。
- [ ] event stream 能解释 tool 为什么没执行、为什么 retry、最终 observation 是什么。
- [ ] `.daedalus/outcome-map.md` 和 `.daedalus/todo.md` 同步 Slice 6 integration 状态。
