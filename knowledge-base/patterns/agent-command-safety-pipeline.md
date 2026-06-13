---
kind = "pattern"
slug = "agent-command-safety-pipeline"
status = "verified"
source = "workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/reflection/closeout.md"
created_at = "2026-06-13"
---

# Agent 本地命令安全执行链路

## 回忆钩子

安全链路不是 `if allowed { run }`，而是 `capability -> approval -> sandbox -> retry -> observation`。

## 现实问题

本地命令执行要同时处理用户授权、文件系统边界、网络权限、命令失败、沙箱拒绝和 UI 可见性。把这些都塞进一个布尔值，会让系统无法解释“为什么可以执行、以什么权限执行、失败后能不能重试”。

## 第一性原理

控制副作用要分层：先识别能力，再判断许可，再选择执行边界，再处理失败后的权限升级，最后把全过程暴露给外部。

## 底层原理

这条链路的每层解决不同问题：

- `Capability`：把工具请求映射到风险类别。
- `ApprovalRequirement`：判断拒绝、需要审批、还是免审批。
- `ExecutionAttempt`：决定 sandbox first 还是 no sandbox first。
- `RetryPolicy`：只在 sandbox denied 等权限边界失败时考虑升级。
- `Observation/Event`：把结果回灌给模型，也让 UI 和用户看到关键安全事件。

## 关键不变量

- capability match 是执行前的第一道 gate。
- `NeedsApproval` 被用户批准后，仍然可以先 sandbox 执行。
- sandbox denied 后的 no-sandbox retry 是第二次更高风险的执行，不能和首次批准混为一谈。
- 重试策略要同时看失败类型、approval policy、capability retry policy 和用户决定。

## 取舍

分层会让数据结构更多，但每层职责更清晰，测试也更容易围绕状态和边界展开。扁平化实现短期更快，但很容易让 retry、approval 和 sandbox 互相污染。

## 不要照搬

不要把 Codex 的具体 enum 名称当成唯一答案。要保留的是分层不变量，不是某个 Rust 类型形状。

## 迁移方式

任何高风险 tool 都可以按这条链路设计：

```text
ToolCall
-> Capability
-> ApprovalRequirement
-> ExecutionAttempt
-> RetryDecision
-> ToolObservation
```

如果工具不是 shell，把 `sandbox` 替换成对应隔离边界，例如 dry-run、事务、配额、审批流或资源 ACL。

## 证据来源

- [closeout.md](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/reflection/closeout.md)
- [tool/runtime.rs](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/tool/runtime.rs)
- [tool/shell/retry.rs](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/tool/shell/retry.rs)
- [Agent 本地命令执行为什么危险](../problems/agent-local-command-execution.md)

## 复习练习

设计一个 `send_email` tool 的安全链路，分别给出 capability、approval scope、隔离方式和 retry gate。

## 反例

只用工具名做白名单，例如“允许 run_command”，却不区分 `pwd`、`npm install`、`curl | sh` 和 `rm -rf`。
