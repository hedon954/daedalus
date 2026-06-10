# Project 验证日志

记录 daedalus 在 project/topic 编排层面的引导效果。

## 使用规则

- 普通学习笔记写入 active topic 的 `notes/`。
- 专题阶段复盘写入 active topic 的 `.daedalus/validation-log.md`。
- 这里只记录 project 层问题：topic 切换、shared context 复用、WIP 管理、迁移和归档。

## 2026-05-23 多专题迁移

- 事件：将 `openai-codex-cli-deep-learning` 从 single-topic workspace 迁移为 multi-topic project。
- 有效引导：旧学习进度被迁入 `topics/tools-permissions`，project root 开始承担 active topic、topic board 和 shared context 职责。
- 用户亲自完成的实践：用户提出同一个 repo 需要支持多个专题反复走 stage，同时希望专题之间互相助力。
- Agent 代替用户过多的地方：迁移由 Agent 执行，但通过 CLI migration 和 validate 留下可复查路径。
- Project/topic/template/CLI 改进建议：后续应完善 TUI topic 切换视图，并在 shared context 中逐步沉淀跨 topic 证据。
