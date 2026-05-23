# 06 Code Reader Notes

本目录是 `06-code-reader` 的用户学习证据入口。README 只做阶段索引和状态导航；专题文件保存用户回答、Agent 校准、源码证据、验证状态和 demo delta。

## Learning Navigation

- Final artifact: [`demo/design.md`](../../demo/design.md)
- Current stage: `06-code-reader`
- Current path: auth / approval / sandbox -> demo command execution state machine
- Current open gaps: none; ready for `07-demo-architecture` review

## Topic Index

| Topic | File | Status | Demo impact |
| --- | --- | --- | --- |
| Legacy code-reading synthesis | [`../code-reading.md`](../code-reading.md) | legacy / active reference | auth/approval/sandbox 总结与复盘 |
| Context and compaction | [`../codex-context-and-compaction.md`](../codex-context-and-compaction.md) | 已验证 | demo 暂不实现完整 compact |
| Decision composition | [`../code-reading.md#9-decision-合成从多段命令到总审批结论`](../code-reading.md#9-decision-合成从多段命令到总审批结论) | 已映射到 demo 接口 | `approval_requirement_for_command` |
| Runtime request assembly | `runtime-request-assembly.md` | 用户已复述 / 源码已核对 | `CommandRequest` 字段来源 |
| Orchestrator retry | `orchestrator-retry.md` | 用户已复述 / 源码已核对 | `SandboxRetryState` |

## Stage Exit Criteria

- [x] Decision 合成已映射成 demo 接口。
- [x] `CommandRequest` 字段来源已确认。
- [x] sandbox denied 后的 retry 状态机已确认。
- [x] 用户能复述 auth/approval/sandbox 的核心不变量。

## Legacy Policy

旧文件保留为历史产物，不做全文拆分。后续新专题写入本目录；必要时在 README 中索引旧文件的相关小节。
