---
name: learning-context-maintainer
description: Maintains durable learning context, todo state, stage summaries, and resume notes for long-running daedalus learning sessions. Use when context is long, a session is paused or resumed, or the agent needs to compact learning state into files.
---

# Learning Context Maintainer

Use this skill when a learning session spans multiple conversations or needs recoverable state.

## Required Inputs

- Current learning goal.
- Current phase.
- Done, doing, next, and blocked todo items.
- Key decisions and open questions.
- Files, diagrams, demos, and notes produced so far.

## Process

1. Load `system/prompts/common/compress-context.md` before compacting.
2. Load `system/prompts/common/resume.md` before resuming.
3. Preserve only state that changes the next action.
4. Delete repeated explanations, stale hypotheses, and chat-only filler.
5. Keep the next action small and explicit.

## Context Template

```markdown
# 学习上下文

## 学习目标

## 当前阶段

## 已完成事项

## 正在进行

## 下一步

## 关键决策

## 未解问题

## 生成的文件与资料
```
