# 08 Demo Coder Notes

本目录记录 `08-demo-coder` 阶段的用户实践问题、Agent 校准、实现取舍和验收反馈。

README 只做索引；具体专题写入单独文件，避免把 demo 实现过程压成一份不可导航的大笔记。

## Learning Navigation

- Final artifact: [`../../demo/README.md`](../../demo/README.md)
- Current stage: `08-demo-coder`
- Current slice: Slice 9 Multi-Tool Independent Execution
- Current path: real/fake model stream -> ReAct loop -> multiple tool calls -> independent runtime results -> stable observation feedback -> bounded next turn

## Topic Index

| Topic | File | Status | Demo impact |
| --- | --- | --- | --- |
| ReAct loop implementation issues | [`01-react-loop-implementation-issues.md`](01-react-loop-implementation-issues.md) | 用户实践中 / 已初步验证 | 决定 `react.rs` 的 streaming、tool call、event、test 边界 |
| Tool runtime boundary and multi-call policy | [`02-tool-runtime-boundary-and-multi-call-policy.md`](02-tool-runtime-boundary-and-multi-call-policy.md) | pure function path 与 command path 均已验证 / multi-call 策略已由 Slice 9 更新 | 决定 `ToolRuntime` 分层、pure tool 与 shell tool 的权限链路；Slice 9 已把多 tool call 定为 independent observation |
| Git commit message rewrite | [`03-git-commit-message-rewrite.md`](03-git-commit-message-rewrite.md) | 已记录 | 沉淀 demo 开发分支历史整理与验收方法 |
| Slice 6 command runtime boundary discussion | [`04-slice-6-command-runtime-boundary-discussion.md`](04-slice-6-command-runtime-boundary-discussion.md) | 已记录 / 待同步 guide | 沉淀 `ApprovalDecider`、`ExecutionRunner`、`NeedsApproval -> SandboxFirst`、`NoSandboxRetry`、ReAct observation 和 Slice 6/7 边界讨论 |
| Slice 8 event protocol design | [`05-slice-8-event-protocol-design.md`](05-slice-8-event-protocol-design.md) | 已实现 / 已验证 | 决定 command approval、command attempt、retry 事件和 broker-based approval 的落地方向 |
| Slice 8 event outlet review | [`06-slice-8-event-outlet-review.md`](06-slice-8-event-outlet-review.md) | 已完成 / 已收口 | 记录事件透出 review 发现：run-scoped sender、approval panic、attempt lifecycle、横切事件散落问题 |
| Slice 8 event protocol refactor retrospective | [`07-slice-8-event-protocol-refactor-retrospective.md`](07-slice-8-event-protocol-refactor-retrospective.md) | 已记录 | 复盘从散落事件发送到 `ApprovalGateway + CommandEventEmitter + run_execution_attempt` 的决策缘由 |
| Slice 9 multi-tool independent policy | [`08-slice-9-multi-tool-independent-policy.md`](08-slice-9-multi-tool-independent-policy.md) | 已决策 / 待实现 | 决定同批 tool calls 互不影响、可并发执行、按 index 稳定回灌 observation，不做 batch-level hard stop |

## Stage Exit Criteria

- [x] `react.rs` 有 deterministic fake LLM 单测覆盖多 tool call、多轮 tool call、tool failure。
- [x] live LLM 测试默认 `#[ignore]`，只作为手动验收。
- [x] `max_turns` 阻止无限 agent loop。
- [x] tool call event 和 tool run event 都能对外透出。
- [x] `ToolRuntime` pure function path 有单测覆盖 add/sub 成功、参数错误和未知工具；错误路径只断言失败存在，不锁定错误文案。
- [x] `ToolRuntime` command path 有单测覆盖 safe read 成功、network install retry 成功、command failure、dangerous shell denied、invalid JSON、unmatched capability。
- [x] demo 关键类型和函数已补齐职责注释与下一步 TODO，恢复上下文时能定位 `run_shell_command` 的剩余断点。
- [x] Slice 6 closeout 完成：guides / todo / outcome-map / design 与当前代码结构同步。
- [x] Slice 7 完成：`RetryPolicy` 已纳入 retry gate，并验证 `safe-read` 不 retry、`safe-test` approval retry、network prompt approval retry、network deny 不被 `WithoutApproval` 绕过。
- [x] Slice 8 完成：approval / command attempt / retry decision 等关键节点透出为外部可观察事件，并通过统一 emitter 和 `run_execution_attempt` 保证 trace 闭合。
