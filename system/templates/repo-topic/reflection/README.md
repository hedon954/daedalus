# Reflection

`reflection/` 是 topic 完成时的用户主动回顾区。

这里不是 Agent 总结区，也不是从 notes 自动提取知识的地方。用户需要在完成 topic 后，用自己的语言写出 closeout reflection；Agent 进入总结阶段后，应根据当前 topic 的具体内容重写反思提示，再进行提问、挑战、补外部参照、建议链接和一致性检查。

当 topic 主体学习已经完成但你还需要等周末或大块时间回顾时，topic 可以进入 `awaiting-reflection`。这是正常的 closeout pending 状态，不是失败；它不会占用日常 active learning slot，但仍然需要尽快完成。

## Files

- [`01-knowledge-candidate-map.md`](01-knowledge-candidate-map.md)：进入 closeout 前，daedalus 应先生成候选地图草稿。
- [`02-closeout-prompts.md`](02-closeout-prompts.md)：daedalus 根据当前 topic 重写后的回顾提示。
- [`03-selection.md`](03-selection.md)：用户确认哪些候选归档、延后、删除或修订。
- [`04-archive-evidence.md`](04-archive-evidence.md)：最终知识归档证据和验证结果。
- [`closeout.md`](closeout.md)：用户主动回顾总结。进入 `knowledge-base/` 前必须先由用户完成它。
