---
kind = "problem"
slug = "safe-local-command-execution"
status = "stable"
source = "workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions"
created_at = "2026-06-11 00:00:00"
---

# 如何让 Agent 安全执行本地命令

## 回忆钩子

当 Agent 需要运行 `npm test`、`cargo build`、`python script.py` 或 shell 命令时，先问“执行边界是什么”，不要直接问“命令能不能跑”。

## 现实问题

本地命令执行同时牵涉文件系统、网络、进程、用户审批和模型自我修复。任何一个环节默认放行，都会让 Agent 从助手变成不受控的自动化主体。

## 第一性原理

安全执行不是一个布尔判断，而是一条按上下文逐步收窄权限的路径：意图、能力、审批、沙箱、执行、重试、观察。

## 机制模型

```text
tool call -> capability match -> approval requirement -> sandbox attempt -> retry gate -> observation
```

## 关键不变量

- 未匹配能力时 fail closed。
- 审批必须绑定 scope，不绑定裸 command name。
- sandbox 失败不能自动无沙箱重试。
- tool 执行事件必须可观察。

## 取舍

更严格的权限模型会增加实现复杂度和用户审批成本；更宽松的模型会提高流畅度，但扩大误执行影响面。

## 不要照搬

不要在业务 demo 早期追求完整系统级 sandbox，先验证执行状态机和审批语义。

## 迁移方式

先用 simulated runner 跑通状态机，再接 OS runner；先支持单命令，再扩展多命令解析。

## 证据来源

- `workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/README.md`
- `workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/notes/09-biz-solver/README.md`

## 复习练习

给出一个需要网络权限的命令，说明它在 `ApprovalPolicy::Never` 和 `OnFailure` 下分别如何失败或重试。

## 当前最佳答案

用 [Local Agent Command Execution Safety Pattern](../patterns/local-agent-command-execution.md) 作为默认设计骨架。
