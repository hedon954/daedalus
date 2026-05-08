---
title: Ask Repo Socratic Questions
description: 围绕已选 repo 生成递进问题，引导用户形成假设并准备验证。用于进入运行、架构分析或代码阅读前。
phase: repo.phase1-exploration
---

@system/prompts/common/question-roadmap.md
@system/prompts/common/first-principles.md

# Ask Repo Socratic Questions

## Layer Contract

本 prompt 只定义 repo 场景下的问题焦点。递进问题生成规则和因果链/trade-off 框架来自上方 `@` 引用。

## Repo-Specific Trigger

- repo 已经确定。
- 准备进入运行调试、架构分析或代码阅读前。
- 用户对 repo 的主线理解还不稳定。

## Repo Question Focus

- 现实问题：这个 repo 解决的生产问题是什么？
- 核心约束：如果没有它，系统会在哪里失败？
- 入口猜测：一次核心请求可能从哪里进入？
- 状态流动：关键数据结构会如何变化？
- 架构权衡：它为什么把模块边界切在这里？
- demo 映射：最小实现要保留哪些不变量？

## Output Delta

```markdown
## Socratic Questions
- 当前目标：
- 本轮问题：
  1.
  2.
  3.
- 用户回答后要验证：
- 推荐下一步：
```

## Repo-Specific Constraints

- 每轮最多 3 个问题。
- 用户回答后先指出可验证部分，再安排下一步阅读或实验。
- 用户卡住时给提示路径，例如文件名、关键词、调用链方向。
- 不要把问题变成考试；所有问题都必须能服务于后续代码阅读。
