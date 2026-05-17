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

本阶段不是从零设计 demo，而是定稿 `06-code-reader` 期间形成的 `demo/design.md` 草案。代码阅读已经负责用源码证据填补关键不变量、数据结构、状态机和验收用例；本阶段负责裁剪、统一、验收和进入实现。

## Repo-Specific Trigger

- repo 的核心链路和架构已经基本清楚。
- 用户准备从阅读进入实现。
- 需要把学习目标转成可验收 demo。
- `.daedalus/outcome-map.md` 中指向 demo 的 open gaps 已经收敛到可以做设计决策。
- `demo/design.md` 已有草案，且关键字段不再被核心源码证据阻塞。

## Repo Demo Principles

- 只保留最核心能力和最关键 trade-off。
- 明确不做什么，防止 demo 膨胀。
- 保留关键不变量，例如模块边界、状态流、插件点、调度方式或错误模型。
- demo 必须能运行、能测试、能解释。

## Repo-Specific Workflow

1. 读取 `.daedalus/outcome-map.md` 和 `demo/design.md` 草案，确认 North Star、当前缺口和 stop rules。
2. 选择一个核心能力，不超过一个主链路。
3. 明确从原 repo 保留的架构决策，并把每个决策绑定到已验证源码证据或用户确认的学习目标。
4. 定义模块、数据结构、主链路、状态机和验收用例。
5. 写出不做清单，防止范围膨胀。
6. 更新 `.daedalus/outcome-map.md`：把 demo 设计相关 gap 标为已解决或仍待实现验证。

## Output Delta

```markdown
## Mini Demo Design
- 要复现的核心能力：
- 来自 repo 的关键架构决策：
- 证据来源：
- 模块划分：
- 核心数据结构：
- 主链路 / 状态机：
- 明确不做：
- 验收用例：
- 进入实现前仍需确认：
```

## Repo-Specific Constraints

- 如果 demo 仍然太大，继续砍到 1-2 天可完成的范围。
- 不要复刻完整 repo。
- 验收用例必须在编码前确定。
- 不要把 `06-code-reader` 未解决的源码疑问带入实现。如果疑问阻塞核心不变量、数据结构或状态机，退回代码阅读；如果不阻塞，把它写入 non-goals 或 future reading。
