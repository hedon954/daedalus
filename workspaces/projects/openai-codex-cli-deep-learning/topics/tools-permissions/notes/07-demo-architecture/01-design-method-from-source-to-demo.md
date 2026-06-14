# 从源码学习到可迁移设计的方法

这份笔记沉淀本次 Codex mini demo 设计过程中的学习方式。它不是 Codex 机制总结，而是一套可以复用到其他 repo learning 的设计方法。

## 核心心法

不要从“源码里有什么”开始设计，而要从“现实系统必须保证什么”开始。源码是成熟样本，不是设计起点。

这次我们一开始如果贴着 Codex enum 和源码分支设计，很容易做成“复刻 Codex 某段权限逻辑”。真正有效的转折点是退回需求侧：

```text
Agent/CLI 如何安全地执行本地命令？
```

然后再问 Codex 为什么需要 approval、sandbox、retry、event stream 这些机制。

## 方法模板

```text
Reality Problem
  -> Required Questions
  -> System Modules
  -> Domain Invariants
  -> Minimal Data Model
  -> Execution State Machine
  -> Observable Events
  -> Acceptance Tests
  -> Build Slices
```

## 1. Reality Problem

先用一句话定义现实问题。

本次问题：

```text
一个 Agent/CLI 要安全地执行本地命令，不能只靠命令字符串黑白名单，
而要把能力声明、执行上下文、权限判断、沙箱执行、失败重试和事件可观测性串成闭环。
```

这一步的作用是防止学习目标退化成“读懂某几个源码文件”。

## 2. Required Questions

把现实问题拆成系统必须回答的问题。

本次用户提出的四个问题成为设计主轴：

```text
1. 如何获取支持的命令能力？
2. 命令的执行路径是什么？
3. 命令的权限判断过程是什么？
4. 命令执行过程中的事件如何向外暴露？
```

这些问题比“Codex 里有哪些 struct”更稳定，因为它们来自需求侧。

## 3. System Modules

把问题映射成模块，而不是直接映射成源码文件。

```text
支持哪些命令能力 -> Capability Registry
执行路径是什么 -> Agent Orchestrator / SandboxRunner
权限如何判断 -> Approval Decision
事件如何暴露 -> Event Protocol
```

Codex 的源码只用于校准这些模块的生产级实现方式。

## 4. Domain Invariants

先定义领域不变量，再定义代码结构。

本次最重要的不变量：

```text
Allow 不等于 bypass sandbox。
CommandFailed 不等于 SandboxDenied。
Approval 不是只批准 command，而是批准 command + cwd + sandbox + network + persistence 的 scope。
Event 名表达阶段，字段表达状态。
Forbidden 是一等分支，不能退化成永远问用户。
```

这些不变量是设计的骨架。结构体只是承载它们。

## 5. Minimal Data Model

数据模型要刚好支撑核心不变量，不要搬运完整生产系统。

本次保留：

```text
CommandRequest.raw_command
CommandRequest.argv
CommandRequest.cwd
CommandRequest.capability
CommandRequest.approval_policy
CommandRequest.sandbox_profile
CommandRequest.network_policy
CommandRequest.justification
```

本次暂不保留：

```text
env
hook_command
process_id
tty
timeout_ms
additional_permissions_preapproved
```

判断标准：

```text
这个字段是否会改变当前 demo 要证明的核心命题？
```

如果不会，先放入 non-goal 或 Phase 2。

## 6. Execution State Machine

状态机要回答“系统如何流动”，尤其要标出提前结束和 retry 的边界。

本次关键边界：

```text
Forbidden -> 策略终止，不能交给模型绕路。
NeedsApproval + Rejected -> 策略终止。
CommandFailed -> 作为 observation 回灌给模型，允许模型自我修复。
SandboxDenied -> 进入 retry gate。
NoSandboxRetry -> 单个 tool call 最多一次。
Agent loop -> 必须有 max_turns。
```

这一步让 demo 接近真实 ReAct loop，而不是只有一个工具函数测试。

## 7. Observable Events

事件协议不要按成功/失败拆成大量事件名，而要按阶段拆事件名，用领域枚举表达状态。

本次设计原则：

```text
事件名表达阶段。
事件字段表达结果、权限、沙箱、retry 和失败类型。
领域枚举是 source of truth。
Event 是某个时间点上的领域状态快照。
```

所以不设计：

```text
ToolDenied
CommandNeedsApproval
ToolSkip
ToolRunSuccess
ToolRunFailed
```

而是设计：

```text
ToolApprovalResolved { requirement, user_decision }
CommandExecutionStarted { attempt }
CommandExecutionFinished { result }
RetryEvaluated { retry_decision }
```

这样未来增加失败类型时扩展 `ExecutionFailure`，而不是扩展事件名。

## 8. Acceptance Tests

验收测试不是补充，而是设计边界线。

测试要证明不变量，而不是只证明功能跑通。

本次 AT-01 到 AT-14 覆盖：

```text
capability 加载
Skip 但不 bypass sandbox
dangerous-shell forbidden
approval rejected 不执行
approval scope 绑定上下文
CommandFailed 回灌给模型
SandboxDenied 进入 retry gate
retry approval 后 NoSandboxRetry
session approval 复用与 scope mismatch
max_turns 防无限循环
```

这些测试反过来固定了 demo 的最小实现范围。

## 9. Build Slices

复杂度要分阶段出场。

本次先拆 Phase：

```text
Phase 1: SimulatedSandboxRunner
Phase 2: OsSandboxRunner
```

再拆 08 阶段编码 slice：

```text
Project Skeleton
Domain Models
Capability Registry
Approval Decision
SimulatedSandboxRunner
Retry Gate
Agent Orchestrator
README / Runbook
```

这种拆法的关键是：

```text
Phase 1 保证学习闭环。
Phase 2 保证 demo 最终能走向可用。
Phase 2 只替换 runner，不重写 approval / retry / event 协议。
```

补充：`assert`、`Result` 和安全拒绝的边界判断单独沉淀在 [`../08-demo-coder/14-assert-result-denied-boundary.md`](../08-demo-coder/14-assert-result-denied-boundary.md)。

## 复用提问清单

以后读一个优秀项目并准备做 mini demo 时，可以按这个顺序问：

```text
1. 这个项目面对的现实压力是什么？
2. naive 实现会坏在哪里？
3. 它为了防止坏掉，引入了哪些核心概念？
4. 哪些概念是领域不变量，哪些只是工程实现细节？
5. 最小 demo 要证明哪个系统命题？
6. 哪些字段足以承载这个命题，哪些字段应该暂缓？
7. 状态机里哪些分支是提前终止，哪些分支可以回到 loop？
8. 事件应该表达阶段，还是表达结果？
9. 哪些验收测试能证明这个 demo 真的保留了核心不变量？
10. 哪些复杂度应该放到 Phase 2，而不是阻塞 Phase 1？
```

## 本次学习方式的结论

源码学习的终点不是“我知道 Codex 怎么写”，而是：

```text
我能从 Codex 的生产实现里抽出一个现实问题的核心不变量，
再把这些不变量压缩成自己的可运行 mini demo。
```

这就是 repo learning 从知识输入走向能力迁移的关键。
