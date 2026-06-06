# Slice 8 Event Protocol Design

## Learning Navigation

- Final artifact: [`../../demo/README.md`](../../demo/README.md) 中的一次命令安全链路 event trace。
- Current stage: `08-demo-coder`
- Current slice: Slice 8 Event Protocol Hardening
- Current gap: approval request、command attempt、retry decision 已开始透出为外部事件；review 后确认下一步要统一事件出口并保证 attempt trace 闭合。
- Evidence path: [`../../demo/src/agent/stream_event.rs`](../../demo/src/agent/stream_event.rs)、[`../../demo/src/tool/shell/approval.rs`](../../demo/src/tool/shell/approval.rs)、[`../../demo/src/tool/runtime.rs`](../../demo/src/tool/runtime.rs)、[`../../demo/src/tool/shell/mod.rs`](../../demo/src/tool/shell/mod.rs)。

## Current Scheme

`StreamEvent` 只承载外部观察者需要看到的事件：

```text
ToolCallFinished
CommandNeedsApproval
CommandExecutionStarted
CommandExecutionFinished
CommandExecutionFailed
CommandRetryEvaluated
ToolRunFinished / ToolRunFailed
```

审批结果不是 `StreamEvent`。它是 `ApprovalBroker` 的内部控制消息：

```text
ToolApprovalResult {
  approval_id,
  decision,
}
```

也就是说：

```text
CommandNeedsApproval: runtime -> 外部观察者 / UI
ToolApprovalResult: UI / test -> ApprovalBroker
UserApprovalDecision: ApprovalBroker -> run_shell_command
```

这样可以保持两个边界清晰：

- 对外事件流只解释 agent 和 command 执行发生了什么。
- 审批回流只负责唤醒等待中的 approval request，不占用对外事件协议。

## Minimal Trace

一条需要审批并发生 sandbox retry 的 command trace 至少应该长这样：

```text
ToolCallFinished(run_command)
  -> CommandNeedsApproval
  -> CommandExecutionStarted(SandboxFirst)
  -> CommandExecutionFailed(SandboxFirst)
  -> CommandRetryEvaluated(RetryWithApproval / RetryWithoutApproval / DoNotRetry)
  -> CommandExecutionStarted(NoSandboxRetry)
  -> CommandExecutionFinished(NoSandboxRetry)
  -> ToolRunFinished(run_command)
```

危险命令 trace 应该更短：

```text
ToolCallFinished(run_command)
  -> ToolRunFailed(tool denied)
```

安全读命令 trace 应该能证明它走了 sandbox-first：

```text
ToolCallFinished(run_command)
  -> CommandExecutionStarted(SandboxFirst)
  -> CommandExecutionFinished(SandboxFirst)
  -> ToolRunFinished(run_command)
```

## Next Implementation Target

下一步只补事件出口：

1. 给 `ToolRuntime` / command runtime 增加 outbound event sink。
2. 在 `run_shell_command` 发出 `CommandExecution*` 和 `CommandRetryEvaluated`。
3. 补 safe read、sandbox denied retry、dangerous denied 三类 trace tests。

## Critical Lens

- 保留 `ToolRun*` 表达模型 tool call 的整体结果。
- 使用 `CommandExecution*` 表达 `run_command` 内部的一次具体 attempt。
- 不把审批结果设计成对外事件，避免把控制消息和观察事件混在一起。
