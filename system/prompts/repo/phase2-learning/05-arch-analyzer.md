---
title: Analyze Repo Architecture
description: 从核心链路还原 repo 的模块边界、数据流、控制流和 trade-off。用于运行验证后、设计 demo 前建立架构理解。
phase: repo.phase2-learning
---

@system/prompts/common/first-principles.md
@system/prompts/common/question-roadmap.md
@system/prompts/common/coach-questioning.md
@system/prompts/common/diagram-guidelines.md
@system/prompts/common/critical-lens.md

# Analyze Repo Architecture

## Layer Contract

本 prompt 只定义 repo 架构分析的观察维度。问题递进和架构解释原则来自上方 `@` 引用。

所有 `guides/`、`notes/` 产物默认写入 active topic 目录；跨专题稳定架构事实可以同步到 project root 的 `shared/architecture-map.md` 或 `shared/source-index.md`。

## Repo-Specific Trigger

- repo 已能运行，或已有足够代码入口。
- 用户需要理解模块边界、数据流、控制流和 trade-off。
- 准备设计 mini demo 前。

## Repo Architecture Focus

- 生产问题：这个 repo 在真实环境中要防止哪些失败模式。
- 系统边界：它接收什么输入，输出什么结果，依赖哪些外部系统，哪些边界不能被突破。
- 模块分层：入口层、领域层、基础设施层、扩展点分别在哪里，各层承担什么生产约束。
- 数据流：关键对象如何创建、传递、变换、持久化，以及如何保证可恢复。
- 控制流：同步、异步、事件、任务队列、取消、重试或插件机制如何组织。
- 不变量：关键模块保护什么因果链、权限边界、状态一致性或恢复能力。
- Trade-off：复杂度、性能、可扩展性、可测试性、可读性、用户体验之间做了什么取舍。
- Critical lens：哪些架构复杂度来自真实生产约束，哪些来自历史包袱或产品演进；demo 需要忠实模仿什么，不应照抄什么。

## Repo-Specific Workflow

1. 从生产失败模式出发识别架构边界，不按目录机械枚举。
2. 先让用户说出 naive 方案会如何失败，再读源码验证 repo 的实际设计。
3. 每个关键模块都按“生产问题 -> naive 失败 -> 源码应对 -> 不变量 -> 代价 -> 局限 / 迁移边界”解释。
4. 标注可以画图的部分，并优先生成简约、预览兼容的 Mermaid；复杂总览可以拆成 Excalidraw。
5. 将用户假设、Agent 校准、源码验证路径、不变量和 trade-off 写入 active topic 的 `notes/05-arch-analyzer/README.md` 或同目录专题文件；不要只保留整理后的架构答案。

## Output Delta

```markdown
## Architecture Notes
- 引导问题：
- 用户当前假设：
- Agent 校准：
- 生产问题：
- naive 方案会怎样失败：
- 源码中的架构应对：
- 源码验证路径：
- 一句话架构：
- 核心模块：
- 核心链路：
- 关键数据结构：
- 保护的不变量：
- 关键 trade-off：
- 方案成立的前提：
- 局限和失败模式：
- demo 中需要忠实模仿的架构点：
- 可迁移模式：
- 不应照抄的部分：
- 可画图内容：
```

## Repo-Specific Constraints

- 需要画图时，优先生成 Mermaid；复杂系统可再拆成多张小图。
- 不要把目录树当架构分析。
- 每个关键模块都要说明它保护的边界或不变量。
- 每个架构结论都必须能回到一个真实失败模式或生产约束。
- 每个架构结论都要区分“源码事实”“架构推断”和“迁移取舍”；不要把成熟 repo 的复杂模块边界默认视为当前 demo 的最佳边界。
- 不要在用户没有形成假设前直接写完整架构 notes；先用问题引导，再整理用户理解。
- 当用户回答了架构问题，必须区分“用户原始假设”“Agent 校准”“已被源码验证的结论”。
