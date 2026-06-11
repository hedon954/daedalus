---
name: daedalus-knowledge-promote
description: 判断 daedalus 候选知识是否可以晋升为稳定知识，并更新条目状态、证据链和索引。用于用户要求稳定、确认、晋升或验证 knowledge-base 条目时。
---

# Daedalus Knowledge Promote

## Contract

晋升是一种判断，编辑完成后，用 CLI 做确定性检查：

```bash
daedalus knowledge index
daedalus knowledge validate
daedalus knowledge link-check
```

## Promotion Gate

候选条目只有满足以下条件，才可以变为 stable：

- source evidence
- user calibration or practical validation
- problem entry
- capability exit
- mechanism model
- transfer boundary
- at least one drill
- no unresolved critical objection

## Output

晋升时，更新 Markdown status，并说明：

- 为什么它已经足够稳定
- 哪些证据支撑它
- 后续还应该复习什么
- 哪些其他条目应该链接到它
