# Topic Board

Topic Board 记录当前 course learning project 的专题列表、课程章节映射、状态和继承关系。一次只能有一个 active topic。

## Active Topic

- `{{TOPIC_SLUG}}`：{{TOPIC_TITLE}}

## Topics

| Topic | Title | Lifecycle | Path | Inherits |
| --- | --- | --- | --- | --- |
| `{{TOPIC_SLUG}}` | {{TOPIC_TITLE}} | active | [`topics/{{TOPIC_SLUG}}`](../topics/{{TOPIC_SLUG}}) | - |

## Parking Lot

- 待补充后续 topic。

## Rules

- 新增专题必须使用 `daedalus topic new <slug> --title <title>`。
- 切换专题必须使用 `daedalus topic activate <slug>`。
- 专题完成后必须反向更新 [`shared/evidence-registry.md`](../shared/evidence-registry.md)。
- 专题进入 transfer/capstone 前，必须能指向已学习的课程概念和验证证据。
