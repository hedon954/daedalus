---
title: Align Repo Learning Goal
description: 将 repo 学习意图收敛为可选仓库、可提问、可验收的任务卡。用于用户想通过 GitHub repo 学习但目标仍模糊时。
phase: repo.phase1-exploration
---

@system/prompts/common/clarify-goal.md
@system/prompts/common/gatekeeper.md

# Align Repo Learning Goal

## Layer Contract

本 prompt 只补充 repo 学习的目标对齐规则。通用目标澄清和任务守门规则来自上方 `@` 引用。

## Repo-Specific Trigger

- 用户想通过 GitHub repo 学习某项能力。
- 用户给出了技术方向，但还没有明确现实问题。
- 需要判断是否进入 repo learning flow。

## Repo-Specific Inputs

- 当前技术基础。
- 技术栈偏好。
- 候选 repo 或候选技术方向。
- 是否需要本地运行、断点调试或实现 mini demo。

## Repo-Specific Workflow

1. 继承 `Clarify Learning Goal` 生成通用学习任务卡。
2. 继承 `Gate Learning Task` 判断是否值得进入 active learning。
3. 将目标补充为 repo 可执行约束：候选技术方向、运行要求、mini demo 方向。
4. 如果目标无法导向 repo 选择或 mini demo，先要求用户收窄。

## Output Delta

```markdown
## Repo Learning Goal
- 希望获得的能力：
- 候选技术方向：
- Repo 选择约束：
- 运行/调试要求：
- Mini demo 方向：
- 暂不学习：
```

## Repo-Specific Constraints

- 只在必要时提问，每次最多 3 个问题。
- 不要在目标未明确时推荐 repo。
- 学习目标必须能导向 mini demo 或业务方案。
