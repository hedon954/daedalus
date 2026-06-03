# Slice 8 Event Protocol Design

本文记录 Slice 8 Event Protocol Hardening 的第一轮设计讨论：如何把 approval / execution attempt / retry / observation 从内部编排结果，变成外部可观察、可测试、可解释的事件协议。

## Learning Navigation

- Final artifact: [`../../demo/README.md`](../../demo/README.md) 中的一次命令安全链路 event trace。
- Current stage: `08-demo-coder`
- Current slice: Slice 8 Event Protocol Hardening
- Current gap: approval / sandbox-first / retry / denied 等安全节点还没有稳定透出为外部事件。
- Evidence path: [`../../demo/src/agent/stream_event.rs`](../../demo/src/agent/stream_event.rs)、[`../../demo/src/model/event.rs`](../../demo/src/model/event.rs)、[`../../demo/src/tool/runtime.rs`](../../demo/src/tool/runtime.rs)、[`../../demo/src/tool/shell/mod.rs`](../../demo/src/tool/shell/mod.rs)、[`../../demo/src/agent/react.rs`](../../demo/src/agent/react.rs)。

## User Proposal

用户提出需要补充几个事件：

```text
ToolNeedsApproval
ToolApprovalResult
ToolRunStarted / ToolRunFailed / ToolRunFinished 增加 in_sandbox 参数
```

同时，`ToolRuntime` 需要支持事件传递能力：

```text
传出去：
  - ToolNeedsApproval
  - ToolRunStarted
  - ToolRunFailed
  - ToolRunFinished

接进来：
  - 接收 ToolApprovalResult
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
  -> ToolExecutionStarted(SandboxFirst)
  -> ToolExecutionFailed(SandboxFirst)
  -> ToolRetryEvaluated(...)
  -> ToolExecutionStarted(NoSandboxRetry)
  -> ToolExecutionFinished(NoSandboxRetry)
ToolRunFinished(run_command)
```

如果只给高层 `ToolRunStarted` 加 `in_sandbox: bool`，会表达不清：

```text
同一个 tool call 先 in_sandbox = true
然后 retry 时又 in_sandbox = false
```

更好的做法是保留高层 tool run 事件，再新增 execution-attempt 级事件，并复用 [`ExecutionAttempt`](../../demo/src/model/event.rs)：

```rust
ToolExecutionStarted {
    index,
    call_id,
    name,
    attempt: ExecutionAttempt,
}

ToolExecutionFinished {
    index,
    call_id,
    name,
    attempt: ExecutionAttempt,
    output,
}

ToolExecutionFailed {
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

`ToolNeedsApproval` 是对外事件：

```text
runtime -> UI / test / controller
```

审批结果首先是控制输入：

```text
UI / test / controller -> runtime
```

但 runtime 收到审批结果后，还应该再向外发出 `ToolApprovalResult`，让 trace 完整：

```text
ToolNeedsApproval
  -> external decision input
  -> ToolApprovalResult
  -> continue / deny
```

所以 `ToolApprovalResult` 不能只被理解为“接进来”的值，它也应该是外部可观察事件。

## Candidate Event Set

最小可行事件集合：

```rust
ToolNeedsApproval {
    approval_id: String,
    index: i64,
    call_id: String,
    name: String,
    reason: String,
    scope: ApprovalScope,
}

ToolApprovalResult {
    approval_id: String,
    index: i64,
    call_id: String,
    name: String,
    decision: UserApprovalDecision,
}

ToolExecutionStarted {
    index: i64,
    call_id: String,
    name: String,
    attempt: ExecutionAttempt,
}

ToolExecutionFinished {
    index: i64,
    call_id: String,
    name: String,
    attempt: ExecutionAttempt,
    output: String,
}

ToolExecutionFailed {
    index: i64,
    call_id: String,
    name: String,
    attempt: ExecutionAttempt,
    error: String,
}

ToolRetryEvaluated {
    index: i64,
    call_id: String,
    name: String,
    decision: RetryDecision,
}
```

`approval_id` 很重要。未来如果同一轮有多个 tool call，或者后续允许并发执行，审批结果必须能准确回到对应请求。

## Minimal Credible Trace

外部观察者不需要看到所有内部 helper，但至少需要看到这条链：

```text
ToolCallFinished
  -> ToolNeedsApproval
  -> ToolApprovalResult
  -> ToolExecutionStarted(SandboxFirst)
  -> ToolExecutionFailed(SandboxFirst)
  -> ToolRetryEvaluated(RetryWithApproval / RetryWithoutApproval / DoNotRetry)
  -> ToolExecutionStarted(NoSandboxRetry)
  -> ToolExecutionFinished(NoSandboxRetry)
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
   - `ToolNeedsApproval`
   - `ToolApprovalResult`
   - `ToolExecutionStarted`
   - `ToolExecutionFinished`
   - `ToolExecutionFailed`
   - `ToolRetryEvaluated`
4. 最后再把 `ApprovalDecider` 从同步 fake/scripted 升级为 channel-based decider。

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
approval channel 可以作为后续实现方向，但先把事件协议和事件出口设计清楚。
```
