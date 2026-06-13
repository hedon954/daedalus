---
status = "verified"
source = "workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions"
---

# ReAct 工具运行时

ReAct 的核心不是“模型会调用工具”，而是模型、工具、观察结果和消息历史形成一个闭环。

```text
messages -> LLM -> tool calls -> tool runtime -> observations -> messages
```

工具结果不能直接当成最终答案。它必须先变成 observation / tool message 回灌给模型，模型再基于新上下文继续推理。

## 从第一性原理看

Agent 要解决的是“模型不具备世界访问能力，但任务需要世界反馈”的问题。

ReAct 论文的核心思想是把 reasoning 和 acting 交织起来：reasoning 帮助模型维护计划和处理异常，acting 让模型从外部环境获得新信息。Google Research 对 ReAct 的介绍也强调，action 会产生 observation feedback，而 reasoning trace 本身不改变外部环境。

OpenAI function calling 的工具流也遵循同样结构：模型请求工具，应用侧执行，再把 tool output 发回模型，随后模型继续生成最终响应或更多工具调用。

所以第一性原理是：

```text
工具调用不是答案
工具调用是模型向环境索取 observation 的动作
```

## 底层原理

一个工具运行时至少需要处理三件事：

1. 规划：tool call 是纯函数、命令、MCP、网络请求，还是未知工具。
2. 执行：按照工具类型进入不同 runtime，并保留安全边界。
3. 回灌：把结果变成模型能继续使用的消息。

demo 里 `ToolRuntime` 先规划再执行，是为了把 ReAct loop 和工具细节解耦：

- 纯函数工具直接执行。
- `run_command` 进入 command safety pipeline。
- 多个 tool call 可以并发执行，但结果仍要按 call index 回灌，保证 messages 可预测。
- 事件是旁路观察，不是模型推理输入。

这个区分很重要。`StreamEvent` 服务 UI、信任和调试；真正驱动下一轮推理的是 tool observation。

## 在 Codex demo 中的体现

相关实现：

- [`agent/react.rs`](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/agent/react.rs)：维护 messages、调用 LLM、执行 tool calls、回灌 tool messages。
- [`tool/runtime.rs`](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/tool/runtime.rs)：规划和执行 tool calls。
- [`tool/event_emitter.rs`](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/tool/event_emitter.rs)：集中发出工具生命周期事件。

这次学习中形成的设计判断：

- 同一批 tool call 通常不应互相依赖；否则后一个 tool call 的参数无法稳定生成。
- 并发执行多个 tool call 时，一个失败不应取消其它已发起工具。让 Agent 一次性看到多个结果或多个错误，更有利于下一轮自我修复。
- tool execution event 和 tool observation 要分清。前者给外部看，后者回灌给模型。

## 现实工程取舍

生产 Agent 的 ToolRuntime 不一定只有本地函数和 shell。它可能包括：

- MCP 工具。
- 业务 API。
- 数据库查询。
- 浏览器操作。
- 长任务 workflow。

但抽象仍然类似：

```text
tool call -> plan -> execute -> observe -> message
```

真正容易出错的是把这些层混在一起：模型流、工具执行、安全审批、UI 事件、消息回灌互相直连。demo 最值得保留的不是具体类型名，而是分层边界。

## 关联

- [Agent 本地命令执行安全](../safety-and-permissions/local-command-execution.md)
- [`agent/react.rs`](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/agent/react.rs)
- 外部资料：
  - [ReAct paper](https://arxiv.org/abs/2210.03629)
  - [Google Research: ReAct](https://research.google/blog/react-synergizing-reasoning-and-acting-in-language-models/)
  - [OpenAI Function Calling](https://developers.openai.com/api/docs/guides/function-calling)
