---
title: Read Core Repo Code
description: 沿核心链路深读关键代码，提取不变量、设计选择和可迁移模式。用于已有入口、架构问题或 demo 设计需要代码证据时。
phase: repo.phase2-learning
---

@system/prompts/common/first-principles.md
@system/prompts/common/summarize.md

# Read Core Repo Code

## Layer Contract

本 prompt 只定义 repo 核心代码阅读方法。可迁移结论表达和解释方式来自上方 `@` 引用。

## Repo-Specific Trigger

- 已有核心入口或架构问题。
- 需要逐行解释关键实现。
- demo 设计需要明确不变量和主链路。

## Repo Reading Order

1. 入口：请求、命令、任务或测试从哪里进入。
2. 核心抽象：接口、trait、class、数据结构承担什么不变量。
3. 主链路：关键函数如何串起来，状态在哪里变化。
4. 边界：错误处理、并发、缓存、IO、持久化、外部依赖。
5. 设计选择：为什么这么切模块，替代方案会有什么代价。

## Repo-Specific Workflow

1. 每次只读一个可闭环片段。
2. 先问用户对代码意图的猜测，再解释细节。
3. 用“这行代码保护了什么不变量”解释关键实现。
4. 读完后产出可迁移结论，并更新 demo 设计。

## Output Delta

```markdown
## Code Reading Note
- 文件：
- 入口函数：
- 核心不变量：
- 调用链：
- 难点：
- 可迁移结论：
```

## Repo-Specific Constraints

- 不要为了覆盖率扫读所有文件。
- 只在关键片段逐行阅读。
- 每个阅读片段都要回到架构问题或 demo 设计。
