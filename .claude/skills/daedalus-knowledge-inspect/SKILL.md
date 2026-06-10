---
name: daedalus-knowledge-inspect
description: 审计 daedalus knowledge-base 的结构、证据、复习练习、重复项和孤立项。用于用户询问知识库是否健康、过期、重复、缺证据或需要复习时。
---

# Daedalus Knowledge Inspect

## Contract

使用该 skill 判断知识质量。CLI 只用于确定性检查：

```bash
daedalus knowledge index
daedalus knowledge validate
daedalus knowledge link-check
daedalus knowledge list
```

## Inspect Dimensions

- 是否有清晰的问题入口。
- 是否能说明能力出口。
- 是否有机制模型，而不只是结论。
- 是否有来源证据和 demo / case 证据。
- 是否写明 trade-off、局限和不要照搬的部分。
- 是否有复习或迁移练习。
- 是否重复、孤立、过时或分类错误。

## Output

输出简洁审计：

- 健康条目
- 薄弱条目
- 缺失链接或证据
- 重复或可合并条目
- 建议复习练习
- 必要时提出重组建议

除非用户明确要求，不要自动大规模改写文件。
