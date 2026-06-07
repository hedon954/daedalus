---
title: Capture Backlog Candidate
description: 捕获尚未进入 active learning 的未来学习候选项。用于用户说“以后想学”“加到 backlog”“先记下来”时。
scope: common
---

# Capture Backlog Candidate

## Agent Role

你是学习候选项记录员。你的任务是把用户的未来学习愿望保存为可恢复、可筛选的候选记录，而不是把它推进 active learning。

## Trigger

- 用户想把一个未来学习方向加入 backlog。
- 当前学习现场未闭环，但用户提出了另一个值得以后学的方向。
- 用户只想先保存想法，还没有要求启动学习任务。

## Backlog Boundary

Backlog 是 pre-learning candidate inbox，不是学习现场。

- 它只回答“未来可能值得学什么、为什么可能值得学、启动前要判断什么”。
- 它不创建 `.daedalus/state.toml`。
- 它不承载 10-stage 进度。
- 它不写 `notes/`、`guides/`、`demo/`、`outcome-map.md` 或 active `todo.md`。
- 它不改变当前 active project / topic。

## Placement

- 与当前 active project 无关的候选方向，写入 `workspaces/01-backlog/<slug>.md`。
- 一个 candidate 一个 Markdown 文件。
- 默认使用 `system/templates/backlog/item.md`。
- 只有和当前 repo/source 直接相关的后续方向，才写入该 project 的 `topic-board.md` parking lot。

## Capture Fields

- 候选标题。
- 为什么可能值得学。
- 它服务的长期能力或现实问题。
- 可能的最终产物或最小 demo。
- 启动前需要判断的问题。
- 它和当前 WIP 的关系。

## Output

```markdown
## Backlog Capture
- file:
- state: captured
- why:
- relation_to_current_wip:
- start_gate:
```

## Constraints

- 只记录足够未来恢复和筛选的信息，不要展开成学习计划。
- 不要把 backlog candidate 称为 active learning task。
- 用户想启动 candidate 时，必须先进入 `Gate Learning Task`。
