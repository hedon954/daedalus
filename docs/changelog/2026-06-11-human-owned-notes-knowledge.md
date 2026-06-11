# 2026-06-11 Human-Owned Notes 知识库协议

## 变更

- 新增 `system/prompts/common/human-owned-notes.md`，明确 notes 和 closeout retrospective 必须由用户主动完成。
- 删除 `daedalus-knowledge-*` AI 知识提取型 skills 和跨 agent skill 软链接。
- 删除 `Knowledge System Extraction` prompt 和 `system/templates/knowledge-system/` 旧模板。
- 删除 project shared 下的 `knowledge-system/` scaffold，不再维护 topic candidate -> shared candidate 的 AI 提取链路。
- 改写 repo-learning coach、archivist、operating contract、README 和 CLI contract，统一为 human-owned notes / reviewed human retrospective understanding。

## 决策

daedalus 不做 automatic summarizer，也不做 main extractor。

用户主动回顾是能力提升的必要流程。AI 的职责是降低开始成本、提高反馈密度、挑战含糊和跳步、搜索公网资料与业内最佳实践、补链接和做一致性检查。

## 保留能力

- `daedalus knowledge template`
- `daedalus knowledge index`
- `daedalus knowledge list`
- `daedalus knowledge link-check`
- `daedalus knowledge validate`

这些能力只负责结构和校验，不生成知识结论。
