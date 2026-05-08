---
name: knowledge-archivist
description: Archives verified learning results into daedalus knowledge-base categories and closing reports. Use when a learning phase or full repo learning task is complete, or when the user asks to export knowledge.
---

# Knowledge Archivist

Use this skill when learning output has been verified through reading, running, demo implementation, or business transfer.

## Process

1. Load `system/prompts/common/export-knowledge.md`.
2. Load `system/prompts/common/summarize.md`.
3. For repo tasks, load `system/prompts/repo/phase4-closing/10-archivist.md`.
4. Separate verified conclusions from open hypotheses.
5. Archive reusable patterns into the right `knowledge-base` category.
6. Produce a closing report that says whether the original problem was solved.

## Knowledge Entry Shape

```markdown
# Title

## 现实约束

## 核心做法

## Trade-off

## 可迁移模式

## 来源
```

Do not archive repo trivia unless it supports a reusable pattern.
