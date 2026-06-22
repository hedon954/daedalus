---
title: Course Concept Roadmap
description: 将章节标题转成概念问题、机制 checkpoint 和薄弱点地图。用于避免照抄课程代码却不知道在学什么。
phase: course.03-concept-roadmap
---

@system/prompts/common/coach-questioning.md
@system/prompts/common/first-principles.md
@system/prompts/common/question-roadmap.md

# Course Concept Roadmap

## Layer Contract

本 prompt 只建立课程概念路线。它不替用户回答所有问题，而是把问题组织成后续 lesson lab 和 mechanism deep dive 的观察目标。

## Workflow

1. 从当前章节提取核心名词、API、输入输出、隐藏机制和常见误解。
2. 将每个概念改写成可回答的问题。
3. 区分三类问题：先靠实验观察、需要文档解释、必要时读源码。
4. 同步 `shared/concept-map.md`。
5. 写入 `guides/03-concept-roadmap/README.md`。

## Output Delta

```markdown
## Concept Roadmap
- 当前章节：
- 核心概念：
- 最小观察问题：
- 机制深挖候选：
- 迁移题：
- 暂不回答的问题：
```

## Constraints

- 不把概念图写成术语表。
- 每个问题都要能连接到实验、解释或迁移任务。
- 用户没有复述前，不把 Agent 答案写进 notes。
