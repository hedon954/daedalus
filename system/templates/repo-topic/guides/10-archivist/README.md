# 10 Archivist

这个目录用于 topic 收尾阶段。它不是 `knowledge-base/` 正文，也不是用户 reflection 的替代品。

核心流程：

```text
draft candidate map
-> closeout prompts
-> user reflection
-> challenge / repair loop
-> selection
-> archive evidence
```

产物：

- [`01-knowledge-candidate-map.md`](01-knowledge-candidate-map.md)：Agent 贪心扫描后生成的候选地图，全部保持 draft。
- [`02-closeout-prompts.md`](02-closeout-prompts.md)：基于候选地图定制的 closeout 问题。
- [`03-selection.md`](03-selection.md)：用户确认哪些候选归档、延后或删除。
- [`04-archive-evidence.md`](04-archive-evidence.md)：最终归档位置、验证命令和剩余复习计划。
