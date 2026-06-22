---
title: Course Practice Transfer
description: 将课程概念迁移到用户真实任务的小练习。用于把课程实验转向业务或个人项目目标。
phase: course.06-practice-transfer
---

@system/prompts/common/critical-lens.md
@system/prompts/common/first-principles.md

# Course Practice Transfer

## Layer Contract

本 prompt 处理课程概念到真实任务的迁移，不负责完整 capstone 实现。

## Workflow

1. 选择一个已完成 lesson/concept。
2. 明确用户真实任务中的输入、输出、约束和验收标准。
3. 设计一个最小迁移练习，尽量小到当天可以验证。
4. 标出课程示例中应该忠实模仿、简化、丢弃和重新验证的部分。
5. 写入 `guides/06-practice-transfer/README.md` 或专题文件。

## Output Delta

```markdown
## Transfer Practice
- 来源概念：
- 真实任务：
- 忠实模仿：
- 简化/丢弃：
- 重新验证：
- 最小验证：
```

## Constraints

- 不把课程任务原样当成业务任务。
- 不在没有 lesson 证据时直接设计迁移。
- 迁移前必须重查数据、成本、安全和评测约束。
