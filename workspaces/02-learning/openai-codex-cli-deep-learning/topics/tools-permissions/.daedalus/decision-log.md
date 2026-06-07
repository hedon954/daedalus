# 决策记录

记录会影响学习方向、任务范围或状态流转的关键决策。

## 2026-05-09 15:58:52

- 决策：初始化 repo 学习任务 `openai-codex-cli-deep-learning`。
- 原因：开启一次 filesystem-first 的 daedalus 学习闭环。
- 影响：当前阶段为 `01-goal-aligner`。

## 2026-05-10 13:15:00

- 决策：将任务状态推进到 `06-code-reader` active。
- 原因：`04-debugger-guide` 已有 `notes/runbook.md` 和用户运行证据；`05-arch-analyzer` 已有 `notes/codex-agent-loop-architecture.md` 作为等价架构产物。
- 影响：下一步聚焦核心代码深读；当前已完成上下文管理 / compact 专题，待选择继续深读 `auth/approval/sandbox` 或开始提炼 mini demo 不变量。

## 2026-05-11 23:44:20

- 决策：回退到 `06-code-reader` 阶段。
- 原因：用户指出当前 code-reader 仍偏机制说明，缺少工业级项目在生产环境真实失败模式下的设计压力分析。
- 影响：后续源码阅读必须按“生产问题 -> naive 失败 -> 源码应对 -> 保护的不变量 -> trade-off -> 可迁移模式”推进；已通过 `daedalus state rollback 06-code-reader` 记录状态流转。

## 2026-06-06

- 决策：Slice 9 采用 multi-tool independent execution，不采用 hard-deny 后跳过后续 tool call。
- 原因：同批 tool calls 的参数已同时确定，默认应作为独立 observation 收集；全部执行后 agent 可以一次性看到多个成功/失败/拒绝结果，减少被 skipped 后重新发起 tool call 的多轮纠错。
- 影响：`ToolRuntime::batch_run` 已承接同批 tool calls 的并发调度，`ToolEventEmitter` 已承接 tool-level lifecycle events，ReAct loop 保持 transcript observation 职责；当前不强制新增独立 `ToolBatchRunner` 类型。
