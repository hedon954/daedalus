---
title: Course Mechanism Deep Dive
description: 当课程 API 隐藏关键机制时，使用文档、源码或论文解释必要机制。用于 API 下面的 loss、generate、tokenizer、Trainer 等机制拆解。
phase: course.05-mechanism-deep-dive
---

@system/prompts/common/first-principles.md
@system/prompts/common/critical-lens.md
@system/prompts/common/diagram-guidelines.md

# Course Mechanism Deep Dive

## Layer Contract

本 prompt 只在 lesson lab 无法解释关键机制时使用。文档、源码或论文读取必须服务一个明确的课程机制问题。

## Workflow

1. 写清机制问题：哪个 API 隐藏了什么因果链。
2. 优先用官方文档、课程材料和可打印输出解释。
3. 仍不清楚时，只读最小源码路径。
4. 用图或伪代码解释输入、状态变化、输出和 failure mode。
5. 写入 `guides/05-mechanism-deep-dive/<mechanism>.md`。

## Output Delta

```markdown
## Mechanism
- 问题：
- 触发 API：
- 最小因果链：
- 可观察证据：
- 源码/文档锚点：
- 不必继续深入的边界：
```

## Constraints

- 不按目录读源码。
- 不为了“彻底”而扩大阅读范围。
- 机制解释必须回到 lesson lab 或 practice transfer。
