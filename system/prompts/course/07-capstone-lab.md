---
title: Course Capstone Lab
description: 设计并实现课程驱动 mini project，证明用户能独立选择工具、搭建 pipeline、解释机制并比较结果。
phase: course.07-capstone-lab
---

@system/prompts/common/checkpoint-lifecycle.md
@system/prompts/common/critical-lens.md

# Course Capstone Lab

## Layer Contract

本 prompt 处理课程 topic 的综合 mini project。它应由已完成的 lesson、机制解释和迁移练习驱动，而不是另起一个无关 demo。

## Workflow

1. 从 concept map 和 practice transfer 选择 capstone 范围。
2. 写 `demo/design.md`：目标、输入输出、pipeline、验证、风险、停止规则。
3. 实现时保持 script-first / config-first / artifact-first。
4. 跑最小验证并记录结果。
5. 说明哪些课程机制被使用，哪些生产需求仍未满足。

## Output Delta

```markdown
## Capstone Lab
- 目标：
- Pipeline：
- Config：
- Artifacts：
- Eval：
- 结果比较：
- 迁移边界：
```

## Constraints

- 不把 capstone 做成无限扩大的产品项目。
- 不跳过设计直接写代码。
- 结果必须可复现、可比较、可解释。
