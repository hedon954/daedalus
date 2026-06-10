---
kind = "skill"
slug = "design-local-agent-command-runtime"
status = "stable"
level = "practiced"
target_level = "reusable"
source = "workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions"
created_at = "2026-06-11 00:00:00"
---

# 设计本地 Agent 命令执行 Runtime

## 回忆钩子

如果一个 Agent runtime 要接入 shell tool，需要先设计权限状态机，而不是先写 `Command::new("sh")`。

## 现实问题

Agent loop 会收到模型 tool call，但 tool runtime 才能知道 cwd、capability、sandbox profile、approval policy 和 retry policy。

## 第一性原理

Runtime 的职责是把不可信意图转成受控执行尝试，并把每个安全决策暴露为事件。

## 机制模型

```text
ToolCall -> ToolRuntime -> run_command -> ApprovalGateway -> ExecutionRunner -> RetryPolicy -> ToolResult
```

## 关键不变量

- ReAct 层不直接执行 shell。
- ToolRuntime 必须先 plan 再 execute。
- Pure function tool 和 shell tool 可以共用 runtime 外壳，但 capability 语义不同。
- ApprovalGateway 应在 CLI session 内共享。

## 取舍

统一 runtime 让事件和测试更集中，但会让纯函数工具也经过一层看似多余的计划结构。

## 不要照搬

不要把所有 command-specific 字段塞进通用 `ToolCall`；shell 参数解析应该留在 shell tool 内部。

## 迁移方式

先支持 pure function path，再接 command path；先让 event lifecycle 稳定，再做 UI 美化。

## 证据来源

- `workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/tool/runtime.rs`
- `workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/tool/shell/mod.rs`
- `workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/notes/08-demo-coder/13-slice-13-codex-like-cli-closeout.md`

## 复习练习

不看代码，写出 `ToolRuntime::run_tools` 并发执行多个 tool call 时，为什么每个 tool call 的失败不应该取消其他 tool call。

## 薄弱点

复杂 shell 解析、跨平台 sandbox 和长期策略持久化仍需后续专题验证。
