---
kind = "problem"
slug = "agent-local-command-execution"
status = "verified"
source = "workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/reflection/closeout.md"
created_at = "2026-06-13"
---

# Agent 本地命令执行为什么危险

## 回忆钩子

模型生成 `command string` 只是意图，不是执行许可。本地命令执行的本质是受控副作用。

## 现实问题

Coding Agent 需要读文件、跑测试、安装依赖、启动服务，最终都可能落到本地命令执行。如果 Host 直接把模型生成的命令丢给 shell，模型就等于绕过用户和系统边界直接操作文件系统、网络和进程。

## 第一性原理

命令一旦进入本机环境，就不再是纯计算，而是会改变世界的副作用。任何会产生副作用的系统，都需要先回答：

- 谁提出意图。
- 谁拥有执行权。
- 影响边界在哪里。
- 失败后能否扩大权限。
- 外界如何验证它没有绕过控制。

因此，Agent 工具执行的核心不是“怎么调用 shell”，而是“Host 如何把模型意图转成可审计、可限制、可拒绝的执行请求”。

## 底层原理

本地命令的风险来自多个层级叠加：

- shell 语法会把字符串解释为进程、重定向、管道、heredoc 和多命令组合。
- 进程继承 `cwd`、环境变量、文件权限、网络能力和父进程上下文。
- 沙箱只能限制部分资源边界，不能替代审批和策略判断。
- sandbox denied 后如果自动裸跑，就等于把隔离失败变成了权限升级。

## 关键不变量

- 模型只产生意图，Host 才能做权限判断。
- 能力匹配失败必须 fail closed。
- `Allow / Skip approval` 不等于 `bypass sandbox`。
- sandbox denied 后不能自动裸跑。
- 用户授权必须绑定 scope。
- 审批、执行、沙箱、重试和结果必须可观察。

## 取舍

更严格的控制会增加审批打扰和实现复杂度；更宽松的控制会降低安全性和可解释性。合理设计要把审批粒度绑定到真实风险，而不是只绑定命令名或工具名。

## 不要照搬

Codex 的命令级权限模型适合 local coding agent。线上业务 Agent 不一定需要 shell prefix 权限，但仍然需要 capability、approval、isolation/retry 和 observability 这几类边界。

## 迁移方式

遇到任何有副作用的 Agent tool，先把问题翻译成：

```text
model intent -> host capability -> approval/scope -> isolation -> retry gate -> observation
```

然后根据业务资源替换 shell 维度。例如线上系统可以把 scope 绑定到 `user_id`、`workspace_id`、业务 action、资源 ID 和数据分级。

## 证据来源

- [closeout.md](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/reflection/closeout.md)
- [candidate-map.md](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/reflection/candidate-map.md)
- [Agent 本地命令安全执行链路](../patterns/agent-command-safety-pipeline.md)
- [Codex Tools-Permissions 学习案例](../cases/codex-tools-permissions-demo.md)

## 复习练习

解释为什么“用户允许执行命令”不等于“可以不进 sandbox”。再举一个线上业务 Agent 的例子，把 shell scope 替换成业务资源 scope。

## 当前最佳答案

Agent 不能直接执行本地命令。Host 必须把模型意图转成带上下文的执行请求，并通过 capability、approval、sandbox、retry 和 observation 保持控制权。
