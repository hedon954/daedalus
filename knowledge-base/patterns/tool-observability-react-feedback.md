---
kind = "pattern"
slug = "tool-observability-react-feedback"
status = "verified"
source = "workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/reflection/closeout.md"
created_at = "2026-06-13"
---

# 工具可观察性与 ReAct 回灌

## 回忆钩子

事件给人看，observation 给模型看；两者相关，但不是同一个东西。

## 现实问题

Agent 调用工具时，用户需要知道工具有没有被审批、是否在 sandbox、是否发生 retry；模型则需要拿到工具结果继续推理。如果只返回一个字符串，UI 不可信；如果只发事件，模型无法稳定进入下一轮。

## 第一性原理

ReAct 的闭环是 `messages -> model -> tool call -> observation -> messages`。工具执行不是终点，而是下一轮推理的输入。

## 底层原理

- `StreamEvent` 记录过程：thinking、text、tool selected、approval、execution started/finished、tool result。
- `ToolObservation` 回灌模型：包含 tool call id、工具名和执行结果。
- UI 事件和模型消息应共享事实来源，但服务不同消费者。
- 多个 tool call 可以并发执行，但回灌给模型时需要保持可预测的对应关系。

## 关键不变量

- 工具结果不能绕过 messages 直接变成最终答案。
- approval / execution / retry 事件必须覆盖安全关键节点。
- 事件发射最好集中管理，避免分散在业务代码里漏发。
- ReAct loop 要保留短期消息记忆，否则多轮对话会断。

## 取舍

事件越细，UI 和调试越可信，但代码更容易散。集中式 `EventEmitter` 能降低漏发风险，但要避免把业务流程过度耦合到事件层。

## 不要照搬

不要为了“事件完整”而让事件驱动核心业务。事件是旁路观察；真正驱动下一轮模型推理的是 tool observation。

## 迁移方式

设计 Agent tool runtime 时，明确两条输出：

```text
execution events -> UI / audit / debug
tool observation -> messages -> model
```

## 证据来源

- [closeout.md](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/reflection/closeout.md)
- [agent/react.rs](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/agent/react.rs)
- [tool/event_emitter.rs](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/tool/event_emitter.rs)
- [tool/runtime.rs](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/tool/runtime.rs)

## 复习练习

画出一次 tool call 的双通道输出：哪些信息进入 UI event，哪些信息进入 model observation。解释为什么二者不能合并成一个字符串。

## 反例

工具执行成功后只在终端打印结果，但没有生成 tool message，导致模型下一轮不知道工具结果。
