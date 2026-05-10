---
title: Analyze Repo Architecture
description: 从核心链路还原 repo 的模块边界、数据流、控制流和 trade-off。用于运行验证后、设计 demo 前建立架构理解。
phase: repo.phase2-learning
---

@system/prompts/common/first-principles.md
@system/prompts/common/question-roadmap.md
@system/prompts/common/coach-questioning.md
@system/prompts/common/diagram-guidelines.md

# Analyze Repo Architecture

## Layer Contract

本 prompt 只定义 repo 架构分析的观察维度。问题递进和架构解释原则来自上方 `@` 引用。

## Repo-Specific Trigger

- repo 已能运行，或已有足够代码入口。
- 用户需要理解模块边界、数据流、控制流和 trade-off。
- 准备设计 mini demo 前。

## Repo Architecture Focus

- 系统边界：它接收什么输入，输出什么结果，依赖哪些外部系统。
- 模块分层：入口层、领域层、基础设施层、扩展点分别在哪里。
- 数据流：关键对象如何创建、传递、变换、持久化。
- 控制流：同步、异步、事件、任务队列或插件机制如何组织。
- Trade-off：复杂度、性能、可扩展性、可测试性之间做了什么取舍。

## Repo-Specific Workflow

1. 从核心链路出发识别模块，不按目录机械枚举。
2. 先让用户说出当前架构假设，再抽取关键数据结构和依赖方向。
3. 解释每个边界背后的现实约束。
4. 标注可以画图的部分，并优先生成简约、Typora 兼容的 Mermaid。
5. 将用户的架构假设、Agent 校准和源码验证路径写入 notes；不要只保留整理后的架构答案。

## Output Delta

```markdown
## Architecture Notes
- 引导问题：
- 用户当前假设：
- Agent 校准：
- 源码验证路径：
- 一句话架构：
- 核心模块：
- 核心链路：
- 关键数据结构：
- 关键 trade-off：
- 可画图内容：
```

## Repo-Specific Constraints

- 需要画图时，优先生成 mermaid；复杂系统可再拆成多张小图。
- 不要把目录树当架构分析。
- 每个关键模块都要说明它保护的边界或不变量。
- 不要在用户没有形成假设前直接写完整架构 notes；先用问题引导，再整理用户理解。
- 当用户回答了架构问题，必须区分“用户原始假设”“Agent 校准”“已被源码验证的结论”。
