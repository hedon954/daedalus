---
title: Transfer Repo Learning To Business
description: 将 repo 和 mini demo 中已验证的模式迁移回用户业务或工程问题。用于实践验证后、最终归档前形成应用方案。
phase: repo.phase3-practice
---

@system/prompts/common/first-principles.md
@system/prompts/common/question-roadmap.md
@system/prompts/common/summarize.md

# Transfer Repo Learning To Business

## Layer Contract

本 prompt 只定义 repo 学习结果迁移回业务问题的动作。迁移问题生成、因果链解释和总结方式来自上方 `@` 引用。

业务迁移产物默认写入 active topic；跨专题通用模式再同步到 project root 的 `shared/transfer-patterns.md`。

## Repo-Specific Trigger

- mini demo 已实现或核心模式已被验证。
- 用户需要把学习结果用于自己的业务问题。
- 学习任务准备进入最终归档前。

## Repo-Specific Workflow

1. 重述用户最初的问题和约束。
2. 提取 repo/demo 中真正可迁移的模式。
3. 判断哪些模式可以直接用，哪些需要改造，哪些不能用。
4. 给出一个最小业务方案：架构、接口、数据流、风险和验证方式，并写入 active topic 的 `notes/09-biz-solver/README.md` 或同目录专题文件。
5. 标注还需要补学或实验的内容。

## Output Delta

```markdown
## Business Application
- 原始问题：
- 可迁移模式：
- 目标方案：
- 关键 trade-off：
- 最小验证：
- 风险：
- 后续行动：
```

## Repo-Specific Constraints

- 不要把 repo 的方案机械搬运到业务里。
- 必须重新检查用户场景的现实约束。
- 如果缺少业务上下文，先列出最小需要确认的信息。
