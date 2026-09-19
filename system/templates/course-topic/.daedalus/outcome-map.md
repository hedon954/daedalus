# Outcome Map

Outcome Map 是 topic 导航仪表盘，不是聊天总结。每次继续学习、深读源码、切换阶段或更新 todo 前，先用它定位：本 topic 的最终产物是什么、当前在哪、正在补哪个缺口、哪些细节停止扩展。

## North Star

- Project：`{{TASK_NAME}}`
- Topic：`{{TOPIC_SLUG}}` - {{TOPIC_TITLE}}
- 最终要获得的能力：
- 最小可验证 lesson lab：
- 现实问题中的迁移目标：

## Inherited Context

- Syllabus map：[`../../shared/syllabus-map.md`](../../shared/syllabus-map.md)
- Course progress：[`../../shared/course-progress.md`](../../shared/course-progress.md)
- Concept map：[`../../shared/concept-map.md`](../../shared/concept-map.md)
- Shared evidence：[`../../shared/evidence-registry.md`](../../shared/evidence-registry.md)

## Final Artifacts

| 产物 | 用途 | 状态 |
| --- | --- | --- |
| [`.daedalus/task-card.md`](task-card.md) | 学习目标与验收标准 | 草稿 |
| [`guides/02-syllabus-mapper/README.md`](../guides/02-syllabus-mapper/README.md) | syllabus 取舍与章节路线 | 待填 |
| [`guides/03-concept-roadmap/README.md`](../guides/03-concept-roadmap/README.md) | 概念问题路线图 | 待填 |
| [`guides/04-lesson-lab/README.md`](../guides/04-lesson-lab/README.md) | lesson 可观察实验指南 | 待填 |
| [`notes/04-lesson-lab/README.md`](../notes/04-lesson-lab/README.md) | 用户运行、观察和解释证据 | 待填 |
| [`guides/05-mechanism-deep-dive/README.md`](../guides/05-mechanism-deep-dive/README.md) | 必要机制深挖 | 待填 |
| [`guides/06-practice-transfer/README.md`](../guides/06-practice-transfer/README.md) | 迁移到真实任务的练习 | 待填 |
| [`demo/design.md`](../demo/design.md) | mini demo 设计草案与定稿 | 待填 |
| [`review/mastery-map.md`](../review/mastery-map.md) | 掌握度验证 | 待填 |
| knowledge-base entry | 已验证知识归档 | 待填 |

## Current Position

Current Position 是学习地图，不是实现事实源。涉及实现阶段时，必须用当前代码、测试、运行输出和用户已验证观察校准后再判断完成度。

- 当前阶段：01-need-aligner
- 当前目标：澄清 North Star、最终产物和验收标准。
- 当前障碍：课程学习诉求、章节取舍和最小 lesson lab 尚未由用户确认。
- 当前动作服务的产物：`.daedalus/task-card.md`、`.daedalus/outcome-map.md`
- 当前光标：暂无具体 lesson、mechanism 或 transfer checkpoint；以 `todo.md` 的 `Current Cursor` 为临时恢复坐标。

## Contribution Back To Project

- 本 topic 完成后预计新增或更新哪些 shared evidence：
- 哪些 shared glossary / architecture-map 需要更新：
- 哪些 future topics 被发现：

## Artifact Dependency Graph

```text
task-card -> syllabus-map -> concept-roadmap -> lesson-lab
lesson-lab -> mechanism-deep-dive -> practice-transfer
practice-transfer -> capstone/review -> knowledge-base entry
```

## Open Gaps

- [ ] 确认最终要获得的能力。
- [ ] 确认最小可验证 lesson lab。
- [ ] 确认业务迁移目标。

## Why This Step Matters

当前步骤决定后续应该学哪些章节、停止哪些材料、最终 lesson lab 和迁移任务如何验收。

## Critical Lens

Critical Lens 用来防止把学习素材当成权威。它不是反对模仿，而是让 demo 的模仿变成有意识、有证据、有边界的学习动作。

- 当前素材中可能被过度神化的设计：
- 当前 lesson lab 需要忠实模仿的核心机制：
- 当前 demo 不应无意识照抄的设计：
- 当前还没有验证的素材假设：
- 当前可以尝试简化、改进或丢弃的部分：
- 当前迁移到业务场景前必须重新验证的约束：

## Stop Rules

- 不学与 North Star、lesson lab、迁移任务或知识归档无关的课程细节。
- 不因为课程还有章节没覆盖完就继续学。
- 不把 Agent-only 预读写成用户已经掌握的 notes。
