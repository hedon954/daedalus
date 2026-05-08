---
title: Design Repo Mini Demo
description: 将 repo 核心能力压缩成可实现、可验证的 mini demo 架构。用于核心链路和 trade-off 已明确、准备进入编码前。
phase: repo.phase3-practice
---

@system/prompts/common/clarify-goal.md
@system/prompts/common/first-principles.md

# Design Repo Mini Demo

## Layer Contract

本 prompt 只定义 repo mini demo 的设计约束。可验收目标来自 `Clarify Learning Goal`，核心 trade-off 解释来自 `Apply First Principles`。

## Repo-Specific Trigger

- repo 的核心链路和架构已经基本清楚。
- 用户准备从阅读进入实现。
- 需要把学习目标转成可验收 demo。

## Repo Demo Principles

- 只保留最核心能力和最关键 trade-off。
- 明确不做什么，防止 demo 膨胀。
- 保留关键不变量，例如模块边界、状态流、插件点、调度方式或错误模型。
- demo 必须能运行、能测试、能解释。

## Repo-Specific Workflow

1. 选择一个核心能力，不超过一个主链路。
2. 明确从原 repo 保留的架构决策。
3. 定义模块、数据结构、主链路和验收用例。
4. 写出不做清单，防止范围膨胀。

## Output Delta

```markdown
## Mini Demo Design
- 要复现的核心能力：
- 来自 repo 的关键架构决策：
- 模块划分：
- 核心数据结构：
- 主链路：
- 明确不做：
- 验收用例：
```

## Repo-Specific Constraints

- 如果 demo 仍然太大，继续砍到 1-2 天可完成的范围。
- 不要复刻完整 repo。
- 验收用例必须在编码前确定。
