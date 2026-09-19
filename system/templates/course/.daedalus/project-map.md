# Project Map

Project Map 是长期 course learning project 的导航仪表盘。它描述课程主线、章节依赖和跨 topic 学习路线，不替代任何 topic 的 outcome map。

## North Star

- 长期学习素材：{{COURSE_URL}}
- 课程材料入口：[`source/`](../source)
- 当前 active topic：`{{TOPIC_SLUG}}`
- 最终希望沉淀的跨专题能力：

## Shared Context

| 产物 | 用途 | 状态 |
| --- | --- | --- |
| [`shared/syllabus-map.md`](../shared/syllabus-map.md) | 课程章节、概念依赖和必学/可跳过边界 | 草稿 |
| [`shared/course-progress.md`](../shared/course-progress.md) | 课程进度、章节状态和当前 checkpoint | 草稿 |
| [`shared/concept-map.md`](../shared/concept-map.md) | 跨章节概念图、机制问题和薄弱点 | 草稿 |
| [`shared/glossary.md`](../shared/glossary.md) | 跨 topic 术语表 | 草稿 |
| [`shared/evidence-registry.md`](../shared/evidence-registry.md) | 可继承的 verified evidence | 草稿 |
| [`shared/transfer-patterns.md`](../shared/transfer-patterns.md) | 跨 topic 可迁移模式 | 草稿 |

## Active Topic

- slug：`{{TOPIC_SLUG}}`
- title：{{TOPIC_TITLE}}
- topic workspace：[`topics/{{TOPIC_SLUG}}`](../topics/{{TOPIC_SLUG}})
- topic outcome map：[`topics/{{TOPIC_SLUG}}/.daedalus/outcome-map.md`](../topics/{{TOPIC_SLUG}}/.daedalus/outcome-map.md)

## Project Stop Rules

- 不把 topic 草稿直接提升为 shared evidence。
- 不在 project root 记录 topic stage 进度。
- 不把多个 topic 同时设为 active。
- 不把课程代码跑通等同于概念掌握；必须留下观察、解释或迁移证据。
