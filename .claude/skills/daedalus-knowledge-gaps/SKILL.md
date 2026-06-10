---
name: daedalus-knowledge-gaps
description: 从 daedalus 技能树、问题入口、知识条目和复习结果中发现能力缺口。用于用户询问下一步学什么、还缺什么、或如何从知识体系反推 backlog/topic 时。
---

# Daedalus Knowledge Gaps

## Goal

从当前知识体系中找出下一个最有价值的学习目标。

## Inputs

- `knowledge-base/trees/`
- `knowledge-base/problems/`
- `knowledge-base/skills/`
- `knowledge-base/drills/`
- current project/topic state
- backlog candidates

## Reasoning Path

1. 从用户真实想解决的问题出发。
2. 识别需要的能力。
3. 对比现有 skills 和证据。
4. 区分完全缺失的知识与验证薄弱的知识。
5. 只有当候选 topic/backlog 能解锁具体能力时才建议启动。

## Output

- 当前最强能力
- 薄弱或脆弱能力
- 被阻塞的真实问题
- 推荐的下一批学习候选
- 启动新 topic 前建议完成的练习
