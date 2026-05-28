# 08 Demo Coder Notes

本目录记录 `08-demo-coder` 阶段的用户实践问题、Agent 校准、实现取舍和验收反馈。

README 只做索引；具体专题写入单独文件，避免把 demo 实现过程压成一份不可导航的大笔记。

## Learning Navigation

- Final artifact: [`../../demo/README.md`](../../demo/README.md)
- Current stage: `08-demo-coder`
- Current slice: Slice 6 Agent Orchestrator
- Current path: real/fake model stream -> ReAct loop -> tool decision event -> tool execution event -> observation feedback -> bounded next turn

## Topic Index

| Topic | File | Status | Demo impact |
| --- | --- | --- | --- |
| ReAct loop implementation issues | [`react-loop-implementation-issues.md`](react-loop-implementation-issues.md) | 用户实践中 / 已初步验证 | 决定 `react.rs` 的 streaming、tool call、event、test 边界 |
| Tool runtime boundary and multi-call policy | [`tool-runtime-boundary-and-multi-call-policy.md`](tool-runtime-boundary-and-multi-call-policy.md) | 已记录 / 待实现验证 | 决定 `ToolRuntime` 分层、pure tool 与 shell tool 的权限链路、多 tool call 被拒后的 observation 策略 |
| Git commit message rewrite | [`git-commit-message-rewrite.md`](git-commit-message-rewrite.md) | 已记录 | 沉淀 demo 开发分支历史整理与验收方法 |

## Stage Exit Criteria

- [x] `react.rs` 有 deterministic fake LLM 单测覆盖多 tool call、多轮 tool call、tool failure。
- [x] live LLM 测试默认 `#[ignore]`，只作为手动验收。
- [x] `max_turns` 阻止无限 agent loop。
- [x] tool call event 和 tool run event 都能对外透出。
- [ ] `demo/README.md` 能说明 Slice 6 的运行方式和 Phase 1 简化边界。
