# Todo Path Board

Todo 是动态路径看板。学习证据变化、阶段完成、学习路径需要收窄或扩展时，都要同步更新它和 [`outcome-map.md`](outcome-map.md)。

> Agent 负责拆解、指导、排障和验收；用户负责关键实践、观察和手写笔记。不要把 Agent 自动完成的事项伪装成用户已经掌握。

## North Star

- Project：`{{TASK_NAME}}`
- Topic：`{{TOPIC_SLUG}}` - {{TOPIC_TITLE}}
- 最终产物：
- 最小 demo：
- 业务迁移目标：

## Current Path

当前 active topic 正在从 `01-goal-aligner` 走向 `.daedalus/task-card.md` 和 `.daedalus/outcome-map.md`。

## Now

- 当前问题：澄清当前学习目标、学习边界和验收标准。
- 为什么现在做它：没有 North Star，后续 repo 选择、问题路线图、源码阅读和 demo 都会失去取舍标准。
- 完成后解锁：进入候选 repo 选择，并能判断哪些材料不值得读。

## Current Cursor

Current Cursor 是恢复定位器，不是完成证明。恢复或判断阶段状态时，必须用当前代码、测试、运行输出或用户已验证观察重新校准。

- Code frontier：暂无，尚未进入具体源码或 demo 实现边界。
- Already wired：暂无。
- Current open decision：暂无。
- Do not suggest：不要跳过 goal alignment 直接进入 repo 阅读。

## Critical Checkpoint

Critical Checkpoint 只记录会影响后续判断的关键取舍，不写成聊天日志。

- Source constraint：当前素材方案成立的现实约束是什么？
- Faithful imitation：demo 中哪些机制需要先忠实模仿以获得实现手感？
- Simplified / improved / discarded：本轮哪些部分应该简化、改进或丢弃？
- Transfer risk：迁移到业务场景前必须重新验证什么？

## Gaps Blocking Next Stage

- [ ] 学习目标：阻塞 `.daedalus/task-card.md` 的当前目标和验收标准。
- [ ] 最小 demo：阻塞 `.daedalus/outcome-map.md` 的 North Star。
- [ ] 业务迁移目标：阻塞后续 `notes/business-application.md` 的评价标准。

## Stage Exit Criteria

- [ ] 可以解释本任务为什么值得学习一个真实 repo。
- [ ] 可以说清最终产物和最小 demo。
- [ ] 可以用验收标准判断是否进入 `02-repo-scout`。

## Done

## Canceled
