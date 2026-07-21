# Project Map

Project Map 是长期 course learning project 的导航仪表盘。它描述课程主线、章节依赖和跨 topic 学习路线，不替代任何 topic 的 outcome map。

## North Star

- 长期学习素材：Hugging Face LLM Course 及其相关官方文档、notebooks 和开源示例
- 课程材料入口：[`source/`](../source)
- 当前 active topic：`lora-feedback-loop`
- 最终希望沉淀的跨专题能力：以 Hugging Face LLM Course 为主学习材料，用系统工程第一性原理理解 AI 模型、训练、评测、反馈和部署链路，避免只停留在应用层 glue code。

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

- slug：`lora-feedback-loop`
- title：LoRA 微调与数据反馈闭环
- topic workspace：[`topics/lora-feedback-loop`](../topics/lora-feedback-loop)
- topic outcome map：[`topics/lora-feedback-loop/.daedalus/outcome-map.md`](../topics/lora-feedback-loop/.daedalus/outcome-map.md)

## Project Stop Rules

- 不把 topic 草稿直接提升为 shared evidence。
- 不在 project root 记录 topic stage 进度。
- 不把多个 topic 同时设为 active。
- 不把课程代码跑通等同于概念掌握；必须留下观察、解释或迁移证据。

## Course Learning Migration

- 课程主页：https://huggingface.co/learn/llm-course/en
- 新 shared 入口：[`shared/syllabus-map.md`](../shared/syllabus-map.md)、[`shared/course-progress.md`](../shared/course-progress.md)、[`shared/concept-map.md`](../shared/concept-map.md)
- 课程实验现场已迁入 `guides/04-lesson-lab`；后续新 guide 使用 course-learning 阶段名。
