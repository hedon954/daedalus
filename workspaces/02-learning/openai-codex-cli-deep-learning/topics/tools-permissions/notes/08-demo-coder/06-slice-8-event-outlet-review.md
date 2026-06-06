# Slice 8 Event Outlet Review

## Learning Navigation

- Final artifact: [`../../demo/README.md`](../../demo/README.md) 中的一次 command 安全执行 trace。
- Current stage: `08-demo-coder`
- Current slice: Slice 8 Event Protocol Hardening
- Current gap: command approval / execution / retry 事件已经开始透出；stream 绑定、错误处理风险和 emitter 上下文散落问题已修复，剩余问题是 attempt lifecycle 仍由各分支手写。
- Paired guide: [`../../guides/08-demo-coder/09-slice-8-command-event-emitter-refactor.md`](../../guides/08-demo-coder/09-slice-8-command-event-emitter-refactor.md)

## Review Findings

本轮最新验证结果：

```text
cargo test --manifest-path workspaces/02-learning/openai-codex-cli-deep-learning/topics/tools-permissions/demo/Cargo.toml
-> 63 passed, 3 ignored

git diff --check
-> passed
```

测试通过说明当前主链路可以跑通。上一轮 review 暴露的 stream 绑定、send failure 和 attempt trace 问题已经闭合；当前剩余的是代码结构问题，不是功能阻塞。

## What Is Now True

当前实现已经有这些事实：

- `StreamEvent` 包含 `CommandNeedsApproval`、`CommandExecutionStarted`、`CommandExecutionFinished`、`CommandExecutionFailed`、`CommandRetryEvaluated`。
- `ToolApprovalResult` 已从外部 `StreamEvent` 中移除，作为内部审批回传控制消息。
- `ApprovalGateway + PendingApproval` 已取代旧 `ApprovalController / ApprovalBroker` 事件绑定。
- `run_shell_command` 已经能在 command path 中发 command-level event，并且 `CommandNeedsApproval` 会进入当前 run 的 event stream。
- `CommandEventEmitter` 已抽取，统一填充 command event 的 `index / call_id / name`。
- `ToolRuntime` 已把 `run_command` 的 command path 接入 ReAct 外部 stream。

这个方向是对的：外部观察者看见安全链路，内部控制消息不污染公共事件协议。

## Problems Found And Current Status

### 1. Approval event sender is not run-scoped

`ApprovalBroker` 当前在构造时持有一个 `EventSender`。但 `ReActAgent::run()` 每次 run 都会创建新的 stream channel。

这会导致一个结构性风险：`CommandNeedsApproval` 可能被发到 broker 初始化时捕获的 sender，而不是当前 `agent.run()` 返回给外部观察者的 stream。

这不是单测小问题，而是职责边界问题：

```text
current run stream
  -> should receive CommandNeedsApproval

ApprovalBroker captured sender
  -> may be a different stream
```

状态：已修复。

当前方案：

```text
ApprovalGateway
  -> create_pending_approval
  -> wait ToolApprovalResult by approval_id

run_shell_command
  -> send CommandNeedsApproval through current run EventSender
  -> wait PendingApproval
```

事件属于当前 run，审批等待属于 gateway。二者已经分离。

### 2. Approval event send failure can panic

旧实现中 approval request 事件发送失败会 panic 或继续等待一个外界永远看不到的 pending approval。

如果外部 stream receiver 已经 drop，当前行为会 panic，并且 pending approval 已经插入。更合理的行为应该是 fail closed：

```text
cannot publish approval request
  -> remove pending approval
  -> reject / return controlled error
```

安全链路里，事件无法发出不应导致裸跑，也不应该让后台 task 因 panic 丢失上下文。

状态：已修复。当前发送失败会 fail closed，并清理 pending。

### 3. SandboxFirst attempt is not closed on retry success paths

旧 retry 成功路径可能出现：

```text
CommandExecutionStarted(SandboxFirst)
CommandRetryEvaluated(...)
CommandExecutionStarted(NoSandboxRetry)
CommandExecutionFinished(NoSandboxRetry)
```

但缺少：

```text
CommandExecutionFailed(SandboxFirst)
```

如果我们的事件契约是“每一次 execution attempt 都必须有 started 和 terminal event”，那这个 trace 是不闭合的。外部 observer 很难判断第一次 sandbox attempt 到底失败了、被取消了，还是被 retry event 隐式覆盖了。

状态：已修复。shell / ReAct trace tests 已锁定：

```text
CommandExecutionStarted(SandboxFirst)
CommandExecutionFailed(SandboxFirst)
CommandRetryEvaluated(...)
CommandExecutionStarted(NoSandboxRetry)
CommandExecutionFinished(NoSandboxRetry)
```

### 4. Event emission is a cross-cutting concern but is handwritten everywhere

状态：已部分修复。`CommandEventEmitter` 已经集中 event construction，`run_shell_command` 不再反复填 `index / call_id / name`。

当前仍然存在的是 attempt lifecycle 没有被 helper 结构化保护：业务分支仍要记得在 runner 前发 `execution_started`，并在 runner 后发 `execution_finished` 或 `execution_failed`。

```text
tx.send(Ok(StreamEvent::CommandExecutionStarted { index, call_id, name, ... }))
```

这带来三个问题：

- 重复填充 `index / call_id / name`，容易漏字段或错字段。
- 每个分支都要记得发 terminal event，容易遗漏。
- 业务控制流和观察协议混在一起，后续改 event payload 会牵动所有分支。

用户指出“事件透出有点恶心”，本质上就是这个横切关注点没有集中处理。

## Design Lesson

事件不是附属日志。对于安全执行链路，事件是对外证明：

```text
tool call did not run directly
  -> capability / approval / sandbox / retry gates were actually visited
```

因此事件协议需要被当成一个小型领域层来设计，而不是在各个分支里临时 `send`。

## Updated Invariant

后续 Slice 8 应把事件透出收敛为两个不变量：

1. **Run-scoped event invariant**
   当前 run 产生的 approval / execution / retry event 必须进入当前 run 返回的 `EventStream`。

2. **Attempt lifecycle invariant**
   每个 `ExecutionAttempt` 一旦发出 `CommandExecutionStarted`，必须最终发出 `CommandExecutionFinished` 或 `CommandExecutionFailed`。

## Minimal Next Move

下一步不继续到处补事件发送，也不再讨论是否需要 `CommandEventEmitter`。用户已确认 `run_execution_attempt` 有必要，因为生命周期不变量应该由代码结构保证。

下一步引入：

```text
run_execution_attempt
  -> emit execution started
  -> runner.run(...)
  -> emit execution finished / failed
  -> return ExecutionResult
```

这样事件闭合由 helper 保证，业务分支只处理 approval / retry decision。
