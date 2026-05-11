---
title: Read Core Repo Code
description: 沿核心链路深读关键代码，提取不变量、设计选择和可迁移模式。用于已有入口、架构问题或 demo 设计需要代码证据时。
phase: repo.phase2-learning
---

@system/prompts/common/first-principles.md
@system/prompts/common/summarize.md
@system/prompts/common/coach-questioning.md
@system/prompts/common/diagram-guidelines.md

# Read Core Repo Code

## Layer Contract

本 prompt 只定义 repo 核心代码阅读方法。核心原则：不要按函数清单读源码；要从工业级项目的真实失败模式出发，追踪源码如何处理生产约束、保护不变量、付出代价，并抽取可迁移模式。

## Repo-Specific Trigger

- 已有核心入口或架构问题。
- 需要逐行解释关键实现。
- demo 设计需要明确不变量和主链路。

## Repo Reading Order

1. 生产问题：真实环境中这个能力会遇到什么失败模式。
2. naive 方案：最直接实现会在哪里失败，造成什么事故或维护成本。
3. 核心抽象：接口、trait、class、数据结构保护什么不变量。
4. 主链路：关键函数如何串起来，状态在哪里变化，失败如何被兜底。
5. 边界条件：错误处理、并发、取消、重试、缓存、IO、持久化、外部依赖、权限、安全和恢复。
6. 设计代价：为什么这么切模块，替代方案会有什么代价，哪些部分不值得照抄。

## Repo-Specific Workflow

1. 每次只读一个可闭环的生产问题，不按文件或函数覆盖率推进。
2. 先让用户说出 naive 实现和可能失败点，再读源码验证 repo 的真实应对。
3. 用“这段代码防止了什么生产事故”和“这段代码保护了什么不变量”解释关键实现。
4. 读完后产出可迁移模式、不应照抄的部分，并更新 demo 不变量清单。
5. 每轮代码阅读都要记录“生产问题、用户猜测、Agent 校准、源码证据、不变量、代价、验证状态”，避免 Agent 直接替用户完成理解。

## Output Delta

```markdown
## Code Reading Note
- 阅读问题：
- 用户猜测：
- Agent 校准：
- 生产问题：
- naive 方案会怎样失败：
- 文件：
- 入口函数：
- 核心不变量：
- 调用链：
- 失败处理：
- 工业约束：
- 难点：
- 源码证据：
- 设计代价：
- 验证状态：
- 可迁移模式：
- 不应照抄的部分：
```

## Repo-Specific Constraints

- 不要为了覆盖率扫读所有文件。
- 只在关键片段逐行阅读。
- 不要只解释“代码做了什么”；必须解释“为什么生产环境需要它”和“如果没有它会怎样失败”。
- 每个源码专题都必须落到不变量、失败兜底、设计代价和可迁移模式。
- 每个阅读片段都要回到架构问题或 demo 设计。
- 不要替用户直接写完整代码阅读笔记；先让用户描述理解，再由 Agent 校正、补充证据和按需画图。
- 用户回答中的误解也要保留并标注为“已校正”或“待验证”，不要静默改写成正确答案。
