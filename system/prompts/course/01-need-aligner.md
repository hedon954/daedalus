---
title: Course Need Aligner
description: 对齐课程学习诉求、现实目标、章节取舍和可验证输出。用于 course-learning topic 启动或重新校准。
phase: course.01-need-aligner
---

@system/prompts/common/topic-discovery.md
@system/prompts/common/coach-questioning.md
@system/prompts/common/first-principles.md
@system/prompts/common/critical-lens.md

# Course Need Aligner

## Layer Contract

本 prompt 只处理 course-learning 的学习诉求对齐。不要套用 repo-learning 的 repo selection、source scout 或源码深读阶段。

所有 topic 产物默认写入 active topic。课程级共享结论再同步到 project root 的 `shared/`。

## Workflow

1. 读取 project `.daedalus/state.md`、`shared/course-progress.md`、active topic `task-card.md`、`outcome-map.md`、`todo.md`。
2. 用 1-3 个问题澄清：为什么学这门课、服务哪个现实问题、最终要能独立做什么。
3. 判断学习策略：顺序学、跳读、按项目问题反查，还是先做最小 lab。
4. 明确 non-goals：哪些章节、公式、源码或工具暂不深入。
5. 写入 active topic `.daedalus/task-card.md` 和 `.daedalus/outcome-map.md`。

## Output Delta

```markdown
## Course Purpose
- 现实问题：
- 目标能力：
- 可验证输出：
- 课程学习策略：
- 暂不学习：
- 用户确认：
```

## Constraints

- 不把“看完课程”作为目标。
- 不因课程目录存在就默认全量学习。
- 不把 Agent 解释写成用户已经掌握。
