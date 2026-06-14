# Slice 8 Event Protocol Hardening

## Learning Navigation

- Final artifact: [`../../demo/README.md`](../../demo/README.md) 中的一次命令安全链路 event trace。
- Current stage: `08-demo-coder`
- Current slice: Slice 8 Event Protocol Hardening
- Current gap: 已完成。approval request、CommandExecution、CommandRetry 事件已透出；run-scoped approval request、send failure fail closed、pending cleanup、`CommandEventEmitter`、retry 成功路径 attempt trace 和 `run_execution_attempt` 已闭合。
- Evidence needed: [`../../demo/src/agent/react.rs`](../../demo/src/agent/react.rs)、[`../../demo/src/agent/stream_event.rs`](../../demo/src/agent/stream_event.rs)、[`../../demo/src/model/event.rs`](../../demo/src/model/event.rs)、[`../../demo/src/tool/runtime.rs`](../../demo/src/tool/runtime.rs)、[`../../demo/src/tool/shell/mod.rs`](../../demo/src/tool/shell/mod.rs)、[`../../demo/src/tool/shell/retry.rs`](../../demo/src/tool/shell/retry.rs)。
- After this: Slice 9 可以基于稳定事件协议实现 multi-tool independent execution。

## Critical Lens

- Source fact: Codex 把工具执行拆成 approval、sandbox、retry、tool observation 等多个安全节点，而不是把 tool call 直接等同于函数调用。
- Source assumption under test: Codex 的内部事件/observation 边界是否适合我们 demo 直接模仿，还需要对照当前 demo 的 `StreamEvent` 与 `ToolRuntimeResult`。
- Strength: 外部可观察事件能让用户和测试理解“为什么这个命令被允许、拒绝或重试”，降低权限系统黑箱感。
- Limitation / failure mode: 事件过细会把内部 helper 结构暴露成公共协议，后续重构成本变高；事件过粗又无法解释安全决策。
- Faithful imitation: 先模仿 Codex 的分层安全链路，让 event trace 能反映 approval / sandbox-first / retry decision / observation 回灌。
- Transfer decision: Phase 1 只暴露足以解释安全不变量的事件，不复刻 Codex 完整 UI/TUI 事件系统。
- Not-to-copy: 不把每个内部函数边界都变成事件；不为追求“像 Codex”而制造难以维护的平行 event model。

## Action Card

- Goal: 让 `run_command` 的关键安全节点进入外部 stream，而不是只存在于内部测试和返回值里。
- Why: `demo/README.md` 需要展示一条可解释 trace，证明 tool call 经过了 approval、sandbox first、retry 和 observation，而不是直接执行。
- Current checkpoint:
  - 已完成：`StreamEvent` 包含 `CommandNeedsApproval`、`CommandExecutionStarted/Finished/Failed`、`CommandRetryEvaluated`。
  - 已完成：`ApprovalGateway + PendingApproval` 使用 internal `ToolApprovalResult` channel + pending oneshot，把审批请求和审批结果配对；`CommandNeedsApproval` 由当前 run 的 `EventSender` 发出。
  - 已验证：`cargo test` 通过 63 个默认测试，3 个 live LLM 测试 ignored。
  - 已完成：`CommandEventEmitter` 已集中填充 command event 的 `index / call_id / name`。
  - 已完成：`run_execution_attempt` 已集中保证 attempt lifecycle，不再由业务分支手写 started / terminal event。
- Steps:
  1. 已完成：引入 `run_execution_attempt`，统一保证每个 attempt 都有 started 和 terminal event。
  2. 已完成：替换 `SandboxFirst`、`NoSandboxFirst`、`NoSandboxRetry` 的手写 started / terminal event 逻辑。
  3. 已完成：approval request 通过当前 run stream 发出，不让 `ApprovalGateway` 长期持有外部 stream sender。
  4. 已验证：trace 测试覆盖 safe read、network install sandbox denied -> retry、dangerous shell denied；ReAct observation 回灌未被破坏。
- Verify:
  - 单测覆盖 safe command 成功 trace。
  - 单测覆盖 dangerous command denied trace。
  - 单测覆盖 sandbox denied 后 retry trace，并确认 `SandboxFirst` 失败事件先于 `NoSandboxRetry`。
  - ReAct 层 observation 仍能进入下一轮 LLM messages。

## Design Questions

1. 外部观察者到底需要知道“审批结论”，还是只需要知道“工具真实执行开始/结束”？
2. 哪些事件应该直接复用现有领域枚举，哪些应该转成更稳定的外部 event payload？
3. 如果未来接入 `OsExecutionRunner`，当前事件协议是否还能表达 sandbox-first 和 no-sandbox retry 的差异？

## Stop Rules

- 不做完整 UI rendering。
- 不做 session approval persistence。
- 不处理 multi-tool independent execution；这留给 Slice 9。
- 不把每个内部函数调用都暴露成 event。
