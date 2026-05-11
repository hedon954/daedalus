# Codex 核心代码阅读记录

本文件是 `06-code-reader` 阶段的标准入口。所有核心代码阅读都按以下结构记录：

```text
生产问题 -> naive 失败 -> 源码应对 -> 保护的不变量 -> trade-off -> 可迁移模式
```

## 1. Auth / Approval / Sandbox：本地执行如何避免失控

### 1.1 生产问题

工业级 Agent CLI 不是只“调用工具”，而是在用户本机执行有副作用的操作。真实风险是：

- 执行用户没有授权的命令。
- 修改或删除授权边界之外的文件。
- 触碰没有 Git 或备份兜底的重要目录、系统文件、密钥或隐私数据。
- 在用户没有充分理解副作用时，把模型生成的命令当成可信操作直接执行。

核心问题：

> Codex 如何避免本地 Agent 变成不受控的远程执行器？

### 1.2 用户当前假设

用户认为最危险的是执行了不该执行的命令，尤其是操作授权范围之外的文件夹。工作区内误删通常还能靠 Git 恢复；工作区外的目录可能没有版本控制，甚至可能包含重要系统文件，因此风险更高。

用户对权限分层的初始假设：

1. 用户明确禁止的命令应该最先拦截。
2. `rm -rf` 等高度危险命令任何时候都需要 double check，可以用黑名单机制加小模型判断危险程度。
3. 用户白名单命令可以直接执行。
4. 无害的简单读命令可以直接执行。
5. 不明确的情况寻求用户同意。

用户也意识到：这个分层更像“权限强弱分层”，不一定是源码实现里的“责任边界分层”。

### 1.3 Agent 校准

用户对核心风险的判断成立。真正危险的不是命令执行失败，而是 Agent 越过用户理解和授权的边界，对不可恢复或未授权资源产生副作用。

需要修正的是分层方式：Codex 源码不是单纯按“危险等级”组织，而是按“实现责任边界”组织。

风险等级分层类似：

```text
最危险 -> 次危险 -> 白名单 -> 读命令 -> 问用户
```

Codex 的实现边界更接近：

```text
这个阶段能知道什么信息？
这个阶段能做什么决定？
这个阶段失败后谁来兜底？
```

### 1.4 已读源码入口

- `core/src/tools/registry.rs`
- `core/src/tools/sandboxing.rs`
- `core/src/tools/orchestrator.rs`

### 1.5 源码中的责任边界

#### ToolRegistry：结构性门禁

`ToolRegistry::dispatch_any` 先处理工具分发前的结构性问题：

- 工具是否存在。
- payload 是否和工具类型匹配。
- 是否触发 `pre_tool_use_hooks`。
- 工具是否 mutating。
- mutating tool 是否需要等待 `tool_call_gate`。
- 成功后是否触发 `post_tool_use_hooks`。

这一层不是审批中心，而是工具调用能否被正确、安全地分发的第一道边界。

#### ToolRuntime / Approvable：请求级审批需求

`ExecApprovalRequirement` 表达具体请求的审批需求：

- `Skip`：本次不需要审批。
- `NeedsApproval`：本次需要用户、hook 或 guardian 审批。
- `Forbidden`：本次直接禁止执行。

这个抽象比“黑名单”更灵活，因为不同 tool 可以根据自己的请求、policy、sandbox 权限和上下文决定审批要求。

#### ToolOrchestrator：审批、sandbox 和重试编排

`ToolOrchestrator` 是中心编排层，主流程是：

```text
approval
-> select sandbox
-> first attempt
-> sandbox denial
-> decide whether retry without sandbox needs approval
-> second attempt
```

它集中处理：

- `run_permission_request_hooks`
- guardian / user approval
- sandbox selection
- network approval
- sandbox denied 后是否允许 no-sandbox retry

#### SandboxAttempt：策略落地为执行环境

`SandboxAttempt` 把策略转换成真实执行环境：

- sandbox 类型
- permission profile
- cwd
- managed network
- Linux / Windows sandbox 参数
- network denial cancellation token

这一层的职责不是判断用户意图，而是把已经决定的权限策略变成 OS/sandbox 可执行的约束。

### 1.6 当前可验证结论

Codex 的权限机制不是一个单点“危险命令分类器”，而是多层防线：

```text
工具存在性 / payload 合法性
-> tool 自己声明审批需求
-> hooks / guardian / user approval
-> sandbox first attempt
-> sandbox denial
-> 是否允许审批后 no-sandbox retry
```

这解释了为什么“最严格放最前面”只对一部分问题成立：

- 能在前置阶段确定的禁止项，应尽早拒绝。
- 但是否越权访问文件系统，往往需要 sandbox 执行时才能真实验证。
- 是否需要 no-sandbox retry，必须等 sandbox denial 后才知道。
- 是否允许 retry，必须重新走 approval / guardian / user decision。

### 1.7 保护的不变量

- Agent 不能绕过用户授权边界直接裸跑高副作用命令。
- 工具调用必须先通过结构性门禁，再进入具体 runtime。
- sandbox failure 不能自动升级为无沙箱执行。
- no-sandbox retry 必须受到 approval policy、hook、guardian 或用户决策约束。
- 工具副作用需要被 hook、telemetry、dispatch trace 和 turn state 记录。

### 1.8 设计代价

- 权限逻辑分散在 registry、runtime trait、orchestrator、sandbox attempt、hook、guardian、network approval 多处，读起来复杂。
- 代码不如单一“命令危险等级判断器”直观。
- 但这种复杂度换来的是边界清晰：结构门禁、审批判断、执行环境、失败升级和审计分别由不同层承担。

### 1.9 可迁移模式

如果实现自己的 Agent/CLI，可以迁移这个分层：

```text
Tool Registry：工具存在性、payload 类型、hook、mutating 标记
Tool Runtime：请求级审批需求
Orchestrator：统一审批、sandbox、retry 编排
Sandbox Attempt：把策略落到真实执行环境
Audit/Trace：记录每次决策、失败和升级
```

不建议一开始照抄 Codex 的全部复杂度；可以先实现：

- `Allowed / NeedsApproval / Forbidden`
- 工作区根目录边界
- sandbox first attempt
- denied 后必须重新审批才能提升权限
- 每次执行记录审计日志

### 1.10 下一步源码验证问题

1. `ExecApprovalRequirement::Forbidden`、`NeedsApproval`、`Skip` 分别在保护什么生产风险？
2. 为什么 Codex 先 sandbox 执行，失败后才考虑 no-sandbox retry，而不是一开始就让用户批准后直接裸跑？
3. `ToolRegistry::dispatch_any` 里的 `pre_tool_use_hooks`、`is_mutating`、`tool_call_gate` 分别在防什么事故？

### 1.11 `ExecApprovalRequirement` 保护的生产风险

源码位置：

- `core/src/tools/sandboxing.rs`
- `core/src/tools/orchestrator.rs`

`ExecApprovalRequirement` 有三个结果：

```rust
pub(crate) enum ExecApprovalRequirement {
    Skip { bypass_sandbox: bool, ... },
    NeedsApproval { reason: Option<String>, ... },
    Forbidden { reason: String },
}
```

它不是“命令危险等级枚举”，而是告诉 `ToolOrchestrator` 对这一次 tool call 应该采取什么审批策略。

- `Forbidden`：保护“策略上不允许继续”的边界。比如 granular approval policy 不允许 sandbox approval prompt 时，源码会直接返回 `Forbidden`，避免系统假装可以问用户或绕过策略。
- `NeedsApproval`：保护“用户/自动审查必须知情”的边界。它不判断命令最终一定危险，而是表达当前 policy 和 sandbox 状态下不能自动执行。
- `Skip`：保护“低风险或已被策略允许的操作不被过度打扰”的体验边界。即使 `Skip`，也不等于一定裸跑；默认 `bypass_sandbox` 是 `false`，仍会进入 sandbox first attempt。

关键校准：

> `Skip` 是“跳过审批”，不是“跳过沙箱”。是否绕过 sandbox 还要看 `bypass_sandbox` 和 `sandbox_mode_for_first_attempt`。

### 1.12 为什么先 sandbox，再考虑 no-sandbox retry

源码位置：

- `core/src/tools/orchestrator.rs`
- `core/src/tools/sandboxing.rs`

`ToolOrchestrator` 的顺序是：

```text
approval requirement
-> request approval if needed
-> select initial sandbox
-> first attempt
-> if SandboxErr::Denied
-> check escalation policy
-> request approval for retry if needed
-> second attempt with SandboxType::None
```

这个顺序解决的是生产环境里的“最小权限执行”问题。

如果一开始就让用户批准后裸跑，会有两个问题：

1. 用户被迫为本来可以在 sandbox 内安全完成的命令承担无沙箱风险。
2. 系统无法区分“命令本身失败”和“命令因为 sandbox 权限不够失败”。

Codex 选择 sandbox-first，收益是：

- 尽量让命令在受限环境中完成。
- 只有 sandbox denial 后，才把“是否提升权限”变成明确问题。
- no-sandbox retry 不是自动发生，仍受 `AskForApproval`、`wants_no_sandbox_approval`、guardian、hook、用户决策约束。

这里保护的不变量是：

> sandbox failure 不能自动升级为裸跑；权限升级必须重新经过显式策略或审批。

### 1.13 `dispatch_any` 的结构门禁

源码位置：

- `core/src/tools/registry.rs`

`ToolRegistry::dispatch_any` 不是审批中心，而是工具调用进入 runtime 前后的结构门禁和审计点。

它做了几类事情：

1. 工具存在性检查：找不到 handler 时返回可回灌给模型的错误，而不是执行未知工具。
2. payload 类型检查：工具名和 payload 类型不匹配时 fatal，避免模型输出的结构错误被误执行。
3. `pre_tool_use_hooks`：给外部策略一个执行前拦截点，可以把结果作为 tool response 回给模型。
4. `is_mutating` + `tool_call_gate`：对可能修改环境的工具等待 gate release，避免有副作用工具在不合适的时机并发推进。
5. telemetry / dispatch trace / post hook：记录工具结果，并允许 post hook 添加上下文或替换反馈。

这些机制防的不是单一危险命令，而是更底层的工程事故：

- 模型调用不存在的工具。
- payload 结构不匹配导致错误工具被执行。
- 外部组织策略无法在执行前拦截。
- 多个 mutating tool 并发破坏工作区状态。
- 工具执行后缺少审计、补充上下文或停止信号。

### 1.14 本轮验证结论

Codex 的权限/沙箱设计可以浓缩成一句话：

> 它不把安全性押在一次“命令危险判断”上，而是把工具调用拆成结构门禁、请求级审批、sandbox-first 执行、失败后受控升级和审计回流多个责任边界。

这对自己的 Agent/CLI 项目的启发是：

- 第一版不要只做黑名单/白名单。
- 至少要区分 `Forbidden / NeedsApproval / SkipApproval`。
- `SkipApproval` 不应等于 `NoSandbox`。
- sandbox denial 后的 retry 必须是一个新的审批事件。
- mutating tool 需要串行化或 gate，不能和 read-only tool 用同一并发策略。

验证状态：已验证第一轮源码边界；下一步需要继续读具体 shell/unified exec runtime 如何实现审批键、命令描述、sandbox cwd 和权限 profile。
