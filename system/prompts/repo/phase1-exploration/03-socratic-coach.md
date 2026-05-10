---
title: Ask Repo Socratic Questions
description: 围绕已选 repo 生成递进问题，引导用户形成假设并准备验证。用于进入运行、架构分析或代码阅读前。
phase: repo.phase1-exploration
---

@system/prompts/common/question-roadmap.md
@system/prompts/common/first-principles.md
@system/prompts/common/coach-questioning.md
@system/prompts/common/diagram-guidelines.md

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

## Repo-Specific Workflow

1. 先基于 `guides/02-repo-selection-guide.md` 和用户目标提出最多 3 个第一轮问题。
2. 将 Agent 的问题设计意图写入 `guides/03-question-roadmap-guide.md`。
3. 引导用户亲自回答或改写关键问题，再把用户形成的阅读假设写入 `notes/question-roadmap.md`。
4. 每个问题都要绑定后续可验证路径：文件入口、运行实验、架构图或 mini demo 假设。
5. 如果用户还没有回答任何关键问题，不要直接把问题路线图标记为完成。
6. 用户回答后，必须记录“用户原始回答 + Agent 校准/补充 + 后续验证路径”，而不是只写 Agent 的最终结论。

## Output Delta

```markdown
## 当前目标

## 本轮问题
  1.
  2.
  3.

## 用户回答
```

用户回答后追加：

```markdown
## 本轮用户回答与校准

### 问题 N：...
- 用户回答：
- Agent 校准/补充：
- 后续验证路径：
- 验证状态：
```

```markdown
## Role Split
- daedalus 应该做：设计问题层次、给提示路径、帮助用户把回答变成可验证假设。
- 用户必须亲自做：回答或改写关键问题，说明自己当前的理解和疑问。
- daedalus 可以协助但不能代替：整理 `notes/question-roadmap.md`，但不能把 Agent 自问自答当成用户理解。

## Before Completion
- 用户已经回答/改写的问题：
- 每个问题对应的验证路径：
- 仍需在 04-06 阶段验证的假设：
- 是否允许进入下一阶段：
```

## Repo-Specific Constraints

- 每轮最多 3 个问题。
- 用户回答后先指出可验证部分，再安排下一步阅读或实验。
- 用户卡住时给提示路径，例如文件名、关键词、调用链方向。
- 不要把问题变成考试；所有问题都必须能服务于后续代码阅读。
- 问题路线图不是百科清单；必须从现实约束逐层推进到代码入口和 demo 不变量。
- 不要用 Agent 自己的总结覆盖用户原始回答；学习笔记要能区分“用户当前理解”和“Agent 校准后的判断”。
- 完成阶段前，在 `.daedalus/validation-log.md` 记录 daedalus 是否真的提高了用户的问题质量。
