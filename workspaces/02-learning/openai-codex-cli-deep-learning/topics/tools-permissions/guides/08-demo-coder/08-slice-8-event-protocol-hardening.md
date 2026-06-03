# Slice 8 Event Protocol Hardening

## Learning Navigation

- Final artifact: [`../../demo/README.md`](../../demo/README.md) 中的一次命令安全链路 event trace。
- Current stage: `08-demo-coder`
- Current slice: Slice 8 Event Protocol Hardening
- Current gap: approval / execution attempt / retry / denied / skipped 等内部安全节点还没有稳定变成外部可观察事件。
- Evidence needed: [`../../demo/src/agent/react.rs`](../../demo/src/agent/react.rs)、[`../../demo/src/agent/stream_event.rs`](../../demo/src/agent/stream_event.rs)、[`../../demo/src/model/event.rs`](../../demo/src/model/event.rs)、[`../../demo/src/tool/runtime.rs`](../../demo/src/tool/runtime.rs)、[`../../demo/src/tool/shell/mod.rs`](../../demo/src/tool/shell/mod.rs)、[`../../demo/src/tool/shell/retry.rs`](../../demo/src/tool/shell/retry.rs)。
- After this: Slice 9 可以基于稳定事件协议讨论 multi-tool hard-deny 和 skipped semantics，`demo/README.md` 可以解释一次命令为什么被允许、拒绝、sandbox retry 或结束。

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
- Steps:
  1. 先盘点现有 `StreamEvent` / `AgentEvent` / `ToolRuntimeResult` / `RunCommandResult`，确认哪些枚举已经能复用。
  2. 决定最小事件集合：approval resolved / denied、execution started / finished、retry evaluated、tool observation produced。
  3. 给 `ToolRuntime` 或 `run_shell_command` 增加事件出口，但避免让 shell 内部 helper 泄漏到 ReAct 层。
- Verify:
  - 单测覆盖 safe command 成功 trace。
  - 单测覆盖 dangerous command denied trace。
  - 单测覆盖 sandbox denied 后 retry trace。
  - ReAct 层 observation 仍能进入下一轮 LLM messages。

## Design Questions

1. 外部观察者到底需要知道“审批结论”，还是只需要知道“工具真实执行开始/结束”？
2. 哪些事件应该直接复用现有领域枚举，哪些应该转成更稳定的外部 event payload？
3. 如果未来接入 `OsExecutionRunner`，当前事件协议是否还能表达 sandbox-first 和 no-sandbox retry 的差异？

## Stop Rules

- 不做完整 UI rendering。
- 不做 session approval persistence。
- 不处理 multi-tool hard-deny 的 skipped 传播；这留给 Slice 9。
- 不把每个内部函数调用都暴露成 event。
