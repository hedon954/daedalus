---
title: Diagram Guidelines
description: 定义 daedalus 生成 Mermaid 图时的简洁性和 Typora 兼容规则。用于 guides 和用户确认后的 notes。
---

# Diagram Guidelines

## When To Draw

- 调用链、事件流、状态流和模块边界适合画图。
- 图只辅助理解，不替代用户自己的解释。
- 如果用户还没有验证或回答，不要把完整结论图直接写入 `notes/`；可以先写入 `guides/` 作为阅读地图。

## Mermaid Compatibility

- 节点 ID 不使用 Mermaid 保留字，例如 `loop`、`end`、`graph`、`subgraph`。
- `sequenceDiagram` 的 participant alias 使用大写或明确缩写，例如 `SUB`、`APP`、`CORE`。
- participant 展示名包含空格、符号或下划线时，用双引号包裹。
- message 文本中避免写容易被解析误判的 `->`；可以用 `to`、`then` 或中文描述。
- 不在 Mermaid 中写显式颜色或样式，让渲染器使用默认主题。
- 图保持小而清晰；复杂系统拆成多张图。

## Notes Rule

写入 `notes/` 的图应优先表达用户已经观察、回答或验证过的链路。Agent 推测性的完整图应放入 `guides/`，并标注为待验证。
