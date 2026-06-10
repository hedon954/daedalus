---
kind = "case"
slug = "codex-tools-permissions-demo"
status = "stable"
source = "workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions"
created_at = "2026-06-11 00:00:00"
---

# Codex Tools Permissions Demo Case

## 回忆钩子

这个 case 证明：学习 Codex 权限系统不只是读源码，而是可以迁移成一个可运行的 mini agent CLI。

## 现实问题

用户需要理解 Codex 如何组织 tool call、approval、sandbox、retry，并把这种能力迁移到自己的 Agent demo。

## 第一性原理

用 mini demo 验证理解：只有当源码不变量能转成数据结构、状态机、测试和交互，才算真正掌握。

## 机制模型

```text
Codex source reading -> demo design -> Rust implementation -> TUI run -> knowledge pattern
```

## 关键不变量

- Demo 必须跑通真实 ReAct loop。
- Command tool 必须经过 permission/runtime。
- Approval 和 execution events 必须可观察。
- OS sandbox 至少在 macOS 上有真实 runner 验证。

## 取舍

Demo 没有完整复刻 Codex，但保留了最关键的权限不变量；这比泛读所有源码更能形成可迁移能力。

## 不要照搬

不要把 demo 当生产级 agent；它是学习验证器，不是完整产品。

## 迁移方式

把本 case 作为后续学习其他 coding agent tool system 的模板：先抽不变量，再做最小可运行 demo。

## 证据来源

- `workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/README.md`
- `workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/main.rs`
- `workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/.daedalus/state.toml`

## 复习练习

用 10 分钟讲清楚这个 demo 为什么要先做 simulated runner，再做 OS runner。

## 过程摘要

该 topic 从 Codex 权限/沙箱源码出发，完成了 command runtime、approval gateway、retry policy、event protocol、parallel tools、OS runner 和 ratatui CLI REPL。
