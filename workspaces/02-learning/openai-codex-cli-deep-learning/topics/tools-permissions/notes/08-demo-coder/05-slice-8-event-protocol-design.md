# Slice 8 Event Protocol Design

本文记录 Slice 8 Event Protocol Hardening 的第一轮设计讨论：如何把 approval / execution attempt / retry / observation 从内部编排结果，变成外部可观察、可测试、可解释的事件协议。

## Learning Navigation

- Final artifact: [`../../demo/README.md`](../../demo/README.md) 中的一次命令安全链路 event trace。
- Current stage: `08-demo-coder`
- Current slice: Slice 8 Event Protocol Hardening
- Current gap: approval 事件已通过 broker 形成往返；command attempt / retry decision 还没有稳定透出为外部事件。
- Evidence path: [`../../demo/src/agent/stream_event.rs`](../../demo/src/agent/stream_event.rs)、[`../../demo/src/model/event.rs`](../../demo/src/model/event.rs)、[`../../demo/src/tool/runtime.rs`](../../demo/src/tool/runtime.rs)、[`../../demo/src/tool/shell/mod.rs`](../../demo/src/tool/shell/mod.rs)、[`../../demo/src/agent/react.rs`](../../demo/src/agent/react.rs)。

## User Proposal

用户提出需要补充几个事件：

```text
CommandNeedsApproval
CommandApprovalResult
ToolRunStarted / ToolRunFailed / ToolRunFinished 增加 in_sandbox 参数
```

同时，`ToolRuntime` 需要支持事件传递能力：

```text
传出去：
  - CommandNeedsApproval
  - ToolRunStarted
  - ToolRunFailed
  - ToolRunFinished

接进来：
  - 接收 CommandApprovalResult
  - 根据审批是否通过继续执行
```

用户初步思路是新增一对 channel：

```text
runtime -> 外部：发送需要审批、执行开始/结束/失败等事件
外部 -> runtime：发送审批结果，runtime 再继续执行
```

## Agent Calibration

这个方向是对的，因为 approval 让工具执行不再只是同步返回：

```text
plan tool call
  -> emit needs approval
  -> wait for external decision
  -> emit approval result
  -> continue or deny
```

但需要校准两个点。

### 1. `in_sandbox` 不宜直接挂在高层 `ToolRunStarted`

当前 [`stream_event.rs`](../../demo/src/agent/stream_event.rs) 里的 `ToolRunStarted` 是高层 tool-call 事件，含义是“这个 tool call 开始执行”。但一个 `run_command` 内部可能经历多次真实 execution attempt：

```text
ToolRunStarted(run_command)
  -> CommandExecutionStarted(SandboxFirst)
  -> CommandExecutionFailed(SandboxFirst)
  -> CommandRetryEvaluated(...)
  -> CommandExecutionStarted(NoSandboxRetry)
  -> CommandExecutionFinished(NoSandboxRetry)
ToolRunFinished(run_command)
```

如果只给高层 `ToolRunStarted` 加 `in_sandbox: bool`，会表达不清：

```text
同一个 tool call 先 in_sandbox = true
然后 retry 时又 in_sandbox = false
```

更好的做法是保留高层 tool run 事件，再新增 execution-attempt 级事件，并复用 [`ExecutionAttempt`](../../demo/src/model/event.rs)：

```rust
CommandExecutionStarted {
    index,
    call_id,
    name,
    attempt: ExecutionAttempt,
}

CommandExecutionFinished {
    index,
    call_id,
    name,
    attempt: ExecutionAttempt,
    output,
}

CommandExecutionFailed {
    index,
    call_id,
    name,
    attempt: ExecutionAttempt,
    error,
}
```

这样比 `in_sandbox: bool` 更贴近已有模型：

```text
SandboxFirst
NoSandboxFirst
NoSandboxRetry
```

`bool` 只能表达是否在 sandbox，`ExecutionAttempt` 能表达为什么这次不在 sandbox。

### 2. 审批结果要区分 control input 和 observable event

`CommandNeedsApproval` 是对外事件：

```text
runtime -> UI / test / controller
```

审批结果首先是控制输入：

```text
UI / test / controller -> runtime
```

但 runtime 收到审批结果后，还应该再向外发出 `CommandApprovalResult`，让 trace 完整：

```text
CommandNeedsApproval
  -> external decision input
  -> CommandApprovalResult
  -> continue / deny
```

所以 `CommandApprovalResult` 不能只被理解为“接进来”的值，它也应该是外部可观察事件。

## Candidate Event Set

最小可行事件集合：

```rust
CommandNeedsApproval {
    approval_id: String,
    index: i64,
    call_id: String,
    name: String,
    reason: String,
    scope: ApprovalScope,
}

CommandApprovalResult {
    approval_id: String,
    index: i64,
    call_id: String,
    name: String,
    decision: UserApprovalDecision,
}

CommandExecutionStarted {
    index: i64,
    call_id: String,
    name: String,
    attempt: ExecutionAttempt,
}

CommandExecutionFinished {
    index: i64,
    call_id: String,
    name: String,
    attempt: ExecutionAttempt,
    output: String,
}

CommandExecutionFailed {
    index: i64,
    call_id: String,
    name: String,
    attempt: ExecutionAttempt,
    error: String,
}

CommandRetryEvaluated {
    index: i64,
    call_id: String,
    name: String,
    decision: RetryDecision,
}
```

`approval_id` 很重要。未来如果同一轮有多个 tool call，或者后续允许并发执行，审批结果必须能准确回到对应请求。

## Implementation Checkpoint

当前代码已经完成了 Slice 8 的第一步落地：

- [`stream_event.rs`](../../demo/src/agent/stream_event.rs) 已使用 `CommandNeedsApproval`、`CommandApprovalResult`、`CommandExecutionStarted/Finished/Failed`、`CommandRetryEvaluated`，避免把 command attempt 误说成所有 tool 的 execution。
- [`approval.rs`](../../demo/src/tool/shell/approval.rs) 已实现 `ApprovalBroker`：发出 `CommandNeedsApproval`，通过 `approval_id` 和 pending oneshot 等待 `CommandApprovalResult`。
- [`runtime.rs`](../../demo/src/tool/runtime.rs) 已从 `ToolCallFinished` 构造 `ToolCallContext`，并把 `index / call_id / tool_name` 传入 `run_shell_command`。
- [`shell/mod.rs`](../../demo/src/tool/shell/mod.rs) 已在初始 approval 和 retry approval 两处传递 `ToolApprovalRequest`。

还没有完成的是：`CommandApprovalResult` 目前主要是 broker 的控制输入，尚未作为外部 trace 事件重新广播；`CommandExecution*` 和 `CommandRetryEvaluated` 也只是事件类型，还没有从 `run_shell_command` 的 sandbox-first、no-sandbox retry 和 retry gate 分支真实发出。

## Minimal Credible Trace

外部观察者不需要看到所有内部 helper，但至少需要看到这条链：

```text
ToolCallFinished
  -> CommandNeedsApproval
  -> CommandApprovalResult
  -> CommandExecutionStarted(SandboxFirst)
  -> CommandExecutionFailed(SandboxFirst)
  -> CommandRetryEvaluated(RetryWithApproval / RetryWithoutApproval / DoNotRetry)
  -> CommandExecutionStarted(NoSandboxRetry)
  -> CommandExecutionFinished(NoSandboxRetry)
  -> ToolRunFinished
```

这条 trace 能证明：

```text
tool call 没有直接裸跑
  -> 它经过 approval gate
  -> 它先尝试 sandbox
  -> sandbox denied 后经过 retry gate
  -> no-sandbox retry 有明确原因
  -> observation 再回灌给 ReAct loop
```

## Implementation Direction

建议分步落地：

1. 先新增事件类型，不急着改 channel。
2. 让 `ToolRuntime::run` 或后续 `ToolRuntime::run_with_events` 能接收 event sink。
3. 在 `run_shell_command` 中围绕关键节点 emit：
   - `CommandNeedsApproval`
   - `CommandApprovalResult`
   - `CommandExecutionStarted`
   - `CommandExecutionFinished`
   - `CommandExecutionFailed`
   - `CommandRetryEvaluated`
4. 最后再把 event sink 接入 ReAct 外部 stream，并用 trace tests 验证 safe read、sandbox denied retry、dangerous denied。

## Critical Lens

- Source constraint: Codex 的本地命令执行需要让用户、UI 和测试都能看见权限与沙箱链路，而不是只看到最终 tool output。
- Faithful imitation: demo 应模仿 Codex 的分层安全链路，让 approval、sandbox attempt、retry decision、observation 都可观察。
- Simplified / improved / discarded: Phase 1 不复刻 Codex 完整 UI/TUI event system，只保留解释安全不变量所需的事件。
- Transfer risk: 事件过细会把内部 helper 泄漏成公共协议；事件过粗又无法解释安全决策。迁移时要重新确认外部观察者到底需要哪些稳定事件。

## Current Decision

本轮暂定方向：

```text
不要给高层 ToolRunStarted 只加 in_sandbox bool。
改为新增 execution-attempt 级事件，并复用 ExecutionAttempt。
approval channel 已落地为 ApprovalBroker；下一步只补 command attempt / retry 事件出口。
```
