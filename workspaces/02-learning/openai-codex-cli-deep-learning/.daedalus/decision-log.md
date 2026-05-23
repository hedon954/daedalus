# Project 决策记录

记录会影响 project 范围、topic 路线、shared context 或状态流转的关键决策。

## 2026-05-09 15:58:52

- 决策：初始化 repo learning project `openai-codex-cli-deep-learning`。
- 原因：开启 OpenAI Codex CLI 的 filesystem-first 深度学习闭环。
- 影响：原 single-topic 进度后续迁入 `topics/tools-permissions`。

## 2026-05-23

- 决策：将 project 改造成多专题结构，并把当前学习迁移为 `tools-permissions` topic。
- 原因：同一个 Codex repo 后续还需要学习 sub-agent 调度、prompt/context engineering 等独立专题；每个 topic 需要独立 10-stage，又要共享源码索引和证据。
- 影响：project root 只保存 project state、topic board 和 shared context；专题进度、notes、guides、demo 移入 `topics/tools-permissions/`。
