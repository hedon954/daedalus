---
kind = "source-map"
slug = "codex-tools-permissions"
status = "verified"
source = "workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions"
created_at = "2026-06-13"
---

# Codex Tools-Permissions 来源映射

## 回忆钩子

Codex 是设计素材，不是唯一真理；demo 和 closeout 才决定哪些理解真正被吸收。

## 现实问题

学习 repo 时，容易把源码调用链当成知识。来源映射要回答：Codex 源码、demo、notes、guides、closeout、外部资料分别贡献了什么，哪些可以迁移，哪些不能照搬。

## 第一性原理

知识归档要可追溯。没有来源，后续无法判断一个结论是用户理解、Agent 总结、源码事实、demo 经验，还是外部最佳实践。

## 底层原理

本 topic 的来源分层：

- Codex 源码：提供 exec policy、approval、sandbox retry 的参考设计。
- demo：验证哪些不变量必须保留，哪些可以简化。
- notes：记录用户理解、纠正和设计讨论。
- guides：记录 Agent 引导路径和外部补充。
- closeout：确认用户最终理解和迁移边界。
- external docs：补 Rust async、Tokio、OpenAI streaming、OS sandbox 的底层参照。

## 关键不变量

- 只出现在 guides 中、没有被用户吸收或 demo 验证的内容，不能直接当作用户知识归档。
- closeout 是用户理解锚点，但候选挖掘不能只看 closeout。
- 来源映射要保留 not-to-copy 边界。

## 取舍

来源映射会增加维护成本，但能防止知识库变成漂亮却不可追溯的总结。对于大型 topic，source-map 很值得保留。

## 不要照搬

不要把每个源码文件都列成知识来源。只保留影响不变量、trade-off、demo 设计或迁移边界的来源。

## 迁移方式

每次 repo learning closeout 后，至少补一份 source-map：

```text
source fact -> user understanding -> demo evidence -> knowledge entry
```

## 证据来源

- [closeout.md](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/reflection/closeout.md)
- [notes](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/notes)
- [guides](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/guides)
- [demo/src](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Tokio spawning tutorial](https://tokio.rs/tokio/tutorial/spawning)
- [OpenAI streaming responses](https://developers.openai.com/api/docs/guides/streaming-responses)
- [OpenAI function calling](https://developers.openai.com/api/docs/guides/function-calling)
- [Tokio documentation](https://docs.rs/tokio)

## 复习练习

任选一个知识条目，追溯它来自 closeout、notes、guides、demo 和外部资料中的哪些证据。找不到证据的结论要降级为候选或删除。

## 来源结构

```text
Codex source reading
-> user notes / corrections
-> demo implementation and tests
-> closeout reflection
-> candidate-map confirmation
-> knowledge-base entries
```
