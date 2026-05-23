---
title: Close Repo Learning Loop
description: 检查 repo 学习是否闭环，并归档产物、验证结论和可迁移知识。用于完成、暂停或放弃学习任务时。
phase: repo.phase4-closing
---

@system/prompts/common/summarize.md
@system/prompts/common/export-knowledge.md
@system/prompts/common/knowledge-system-extraction.md
@system/prompts/common/compress-context.md

# Close Repo Learning Loop

## Layer Contract

本 prompt 只定义 repo 学习闭环的完成门槛。阶段总结、知识归档和长期上下文压缩来自上方 `@` 引用。

关闭时先关闭 active topic，再在所有 topic 都关闭后关闭 project。不要把一个专题完成误判为整个 project 完成。

## Repo-Specific Trigger

- repo 学习阶段完成。
- 用户准备关闭、暂停或归档任务。
- 需要把已验证内容导出到 `knowledge-base`。

## Repo Completion Evidence

- active topic 的学习任务卡和最终结果。
- repo 选择理由与放弃的候选项。
- 核心问题路线图。
- 运行手册、调试链路、架构图、代码阅读笔记。
- mini demo 的设计、代码位置、验证方式。
- 业务迁移方案。
- 应进入 `knowledge-base` 的可迁移知识点。

## Repo-Specific Workflow

1. 检查 active topic 是否满足 completed 条件；如要关闭 project，必须确认所有 topic 都已 completed 或 abandoned。
2. 汇总产物清单和位置。
3. 区分已验证结论、未解决问题和后续建议。
4. 使用 `Knowledge System Extraction` 从业务目标、现实制约、naive 失败、核心不变量、实现机制、trade-off、最佳实践对比和迁移模式中萃取候选知识体系。
5. 使用 `Export Verified Knowledge` 判断可迁移知识的归档位置。
6. 提醒是否需要创建 review seed，但不要自动创建长期复习计划。
7. 给出 closing report。

## Output Delta

```markdown
## Closing Report
- 是否解决最初问题：
- 最重要的 3 个学习收获：
- 产物清单：
- 知识库归档：
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

- 不要把未验证假设导出为知识库结论。
- 归档必须能帮助下一次学习恢复上下文。
- 如果任务放弃，也要记录放弃原因和已获得价值。
