# Project Map

Project Map 是长期 repo learning project 的导航仪表盘。它描述整个 repo/source 的长期学习计划，不替代任何 topic 的 outcome map。

## North Star

- 长期学习素材：Hugging Face LLM Course 及其相关官方文档、notebooks 和开源示例
- 共享源码位置：[`source/`](../source)
- 当前 active topic：`lora-feedback-loop`
- 最终希望沉淀的跨专题能力：以 Hugging Face LLM Course 为主学习材料，用系统工程第一性原理理解 AI 模型、训练、评测、反馈和部署链路，避免只停留在应用层 glue code。

## Shared Context

| 产物 | 用途 | 状态 |
| --- | --- | --- |
| [`shared/runbook.md`](../shared/runbook.md) | 跨 topic 复用的启动、运行和调试入口 | 草稿 |
| [`shared/architecture-map.md`](../shared/architecture-map.md) | 跨 topic 稳定架构边界 | 草稿 |
| [`shared/source-index.md`](../shared/source-index.md) | 源码模块索引 | 草稿 |
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
