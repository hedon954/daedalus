---
kind = "case"
slug = "codex-tools-permissions-demo"
status = "verified"
source = "workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions"
created_at = "2026-06-13"
---

# Codex Tools-Permissions 学习案例

## 回忆钩子

这个 topic 的核心收获是：通过模仿 Codex，亲手做出一个能安全执行本地命令的 mini Agent CLI。

## 现实问题

学习 Codex tools-permissions 不是为了背源码，而是为了理解 coding agent 如何把模型意图变成受控本地副作用，并把这个能力迁移到自己的 Agent/CLI 设计里。

## 第一性原理

源码学习必须落到可运行 demo，否则很容易只记住调用链。demo 要保留被学习系统的核心不变量，哪怕生产能力暂时简化。

## 底层原理

本案例贯穿了：

- Codex exec policy / approval / sandbox / retry 的源码阅读。
- Rust async streaming 与 OpenAI-compatible API 接入。
- ToolRuntime、ApprovalGateway、ExecutionRunner、EventEmitter 的 demo 实现。
- `sandbox-exec` 真实 OS sandbox 尝试。
- ratatui Agent CLI 的事件循环和状态渲染。

## 关键不变量

- 模型不能直接拥有本地命令执行权。
- demo 可以简化 parser、sandbox 和 UI，但不能删掉 capability、approval、sandbox、retry、observation 这些核心边界。
- closeout 要由用户主动写，knowledge-base 只能归档 reviewed human understanding。

## 取舍

demo 从 SimulatedExecutionRunner 起步，随后补 OsExecutionRunner 和 ratatui CLI。这个路线先保证主链路可理解，再逐步提高真实可用性。

## 不要照搬

不要把 demo 当成生产级 Codex clone。复杂 shell parser、跨平台 sandbox、多用户权限、完整 MCP/TUI 和长期权限存储都仍需要生产级设计。

## 迁移方式

下次学习复杂 repo 时，可以继续复用这条路径：

```text
源码不变量 -> mini demo -> 运行验证 -> closeout -> candidate-map -> knowledge-base
```

## 证据来源

- [closeout.md](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/reflection/closeout.md)
- [candidate-map.md](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/reflection/candidate-map.md)
- [demo/src](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src)
- [Agent 本地命令安全执行链路](../patterns/agent-command-safety-pipeline.md)

## 复习练习

不看 closeout，重新画三张图：工具权限链路、ReAct loop、CLI 分层。画完后对照 demo 代码检查每个节点是否真的存在。

## 过程摘要

这个 topic 从 Codex 权限/沙箱源码阅读出发，先抽取本地命令安全执行不变量，再实现 mini demo。实现过程中补齐了 approval、sandbox retry、事件可观察性、OpenAI-compatible streaming、并发 tool call、真实 sandbox-exec 和 ratatui CLI。最后通过用户 closeout 和候选表筛选进入 knowledge-base。
