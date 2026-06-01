# Outcome Map

Outcome Map 是 topic 导航仪表盘，不是聊天总结。每次继续学习、深读源码、切换阶段或更新 todo 前，先用它定位：本 topic 的最终产物是什么、当前在哪、正在补哪个缺口、哪些细节停止扩展。

## North Star

- Project：`{{TASK_NAME}}`
- Topic：`{{TOPIC_SLUG}}` - {{TOPIC_TITLE}}
- 最终要获得的能力：
- 最小可验证 demo：
- 现实问题中的迁移目标：

## Inherited Context

- Shared runbook：[`../../shared/runbook.md`](../../shared/runbook.md)
- Shared architecture：[`../../shared/architecture-map.md`](../../shared/architecture-map.md)
- Shared evidence：[`../../shared/evidence-registry.md`](../../shared/evidence-registry.md)

## Final Artifacts

| 产物 | 用途 | 状态 |
| --- | --- | --- |
| [`.daedalus/task-card.md`](task-card.md) | 学习目标与验收标准 | 草稿 |
| [`notes/03-socratic-coach/README.md`](../notes/03-socratic-coach/README.md) | 递进问题路线图 | 待填 |
| [`notes/04-debugger-guide/README.md`](../notes/04-debugger-guide/README.md) | 本地运行与调试证据 | 待填 |
| [`notes/05-arch-analyzer/README.md`](../notes/05-arch-analyzer/README.md) | 架构与核心边界 | 待填 |
| [`notes/06-code-reader/README.md`](../notes/06-code-reader/README.md) | 用户参与后的核心源码阅读证据 | 待填 |
| [`demo/design.md`](../demo/design.md) | mini demo 设计草案与定稿 | 待填 |
| [`demo/README.md`](../demo/README.md) | mini demo 实现、运行和验收说明 | 待填 |
| [`notes/09-biz-solver/README.md`](../notes/09-biz-solver/README.md) | 业务迁移方案 | 待填 |
| knowledge-base entry | 已验证知识归档 | 待填 |

## Current Position

Current Position 是学习地图，不是实现事实源。涉及实现阶段时，必须用当前代码、测试、运行输出和用户已验证观察校准后再判断完成度。

- 当前阶段：01-goal-aligner
- 当前目标：澄清 North Star、最终产物和验收标准。
- 当前障碍：任务目标和最小 demo 尚未由用户确认。
- 当前动作服务的产物：`.daedalus/task-card.md`、`.daedalus/outcome-map.md`
- 当前光标：暂无具体源码或 demo 实现边界；以 `todo.md` 的 `Current Cursor` 为临时恢复坐标。

## Contribution Back To Project

- 本 topic 完成后预计新增或更新哪些 shared evidence：
- 哪些 shared glossary / architecture-map 需要更新：
- 哪些 future topics 被发现：

## Artifact Dependency Graph

```text
task-card -> question-roadmap -> runbook -> architecture/code-reading
architecture/code-reading -> demo/design -> demo/README
demo evidence -> business-application -> knowledge-base entry
```

## Open Gaps

- [ ] 确认最终要获得的能力。
- [ ] 确认最小可验证 demo。
- [ ] 确认业务迁移目标。

## Why This Step Matters

当前步骤决定后续应该读哪些源码、停止哪些源码、最终 demo 如何验收。

## Stop Rules

- 不读与 North Star、demo、业务迁移或知识归档无关的源码细节。
- 不因为某个目录、函数或专题还没覆盖完就继续读。
- 不把 Agent-only 预读写成用户已经掌握的 notes。
