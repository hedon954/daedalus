# 产物索引

记录学习过程中产生的产物，以及它们服务的学习目的。

> `状态` 列只能使用 `草稿`、`已验证`、`待补充`、`不适用`。不要发明新的状态值；如需扩展，先更新模板、CLI 测试和相关提示词。

| 产物 | 阶段 | 目的 | 状态 |
| --- | --- | --- | --- |
| [`.daedalus/task-card.md`](task-card.md) | 01-goal-aligner | 学习目标与验收标准 | 草稿 |
| [`.daedalus/validation-log.md`](validation-log.md) | all | 记录 daedalus 教学引导效果与改进点 | 草稿 |
| [`guides/`](../guides) | all | Agent 生成的行动指南与验收清单 | 草稿 |
| [`notes/`](../notes) | all | 用户亲自实践后的学习笔记 | 草稿 |
| [`guides/01-goal-alignment-guide.md`](../guides/01-goal-alignment-guide.md) | 01-goal-aligner | 记录目标对齐结果和下一步确认项 | 草稿 |
| [`guides/02-repo-selection-guide.md`](../guides/02-repo-selection-guide.md) | 02-repo-scout | 对用户指定的 Codex CLI 仓库做客观选择评估 | 草稿 |
| [`notes/repo-selection.md`](../notes/repo-selection.md) | 02-repo-scout | 用户确认后的 repo 选择、风险接受和源码准备证据 | 草稿 |
| [`source/pull_source.sh`](../source/pull_source.sh) | 02-repo-scout | 可复现拉取 OpenAI Codex CLI 源码，默认固定到已拉取 commit | 草稿 |
| [`guides/03-question-roadmap-guide.md`](../guides/03-question-roadmap-guide.md) | 03-socratic-coach | 第一轮问题设计意图和验证路径 | 草稿 |
| [`notes/question-roadmap.md`](../notes/question-roadmap.md) | 03-socratic-coach | 用户回答和后续可验证假设 | 已验证 |
| [`guides/04-run-debug-guide.md`](../guides/04-run-debug-guide.md) | 04-debugger-guide | `codex exec` 最小运行与调试指南 | 草稿 |
| [`notes/runbook.md`](../notes/runbook.md) | 04-debugger-guide | 用户运行命令、观察输出和核心链路验证 | 已验证 |
| [`source/codex/codex-rs/.vscode/launch.json`](../source/codex/codex-rs/.vscode/launch.json) | 04-debugger-guide | Cursor/CodeLLDB 调试交互式 Codex TUI 与 exec 路径 | 草稿 |
| [`notes/architecture.md`](../notes/architecture.md) | 05-arch-analyzer | 架构阶段标准入口，指向最终架构分析笔记 | 已验证 |
| [`notes/codex-agent-loop-architecture.md`](../notes/codex-agent-loop-architecture.md) | 05-arch-analyzer | Codex 交互循环、core agent loop、工具分发、权限审批、沙箱执行和关键 struct 分层最终结论；作为 `notes/architecture.md` 的等价架构产物 | 已验证 |
| [`guides/06-code-reader-guide.md`](../guides/06-code-reader-guide.md) | 06-code-reader | auth/approval/sandbox 专题的工业问题驱动阅读问题、回答格式和源码验证路径 | 草稿 |
| [`notes/code-reading.md`](../notes/code-reading.md) | 06-code-reader | 核心代码阅读标准入口，按生产问题、naive 失败、源码应对、不变量和 trade-off 记录专题阅读 | 草稿 |
| [`notes/codex-context-and-compaction.md`](../notes/codex-context-and-compaction.md) | 06-code-reader | Codex 上下文管理、history、rollout、prompt 构建、工具结果回灌和 compact 机制 | 已验证 |
| [`notes/diagrams/codex-core-agent-loop-flow.excalidraw`](../notes/diagrams/codex-core-agent-loop-flow.excalidraw) | 04-debugger-guide | Codex 核心 Agent loop 标准流程图 | 草稿 |
| [`notes/diagrams/codex-architecture-layers.excalidraw`](../notes/diagrams/codex-architecture-layers.excalidraw) | 04-debugger-guide | Codex 架构分层可编辑图 | 草稿 |
| [`notes/diagrams/codex-tool-approval-sandbox.excalidraw`](../notes/diagrams/codex-tool-approval-sandbox.excalidraw) | 04-debugger-guide | Codex tool system、权限审批与沙箱执行可编辑图 | 草稿 |
| [`.daedalus/long-context.md`](long-context.md) | all | 当前学习状态、已验证结论、未解决问题和恢复提示 | 草稿 |
