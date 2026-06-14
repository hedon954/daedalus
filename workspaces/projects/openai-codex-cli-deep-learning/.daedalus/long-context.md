# Project 长期上下文

只保留跨 topic 可复用的长期上下文，不要粘贴聊天记录。

## 角色边界

- Project root 保存 project 地图、topic board 和 shared context。
- `topics/<slug>/guides/` 保存该专题的 Agent 行动指南。
- `topics/<slug>/notes/` 保存该专题中用户亲自实践和思考后的学习笔记。
- `topics/<slug>/demo/` 保存该专题的 mini demo。
- `shared/` 保存跨专题可复用的源码索引、术语、证据和迁移模式。
- `source/` 保存外部源码缓存，默认不提交到 daedalus 仓库。

## Project 目标

围绕 OpenAI Codex CLI 建立长期 repo learning project，允许按 topic 学习工具系统与权限系统、sub-agent 调度、prompt/context engineering 等方向，并让 topic 之间共享源码索引、运行手册和可迁移模式。

## Current Project State

- 当前没有 active topic，project lifecycle 为 `idle`。
- `tools-permissions`：工具系统与权限系统，已完成 10 个阶段、用户 closeout reflection、candidate-map 确认和 knowledge-base 归档。
- 当前下一步：复盘已完成 topic，或用 `daedalus topic new` / `daedalus topic activate` 启动新专题。

## Topic 路线

- `tools-permissions`：已迁移自旧 single-topic workspace，保留原 10-stage 进度和 demo。
- 后续可新增：`sub-agent-scheduling`、`prompt-context-engineering` 等。

## Shared Decisions

- Project state 只描述 lifecycle、active topic 和 topic 列表。
- Topic state 才描述 10-stage 学习进度。
- `guides/`、`notes/`、`demo/` 默认属于 active topic，不写在 project root。
- 迁移时保留旧学习任务的 `created_at`，并将迁移事件记录为 project transition。

## 已验证的跨专题结论

- 同一 repo 的不同学习方向需要共享 source/runbook/evidence，但每个 topic 必须独立保留 outcome map、todo、notes 和 demo。
- Project complete 前必须先关闭所有 unfinished topics。

## 未解决的跨专题问题

- TUI 目前是最小 project/topic 适配，尚未提供完整 topic 切换视图。
- shared/source-index、shared/glossary、shared/evidence-registry 仍待随着后续 topic 学习逐步充实。

## 恢复上下文提示

恢复时先读 project `.daedalus/state.md` 和 `.daedalus/topic-board.md` 判断是否存在 active topic；若没有 active topic，则不要把 `tools-permissions` 当作当前 WIP，只能作为已完成专题复盘。
