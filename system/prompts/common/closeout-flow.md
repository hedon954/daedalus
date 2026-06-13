---
title: Closeout Push Protocol
description: 定义 topic closeout 的推进状态机。用于用户进入回顾、确认 closeout、完成 challenge、进入 knowledge-base gate 或关闭 topic 时。
scope: common
---

# Closeout Push Protocol

Closeout 不是开放式聊天，它是一条需要持续前推的收尾状态机。Agent 的职责是降低用户摩擦：判断当前 gate、执行能执行的同步动作、给出唯一清晰的下一步。

## State Machine

```text
AwaitingReflection
  -> ReflectionWritten
  -> ChallengeResolved
  -> KnowledgeGate
  -> ArchiveReady
  -> TopicCompleted
```

## Gates

### AwaitingReflection

触发：

- `reflection/closeout.md` 为空、仍是模板，或用户明确说还没写完。

Agent 动作：

- 不代写正文。
- 给用户 3-5 个贴合当前 topic 的回顾问题。
- 明确说下一步是用户写 reflection，而不是继续扩展源码、demo 或知识库。

### ReflectionWritten

触发：

- 用户已经写出 closeout。
- 用户说“写好了”“我觉得 OK 了”“这份 closeout 可以了”。

Agent 动作：

- 读取 closeout、topic map、todo、state 和必要证据。
- 判断是否还需要 challenge；如果对话中已经完成实质 challenge，不要重复要求用户再走一遍。
- 如有缺口，只提出阻塞 closeout 的最少问题。
- 如无阻塞，立即把状态推进到 `ChallengeResolved`。

### ChallengeResolved

触发：

- Agent 已经 challenge 过关键理解，或用户已回答关键 challenge。
- 关键边界已经明确：理解变化、证据、不变量、迁移边界、不可照搬、后续 gap。

Agent 动作：

- 同步 `.daedalus/outcome-map.md`、`.daedalus/todo.md`、`.daedalus/state.toml`。
- 渲染 `state.md`。
- 运行最小 validate。
- 提交 closeout checkpoint，除非用户明确要求不提交。
- 下一步必须指向 `KnowledgeGate`，不要再说“继续 review closeout”。

### KnowledgeGate

触发：

- closeout reflection 和 challenge 已完成。

Agent 动作：

- 以 closeout reflection 为主判断材料，同时扫描当前 topic 的 `guides/`、`notes/`、`demo/`、run/test evidence，提出候选知识条目。
- 候选必须说明它来自用户已吸收或验证的理解，而不是 Agent 在 `guides/` 中单方面写过的内容。
- 区分 closeout 和 knowledge extraction：closeout 聚焦核心主线，但要点名重要旁路知识和基础薄弱点；knowledge extraction 必须贪心扫描所有 guides/notes/closeout/demo/test/external evidence。
- 先给完整候选地图，再建议本轮最小归档集合。不要因为默认输出简短而漏掉 Rust、OS、runtime、UI、测试、并发等过程中学到的可迁移能力。
- 每个候选只给：标题、类型、为什么值得归档、证据来源、适用边界。
- 补必要外部参照或旧知识链接。
- 等用户确认后再写 `knowledge-base/` 正文。

### ArchiveReady

触发：

- 用户确认哪些知识条目可以归档。

Agent 动作：

- 创建或更新 knowledge-base 条目。
- 更新索引和链接。
- 运行 `daedalus knowledge validate`、`daedalus knowledge link-check` 和 workspace validate。
- 同步 topic map。
- 进入 topic completion。

### TopicCompleted

触发：

- 知识归档通过验证，用户确认可以关闭 topic。

Agent 动作：

- 运行 `daedalus topic complete <topic-slug>`。
- 渲染并验证 project/topic 状态。
- 提交 closing checkpoint。
- 输出下一步建议：复习计划、backlog、下一个 topic，或者休息。

## Push Rules

- 不要把“下一步”写成模糊的“继续完善”。必须写成一个 gate 的具体动作。
- 用户说“OK / 可以 / 我觉得完成了”时，Agent 要判断能否推进 gate，而不是停下来等用户再次指挥。
- 如果当前 gate 已经在对话中完成，承认完成并更新地图，不要机械重复流程。
- 每完成一个 gate，都要同步学习地图；如果涉及文件变更，优先提交一个小 checkpoint。
- 只有真正阻塞用户理解或归档正确性的问题，才把控制权交还给用户。
