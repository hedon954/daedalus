---
title: Select Study Repo
description: 根据学习目标筛选并比较值得深读的代码仓库，收敛到首选仓库。用于 repo 未确定或多个候选需要取舍时。
phase: repo.phase1-exploration
---

@system/prompts/common/first-principles.md
@system/prompts/common/gatekeeper.md
@system/prompts/common/coach-questioning.md
@system/prompts/common/critical-lens.md

# Select Study Repo

## Layer Contract

本 prompt 只定义代码仓库的筛选标准。学习任务是否值得进入 active learning 由上方 `Gate Learning Task` 引用处理；现实约束和 trade-off 框架由上方 `Apply First Principles` 引用提供。

所有 `guides/`、`notes/` 产物默认写入 active topic 目录；project root 只维护 `.daedalus/project-map.md`、`.daedalus/topic-board.md` 和 `shared/`。

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
- 批判边界：这个 repo 的设计偏见、历史包袱或产品约束是否会误导当前学习目标。

## Repo-Specific Workflow

1. 根据学习目标列出最多 3 个候选 repo。
2. 对每个 repo 评估匹配度、学习密度、运行风险和 demo 可能性。
3. 给出首选 repo，并说明为什么它最值得优先深入，以及哪些设计不应默认照搬。
4. 如果候选 repo 都不合适，建议调整目标或重新搜索。
5. 将 Agent 的候选比较、选择建议、风险提示和源码拉取建议写入 active topic 的 `guides/02-repo-scout/README.md`。如内容较多，把候选细节拆到同目录专题文件，并从 README 链接。
6. 如果需要准备源码，优先维护 `source/pull_source.sh`，由用户执行拉取；不要默认替用户 clone。
7. 用户如需记录自己的选择思考，可另写 active topic 的 `notes/02-repo-scout/repo-selection-reflection.md`，但不要把它作为阶段必需产物。

## Output Delta

```markdown
## Candidate Repos

### 1. repo/name
- 为什么匹配：
- 核心能力：
- 适合阅读的入口：
- 可能的 mini demo：
- 方案成立的约束：
- 不适合当前目标的地方：
- 风险：
```

```markdown
## Role Split
- daedalus 应该做：提出候选、比较维度、风险提醒和源码拉取脚本。
- 用户必须亲自做：确认首选 repo、接受风险、决定是否执行源码拉取。
- daedalus 可以协助但不能代替：执行 clone、固定 commit、整理 repo-selection；若代为执行必须记录原因。

## Before Completion
- 用户确认的首选 repo：
- 用户接受的风险：
- 用户是否亲自执行 `source/pull_source.sh`：
- active topic 的 `guides/02-repo-scout/README.md` 中的证据：
- 是否允许进入下一阶段：
```

## Repo-Specific Constraints

- 最多推荐 3 个候选 repo。
- 不要只按 star 数排序。
- 必须说明首选 repo 和放弃其他候选的理由。
- 用户指定 repo 时也要客观评估，不要默认附和。
- 首选 repo 也必须写出至少一个局限、偏见、迁移风险或不应照抄的部分。
- 外部源码默认不提交；如由 Agent 拉取，删除嵌套 `.git` 并确认 `source/.gitignore` 生效。
- 完成阶段前，在 active topic 的 `.daedalus/validation-log.md` 记录 daedalus 是否真正帮助用户做出选择。
