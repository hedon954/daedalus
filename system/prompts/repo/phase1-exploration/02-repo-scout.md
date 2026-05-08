---
title: Select Study Repo
description: 根据学习目标筛选并比较值得深读的代码仓库，收敛到首选仓库。用于 repo 未确定或多个候选需要取舍时。
phase: repo.phase1-exploration
---

@system/prompts/common/first-principles.md
@system/prompts/common/gatekeeper.md

# Select Study Repo

## Layer Contract

本 prompt 只定义代码仓库的筛选标准。学习任务是否值得进入 active learning 由上方 `Gate Learning Task` 引用处理；现实约束和 trade-off 框架由上方 `Apply First Principles` 引用提供。

## Repo-Specific Trigger

- 学习目标已经明确，但 repo 未确定。
- 用户给出多个候选 repo，需要比较。
- 需要判断某个 repo 是否值得深读。

## Repo Evaluation

- 目标匹配：是否直接覆盖用户现实问题中的核心能力。
- 代码质量：模块边界、测试、文档、维护活跃度是否足够。
- 学习密度：是否包含清晰的架构决策、trade-off 和生产约束。
- 可运行性：本地启动、断点调试、最小链路复现是否可行。
- 复杂度：是否足够真实，但不会让用户长期迷失。

## Repo-Specific Workflow

1. 根据学习目标列出最多 3 个候选 repo。
2. 对每个 repo 评估匹配度、学习密度、运行风险和 demo 可能性。
3. 给出首选 repo，并说明为什么它最值得优先深入。
4. 如果候选 repo 都不合适，建议调整目标或重新搜索。

## Output Delta

```markdown
## Candidate Repos

### 1. repo/name
- 为什么匹配：
- 核心能力：
- 适合阅读的入口：
- 可能的 mini demo：
- 风险：
```

## Repo-Specific Constraints

- 最多推荐 3 个候选 repo。
- 不要只按 star 数排序。
- 必须说明首选 repo 和放弃其他候选的理由。
