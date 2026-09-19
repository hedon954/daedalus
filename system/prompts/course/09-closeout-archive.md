---
title: Course Closeout Archive
description: 关闭 course-learning topic，区分课程原文、用户实验、Agent 解释和可迁移知识，并准备知识归档。
phase: course.09-closeout-archive
---

@system/prompts/common/closeout-flow.md
@system/prompts/common/archive-knowledge.md
@system/prompts/common/compress-context.md

# Course Closeout Archive

## Layer Contract

本 prompt 处理 course-learning topic 的 closeout。归档必须区分材料原文、用户亲自验证、Agent 校准和仍薄弱基础。

## Workflow

1. 检查 task-card、outcome-map、todo、lesson notes、review 和 demo evidence。
2. 让用户做 closeout retrospective。
3. 写 `reflection/closeout.md` 和 `reflection/candidate-map.md`。
4. 同步 `shared/evidence-registry.md`、`shared/concept-map.md`、`shared/transfer-patterns.md`。
5. 只把足够深、可独立成文的知识候选推进 knowledge-base。

## Output Delta

```markdown
## Course Closeout
- 完成的能力证明：
- 用户亲自验证：
- Agent 解释：
- 仍薄弱：
- 可迁移模式：
- 不归档内容：
```

## Constraints

- 不把浅总结归档为知识。
- 不把课程权威结论直接当成验证结论。
- 不关闭仍缺少用户证据的 topic。
