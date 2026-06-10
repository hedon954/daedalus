---
kind = "pattern"
slug = "local-agent-command-execution"
status = "stable"
source = "workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions"
created_at = "2026-06-10 03:27:21"
---

# Local Agent Command Execution Safety Pattern

## 回忆钩子

当一个 Agent 需要执行本地命令时，不要问“这个命令字符串能不能跑”，而要问“这个意图在当前上下文、权限画像、沙箱边界和审批策略下，应该走哪条执行路径”。

## 现实问题

本地 coding agent 会读写文件、访问网络、安装依赖和启动进程。单纯黑白名单无法表达 cwd、文件权限、网络权限、审批策略、沙箱边界和 retry 语义。

## 第一性原理

模型输出只是意图，不是权限。权限必须由宿主环境根据上下文重新判定，并把执行拆成可观察、可中断、可审批、可重试的状态机。

## 机制模型

```text
model intent
  -> host capability
  -> approval requirement
  -> sandboxed execution attempt
  -> retry / escalation decision
  -> observable tool result
```

模型只能提出 tool call；host runtime 负责把命令映射到 capability，并根据 request context 生成 approval / sandbox / retry 决策。

## 关键不变量

- 模型意图不等于执行权限。
- `Allow` 不等于跳过 sandbox。
- 用户审批必须绑定 command prefix、cwd、sandbox profile、network policy 和 persistence。
- sandbox denied 后不能自动裸跑，必须经过 retry gate。
- 未匹配 capability 的命令 fail closed。
- tool / command / approval / retry events 必须对外可观察。

## 取舍

- 结构更复杂，需要维护 capability registry、approval scope、execution runner 和 retry policy。
- 用户可能需要处理更多审批交互。
- prefix matching 易解释，但对复杂 shell 语法不够强。
- session approval 改善体验，但扩大一次点击的影响范围。

## 不要照搬

- 不要在早期 mini demo 里照搬完整跨平台 sandbox。
- 不要把 Codex 内部所有 event 类型原样搬过来。
- 不要过早实现完整 TUI/MCP elicitation。
- 不要把长期持久化 exec policy 当成第一版必需能力。

## 迁移方式

1. 定义最小 capability registry，只开放必要命令。
2. 定义 `CommandRequest`，把 cwd、sandbox、network、approval policy 放进上下文。
3. 定义 `ApprovalScope`，让一次允许绑定执行权限画像。
4. 把 runner 抽成 `ExecutionRunner`，先用 simulated runner 测状态机，再接 OS sandbox。
5. 把 sandbox denied 交给 retry gate，而不是在 runner 内直接提权。
6. 把安全节点转成稳定事件，供 UI、日志和测试消费。

## 证据来源

- Topic: `workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions`
- Demo: `workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/README.md`
- Business transfer: `workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/notes/09-biz-solver/README.md`
- Closeout note: `workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/notes/08-demo-coder/13-slice-13-codex-like-cli-closeout.md`
- Verification: `cargo test --manifest-path workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/Cargo.toml -j 2`

## 复习练习

1. 不看源码，画出 approval / sandbox / retry 的状态机。
2. 解释为什么 `NeedsApproval approved` 后仍应先走 sandbox。
3. 设计一个会触发 `SandboxDenied -> RetryDecision` 的命令案例。
4. 说明 session approval 为什么不能只绑定 command name。

## 反例

如果运行环境是无交互 CI，并且所有命令都由可信脚本生成，可以固定 `ApprovalPolicy::Never`，禁用 no-sandbox retry，把失败直接作为构建失败处理。
