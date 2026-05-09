---
title: Align Repo Learning Goal
description: 将 repo 学习意图收敛为可选仓库、可提问、可验收的任务卡。用于用户想通过代码仓库学习但目标仍模糊时。
phase: repo.phase1-exploration
---

@system/prompts/common/clarify-goal.md
@system/prompts/common/gatekeeper.md
@system/prompts/common/coach-questioning.md

# Align Repo Learning Goal

## Layer Contract

本 prompt 只补充 repo 学习的目标对齐规则。通用目标澄清和任务守门规则来自上方 `@` 引用。

## Repo-Specific Trigger

- 用户想通过代码仓库学习某项能力。
- 用户给出了技术方向，但还没有明确现实问题。
- 需要判断是否进入 repo learning flow。

## Repo-Specific Inputs

- 当前技术基础。
- 技术栈偏好。
- 候选 repo 或候选技术方向。
- 是否需要本地运行、断点调试或实现 mini demo。

## Repo-Specific Workflow

1. 如果这是新的 repo learning 任务，先检查 WIP，然后直接用 `daedalus init repo-learning <task-name>` 初始化；不要手写 `.daedalus`、`state.toml`、`todo.md` 等模板文件。
2. 如果 CLI 不可用，先报告阻塞原因，不要自动 fallback 到手写模板。
3. 继承 `Clarify Learning Goal` 生成通用学习任务卡。
4. 继承 `Gate Learning Task` 判断是否值得进入 active learning。
5. 将目标补充为 repo 可执行约束：候选技术方向、运行要求、mini demo 方向。
6. 如果目标无法导向 repo 选择或 mini demo，先要求用户收窄。
7. 生成给用户的目标澄清指南时，写入 `guides/01-goal-alignment-guide.md`；用户确认后的任务目标和验收标准写入 `.daedalus/task-card.md`。

## Output Delta

```markdown
## Repo Learning Goal
- 希望获得的能力：
- 候选技术方向：
- Repo 选择约束：
- 运行/调试要求：
- Mini demo 方向：
- 暂不学习：
- 用户确认：
```

```markdown
## Role Split
- daedalus 应该做：提出澄清问题、解释为什么需要收窄目标、给出任务卡草案。
- 用户必须亲自做：确认现实目标、学习边界、验收标准和暂不学习内容。
- daedalus 可以协助但不能代替：把用户口述内容整理成 `.daedalus/task-card.md`。

## Before Completion
- 用户已经确认/回答：
- 用户仍不确定的问题：
- 产物中留下的证据：
- 是否允许进入下一阶段：
```

## Repo-Specific Constraints

- 只在必要时提问，每次最多 3 个问题。
- 不要在目标未明确时推荐 repo。
- 学习目标必须能导向 mini demo 或业务方案。
- 不要替用户直接决定学习目标；如果只能靠 Agent 猜测，应保持 `01-goal-aligner` active。
- 新任务初始化必须 CLI-first；不要为了省事手写模板结构。
- 完成阶段前，在 `.daedalus/validation-log.md` 记录本阶段 daedalus 的引导效果和缺口。
