---
kind = "tree"
slug = "local-agent-safety"
status = "candidate"
source = "workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions"
created_at = "2026-06-11 00:00:00"
---

# Local Agent Safety Skill Tree

## 回忆钩子

本地 Agent 安全能力不是一个点，而是一棵从意图控制到执行隔离的技能树。

## 现实问题

后续学习 sub agent、context engineering、mini tokio 或真实 OS sandbox 时，需要知道它们如何接到已有能力树上。

## 第一性原理

技能树按“能解决什么问题”组织，而不是按源码模块组织。

## 机制模型

```text
tool intent control
  -> permission model
  -> sandbox execution
  -> retry/escalation
  -> observable events
  -> interactive CLI
```

## 关键不变量

- 权限判断是执行之前的 host responsibility。
- 执行隔离和审批是不同层。
- 事件协议是用户信任和调试的基础。

## 取舍

技能树能提供方向感，但需要随着新 topic 持续重组。

## 不要照搬

不要把阶段顺序当成能力依赖；有些能力可以并行发展。

## 迁移方式

当前树的核心条目：

- [如何让 Agent 安全执行本地命令](../problems/safe-local-command-execution.md)
- [设计本地 Agent 命令执行 Runtime](../skills/design-local-agent-command-runtime.md)
- [Local Agent Command Execution Safety Pattern](../patterns/local-agent-command-execution.md)
- [Codex Tools Permissions Demo Case](../cases/codex-tools-permissions-demo.md)

## 证据来源

- `workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions`

## 复习练习

把这棵树扩展一个“真实 OS sandbox 深入专题”，列出它依赖哪些已有能力。

## 能力依赖

1. ReAct loop 基础理解。
2. Tool runtime 分层。
3. Approval scope 设计。
4. Sandbox runner 抽象。
5. Retry policy 组合。
6. Event protocol 和 TUI 观察。
