---
title: Repo Learning Reflection Loop
description: 检查 repo 学习是否闭环，并归档产物、验证结论和可迁移知识。用于完成、暂停或放弃学习任务时。
phase: repo.phase4-closing
---

@system/prompts/common/summarize.md
@system/prompts/common/human-owned-notes.md
@system/prompts/common/archive-knowledge.md
@system/prompts/common/compress-context.md
@system/prompts/common/critical-lens.md
@system/prompts/common/diagram-guidelines.md
@system/prompts/common/closeout-flow.md

# Repo Learning Reflection Loop

## Layer Contract

本 prompt 只定义 repo 学习闭环的完成门槛。阶段总结、知识归档和长期上下文压缩来自上方 `@` 引用。

关闭时先关闭 active topic，再在所有 topic 都关闭后关闭 project。不要把一个专题完成误判为整个 project 完成。

## Repo-Specific Trigger

- repo 学习阶段完成。
- 用户准备关闭、暂停或归档任务。
- 用户准备完成 closeout retrospective 并归档到 `knowledge-base`。

## Repo Completion Evidence

- active topic 的学习任务卡和最终结果。
- repo 选择理由与放弃的候选项。
- 核心问题路线图。
- 运行手册、调试链路、架构图、代码阅读笔记。
- mini demo 的设计、代码位置、验证方式。
- 业务迁移方案。
- 用户 closeout retrospective。
- 经过用户回顾和 AI challenge 后，才可能进入 `knowledge-base` 的可迁移理解。
- 学习过程中识别出的局限、失败模式、not-to-copy 和 demo 中的忠实模仿边界。

## Repo-Specific Workflow

先按 `Closeout Push Protocol` 判断当前状态，不要从头重走所有步骤。

1. 判断当前 closeout gate：`AwaitingReflection`、`ReflectionWritten`、`ChallengeResolved`、`KnowledgeGate`、`ArchiveReady`、`TopicCompleted`。
2. 只执行当前 gate 需要的动作；如果上一 gate 已经在对话中完成，承认完成并推进，不要让用户重复证明。
3. 检查 active topic 是否满足 completed 条件；如要关闭 project，必须确认所有 topic 都已 completed 或 abandoned。
4. 汇总产物清单和位置。
5. 检查用户是否已经完成 closeout retrospective；没有则先引导用户完成，不要代写。
6. 区分用户已验证理解、未解决问题和后续建议。
7. 如果 closeout 需要图，检查图是否只承担分层和主方向，关键决策点是否由文字解释。
8. 检查学习过程中出现的通用方法、思维工具或 Agent 失败模式是否需要提升到 project shared、system prompts/templates 或 knowledge-base candidate，而不是只留在当前 topic。
9. closeout review 聚焦核心主线，同时点名重要旁路知识和基础薄弱点；进入 knowledge-base gate 后，按 `Archive Reviewed Human Knowledge` 贪心扫描 closeout、guides、notes、demo、测试、外部参照和第一性原理旁支，判断可迁移理解的归档位置。
10. 提醒是否需要创建 review seed，但不要自动创建长期复习计划。
11. 给出 closing report 或下一 gate 的唯一行动。

## Output Delta

```markdown
## Closing Report
- 是否解决最初问题：
- 最重要的 3 个学习收获：
- 产物清单：
- 知识库归档：
- 忠实模仿与迁移取舍：
- 不应照抄的边界：
- 复习计划建议：
- 后续可复用上下文：
- 下一步建议：
```

## Completion Gate

如果没有 demo、没有运行验证或没有业务迁移，不能标记为 completed，只能标记为 paused 或 abandoned。

CLI 操作必须遵守：

- 专题完成：`daedalus topic complete <topic-slug>`。
- 专题放弃：`daedalus topic abandon <topic-slug>`。
- 整个 project 完成：确认没有 active topic 后再运行 `daedalus task complete <project-dir> --reason <reason>`。

## Repo-Specific Constraints

- 不要把未验证假设归档为知识库结论。
- 不要把 repo 的局部最优方案写成通用最佳实践；必须保留它成立的约束、局限和迁移边界。
- 不要把可复用的学习方法或 Agent 操作教训只埋在 topic closeout 里；应判断是否需要提升到更高层级。
- 归档必须能帮助下一次学习恢复上下文。
- 如果任务放弃，也要记录放弃原因和已获得价值。
