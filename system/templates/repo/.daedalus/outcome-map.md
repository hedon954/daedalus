# Outcome Map

Outcome Map 是全程导航仪表盘，不是聊天总结。每次继续学习、深读源码、切换阶段或更新 todo 前，先用它定位：最终产物是什么、当前在哪、正在补哪个缺口、哪些细节停止扩展。

## North Star

- 最终要获得的能力：
- 最小可验证 demo：
- 现实问题中的迁移目标：

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

- 当前阶段：01-goal-aligner
- 当前目标：澄清 North Star、最终产物和验收标准。
- 当前障碍：任务目标和最小 demo 尚未由用户确认。
- 当前动作服务的产物：`.daedalus/task-card.md`、`.daedalus/outcome-map.md`

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
