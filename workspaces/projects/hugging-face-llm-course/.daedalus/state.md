# Project 状态

> 从 [`.daedalus/state.toml`](state.toml) 生成。不要手动编辑。

## 当前状态

- Project：`hugging-face-llm-course`
- 类型：`course-learning`
- 生命周期：`active`
- Workspace Bucket：`projects`
- Active Topic：`lora-feedback-loop`
- Pending Closeout：`none`
- Course URL：https://huggingface.co/learn/llm-course/en
- 下一步：继续 course-learning active topic `lora-feedback-loop`，按课程阶段推进下一份 guide。

## Topics

- `lora-feedback-loop`: LoRA 微调与数据反馈闭环 (active) -> [`topics/lora-feedback-loop`](../topics/lora-feedback-loop)

## Project Files

- `.daedalus/project-map.md`: present
- `.daedalus/topic-board.md`: present
- `shared/syllabus-map.md`: present
- `shared/course-progress.md`: present
- `shared/concept-map.md`: present
- `shared/evidence-registry.md`: present

## 最近状态流转

> 共 2 条状态流转；下面显示最近 2 条，完整历史见 [`.daedalus/state.toml`](state.toml) 的 `[[transitions]]`。

- `2026-06-14 14:42:45` 由 `daedalus-cli` 对 `project` 执行 `init`：初始化 course learning project，并创建初始专题 `lora-feedback-loop`。
- `2026-06-23 00:53:42` 由 `daedalus-cli` 对 `project` 执行 `migrate`：确认项目使用 course-learning 语义。

## 下一步 CLI 建议

- `daedalus state render --project`
- `daedalus validate`
