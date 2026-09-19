---
name: discovery-driven-topic-selection
overview: 追踪 daedalus 选题发现能力改进和 LoRA Fine-tuning Feedback Loop Lab 启动计划；详细方案以 docs/plan 为准。
status: completed
todos:
  - id: formalize-plan-location-rule
    content: 确认正式详细计划放在 docs/plan，.codex/plans 只做带 status/todos 的 Codex 追踪卡并引用正式计划。
    status: completed
  - id: write-daedalus-improvement-plan
    content: 编写 daedalus 澄清、挖掘、选题能力改进正式计划。
    status: completed
  - id: write-lora-topic-plan
    content: 编写 LoRA Fine-tuning Feedback Loop Lab 新 topic 启动正式计划。
    status: completed
  - id: implement-topic-discovery
    content: 按 docs/plan/13 实现 topic discovery prompt、artifact 和 clarify/gatekeeper 集成。
    status: completed
  - id: start-lora-topic
    content: 按 docs/plan/14 使用 daedalus lifecycle 创建新 project/topic。
    status: completed
isProject: false
---

# Codex Tracking Card

详细计划不在此处展开。本文件只用于 Codex `status` / `todos` 追踪。当前 daedalus 改进和新 topic 初始化均已完成。

## Canonical Plans

- [澄清、挖掘、选题能力改进方案](../../docs/plan/13-discovery-driven-topic-selection.md)
- [LoRA Fine-tuning Feedback Loop Lab 启动计划](../../docs/plan/14-lora-finetuning-feedback-loop-topic.md)

## Tracking Rule

- `docs/plan/`：正式详细计划，作为 daedalus 项目的长期设计资料。
- `.codex/plans/`：Codex 执行追踪卡，只保留 `status`、`todos` 和正式计划链接。
- `workspaces/`：实际进入 daedalus lifecycle 后的 project/topic 状态。
