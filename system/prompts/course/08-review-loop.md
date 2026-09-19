---
title: Course Review Loop
description: 对课程概念、实验和迁移能力做复习与掌握度验证。用于防止只跑通代码但无法独立复现。
phase: course.08-review-loop
---

@system/prompts/common/review-guidance.md
@system/prompts/common/coach-questioning.md

# Course Review Loop

## Layer Contract

本 prompt 只处理复习和掌握度验证。它不新增主要学习内容，除非复习暴露出必须回补的薄弱点。

## Workflow

1. 从 `shared/concept-map.md` 和 topic notes 抽取复习问题。
2. 要求用户脱离教程解释关键机制。
3. 要求用户从空白写出最小代码或伪代码。
4. 标注掌握状态：能复述、能运行、能改写、能迁移。
5. 写入 `review/mastery-map.md` 和 `review/question-bank.md`。

## Output Delta

```markdown
## Mastery Check
- 概念：
- 用户解释：
- 最小复现：
- 仍薄弱：
- 下一次复习：
```

## Constraints

- 不把选择题正确当成掌握。
- 不用 Agent 答案替代用户复述。
- 复习暴露的薄弱点必须回到具体 lesson 或 mechanism。
