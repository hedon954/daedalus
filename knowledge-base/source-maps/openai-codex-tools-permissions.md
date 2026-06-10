---
kind = "source-map"
slug = "openai-codex-tools-permissions"
status = "stable"
source = "workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions"
created_at = "2026-06-11 00:00:00"
---

# OpenAI Codex Tools / Permissions Source Map

## 回忆钩子

Codex 源码的学习价值不在于 API 名称，而在于它如何把 tool call 变成受控执行链路。

## 现实问题

读大型源码容易陷入细节，需要明确哪些源码结论服务 demo 和业务迁移。

## 第一性原理

Source map 要回答“从这个素材中提取了什么、不提取什么、为什么”。

## 机制模型

```text
source file -> invariant -> demo decision -> knowledge entry
```

## 关键不变量

- `Decision` 和 `bypass_sandbox` 不是同一层。
- `ApprovalKey` 需要 command、cwd、sandbox permissions、additional permissions 等上下文。
- sandbox failure 后是否 retry 由 approval policy、network context 和 retry gate 共同决定。

## 取舍

Codex 的完整实现服务真实产品；学习 demo 只保留对本地 command runtime 有迁移价值的部分。

## 不要照搬

不要把所有平台 sandbox、TUI 内部细节和 MCP elicitation 直接搬进 mini demo。

## 迁移方式

把源码结论映射到 [Local Agent Command Execution Safety Pattern](../patterns/local-agent-command-execution.md)。

## 证据来源

- `workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/notes/06-code-reader/README.md`
- `workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/notes/08-demo-coder/README.md`

## 复习练习

解释为什么 heredoc 复杂解析不应该自动生成 future allow amendment。

## 来源结构

核心阅读集中在 exec policy、shell/unified exec request assembly、orchestrator retry 和 sandboxing。
