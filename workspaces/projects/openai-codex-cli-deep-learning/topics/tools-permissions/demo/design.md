# Codex Mini Demo Design

> 状态：architecture blueprint / pending user review。这个文件已由 `06-code-reader` 的源码证据补齐关键缺口，并在 `07-demo-architecture` 中收敛为 Phase 1 / Phase 2 mini demo 蓝图。

## Demo North Star

实现一个最小 Agent CLI 的本地命令执行链路，保留 Codex 的核心权限/沙箱不变量：命令不是只按字符串判断，而是结合命令解析、工作目录、沙箱权限、额外权限、审批策略和失败类型来决定执行路径。

demo 的出发点不是复刻 Codex 的某个源码分支，而是从需求侧证明一个更上层的系统命题：

```text
一个安全的本地命令执行系统，必须先声明支持哪些命令能力，
再定义一次命令请求如何携带上下文，
再合成权限决策和沙箱执行路径，
最后把关键执行事件透明地暴露给用户或上层系统。
```

## Capability Scope

demo 先实现四类命令能力，覆盖从低风险到高风险的本地执行场景：

| Capability | 示例 | 设计意图 |
| --- | --- | --- |
| `safe-read` | `ls`, `cat package.json` | 读取工作区信息，验证低风险命令也要绑定 cwd 和 sandbox scope。 |
| `safe-test` | `npm test`, `cargo test` | 运行项目测试，验证常用开发命令可以默认沙箱执行但不等于无约束执行。 |
| `network-install` | `npm install` | 需要网络或依赖写入，验证网络/文件权限需要显式策略。 |
| `dangerous-shell` | `curl ... | sh`, `rm -rf` | 高风险或不可安全抽象的命令，验证系统必须能拒绝而不是只提示。 |

## Capability Registry Model

`CapabilityKind` 只是能力类型标签。为了让系统知道“支持哪些命令能力，以及这些能力默认如何安全执行”，demo 需要一个 capability registry。registry 中的每条记录是一个 `CapabilityDescriptor`。

```text
CapabilityDescriptor
  - kind: CapabilityKind
  - name: String
  - description: String
  - command_prefixes: Vec<Vec<String>>
  - policy: CapabilityPolicy
```

字段含义：

- `kind`：能力类型，例如 `SafeRead`、`SafeTest`。
- `name`：面向事件、日志、README 的稳定名字，例如 `safe-read`。
- `description`：说明这个能力解决什么执行场景。
- `command_prefixes`：哪些命令前缀可以匹配到这个能力，例如 `["cargo", "test"]`、`["npm", "test"]`。
- `policy`：这个能力的默认安全策略。

`CapabilityPolicy` 描述匹配到该能力后，默认应该如何进入权限判断和执行链路。

```text
CapabilityPolicy
  - default_decision: DefaultDecision
  - first_attempt_sandbox: SandboxProfile
  - network_policy: NetworkPolicy
  - retry_policy: RetryPolicy

DefaultDecision
  - Allow
  - Prompt
  - Forbidden

RetryPolicy
  - Never
  - WithApproval
  - WithoutApproval
```

设计含义：

- `CapabilityKind` 回答“这是什么能力”。
- `CapabilityDescriptor` 回答“系统注册了什么能力，以及哪些命令能匹配它”。
- `CapabilityPolicy` 回答“这种能力默认怎么安全执行”。
- `DefaultDecision::Allow` 只表示默认不需要先问用户，不表示可以跳过 sandbox。
- `RetryPolicy` 只在 sandbox denied 后参与 retry gate，不影响普通命令失败。

## Default Capability Policy

每类 capability 都必须声明默认权限策略。`Allow` 只表示“不需要先问用户”，不表示可以跳过 sandbox。

| Capability | 默认决策 | 首次 sandbox | 网络 | sandbox denied 后 retry |
| --- | --- | --- | --- | --- |
| `safe-read` | `Allow` | 必须 sandbox | 禁止 | 不 retry |
| `safe-test` | `Allow` | 必须 sandbox | 禁止 | 可请求批准 retry |
| `network-install` | `Prompt` | 必须 sandbox | 需要批准 | 可请求批准 retry |
| `dangerous-shell` | `Forbidden` | 不执行 | 不允许 | 不 retry |

设计含义：

- 低风险命令也要进入 sandbox，因为“无需审批”不等于“无约束执行”。
- 网络权限独立于命令字符串，必须由 capability/policy 明确声明。
- retry 是独立决策；第一次 allow 或 prompt 通过，不自动代表后续可以无沙箱执行。
- `dangerous-shell` 用直接拒绝证明系统有 forbidden 分支，而不是把所有风险都转成审批弹窗。

## Command Request Context

一次本地命令执行请求不能只是一段命令字符串。demo 中一次请求至少要携带“命令是什么、属于什么能力、站在哪个目录、带着什么审批和沙箱策略执行”。

```text
CommandRequest
  - raw_command: String
  - argv: Vec<String>
  - cwd: Path
  - capability: CapabilityKind
  - approval_policy: ApprovalPolicy
  - sandbox_profile: SandboxProfile
  - network_policy: NetworkPolicy
  - justification: Option<String>
```

核心枚举：

```text
CapabilityKind
  - SafeRead
  - SafeTest
  - NetworkInstall
  - DangerousShell

ApprovalPolicy
  - Never
  - OnFailure
  - OnRequest

SandboxProfile
  - ReadOnly
  - WorkspaceWrite
  - NoSandbox

NetworkPolicy
  - Deny
  - Prompt
  - Allow
```

设计边界：

- `cwd` 和 `sandbox_profile` 必须同时存在：前者回答“命令站在哪里执行”，后者回答“命令被允许碰到哪里”。
- demo 暂不实现 `env`、`hook_command`、`process_id`、`tty`、`timeout_ms`、`additional_permissions_preapproved`；这些属于 Codex runtime 完整性，不是当前 demo 的核心证明点。

## Approval Decision Model

权限判断不能输出简单布尔值。demo 必须区分“直接禁止”“需要审批”“无需审批但仍进 sandbox”“无需审批且可跳过 sandbox”。

```text
ApprovalRequirement
  - Skip {
      bypass_sandbox: bool,
      reason: String
    }

  - NeedsApproval {
      reason: String,
      approval_scope: ApprovalScope
    }

  - Forbidden {
      reason: String
    }
```

用户批准必须绑定到可解释的 scope，不能只绑定命令名或字符串前缀。

```text
ApprovalScope
  - command_prefix: Vec<String>
  - cwd: Path
  - sandbox_profile: SandboxProfile
  - network_policy: NetworkPolicy
  - persistence: ApprovalPersistence

ApprovalPersistence
  - Once
  - Session
```

权限判断接口：

```text
decide_approval(
  request: CommandRequest,
  capability_policy: CapabilityPolicy
) -> ApprovalRequirement
```

设计含义：

- `Skip { bypass_sandbox: false }` 表示无需先问用户，但首次仍必须在 sandbox 内执行。
- `Skip { bypass_sandbox: true }` 只用于已被策略明确允许的高信任 scope。
- `NeedsApproval` 必须携带 `ApprovalScope`，让用户知道批准的是哪条命令前缀、哪个 cwd、哪种沙箱和网络权限，以及是单次还是 session 生效。
- `Forbidden` 是一等分支，不能退化成“永远问用户一次试试”。

## Execution Runner Strategy

demo 分两阶段实现 execution runner，避免第一阶段被 OS 细节拖垮，同时保留走向真实可用 demo 的接口。`ExecutionRunner` 不是“永远在 sandbox 里跑”的 runner，而是根据 `ExecutionAttempt` 分发 sandbox-first、no-sandbox-first 和 no-sandbox-retry。

```text
ExecutionRunner
  - run(request, attempt) -> ExecutionResult
```

### Phase 1: SimulatedExecutionRunner

第一阶段使用模拟 sandbox，把完整执行链路跑通。它不做真实 OS 隔离，但必须根据 request 和 capability 生成可解释的 allow / denied 结果。

```text
SimulatedExecutionRunner
  - 根据 cwd 判断是否在 workspace 内
  - 根据 sandbox_profile 判断读写是否越界
  - 根据 network_policy 判断网络是否允许
  - 根据 capability 生成模拟执行结果
```

模拟结果必须覆盖：

```text
ExecutionResult
  - Success { stdout }
  - Failure(CommandFailed { exit_code, stderr })
  - Failure(SandboxDenied { output, network_context })
```

这个阶段的目标不是证明 OS 隔离能力，而是证明：

- `cwd`、`sandbox_profile`、`network_policy` 会改变同一条命令的执行路径。
- sandbox denied 是可解释的状态，不是普通命令失败。
- sandbox denied 后必须经过 retry gate，不能自动裸跑。
- event stream 能把 allow、prompt、denied、retry、completed 暴露给用户。

### Phase 2: OsExecutionRunner

第二阶段在同一个 `ExecutionRunner` 接口下接入真实 OS sandbox，让 demo 从“可解释模型”变成“可用 mini demo”。

```text
OsExecutionRunner
  - 复用 CommandRequest / ApprovalRequirement / RetryDecision
  - 首轮使用真实 sandbox 执行命令
  - 保留 sandbox denied -> retry gate -> no-sandbox second attempt 的编排逻辑
  - 将平台相关错误映射回统一 ExecutionFailure
```

Phase 2 的设计边界：

- 可以先只支持当前开发平台，不追求跨平台完整性。
- OS sandbox 只替换 runner，不重写 approval 和 retry 状态机。
- 如果真实 sandbox 无法提供精确 denied reason，也要映射成可解释的 `SandboxDenied` 事件。
- 网络控制可以先保持模拟或 coarse-grained，不把 host-level policy 作为第一版硬目标。

## Event Protocol

事件协议对标最小 Agent ReAct loop。事件名表达“系统走到哪个阶段”，成功/失败、是否沙箱、是否重试等信息放在事件字段里。领域枚举是 source of truth，event 只是某个时间点上的领域状态快照。

```text
AgentEvent
  - AgentStarted {
      session_id,
      cwd
    }

  - ToolRegistryLoaded {
      capabilities: Vec<CapabilityDescriptor>
    }

  - UserQueryReceived {
      query: String
    }

  - ModelStarted {
      turn_id
    }

  - ModelFinished {
      turn_id,
      message
    }

  - ToolChosen {
      turn_id,
      tool_call_id,
      request: CommandRequest
    }

  - ToolApprovalStarted {
      tool_call_id,
      request: CommandRequest
    }

  - ToolApprovalResolved {
      tool_call_id,
      requirement: ApprovalRequirement,
      user_decision: Option<UserApprovalDecision>
    }

  - CommandExecutionStarted {
      tool_call_id,
      request: CommandRequest,
      attempt: ExecutionAttempt
    }

  - CommandExecutionFinished {
      tool_call_id,
      request: CommandRequest,
      attempt: ExecutionAttempt,
      result: ExecutionResult
    }

  - RetryEvaluated {
      tool_call_id,
      retry_decision: RetryDecision
    }

  - FinalOutputEmitted {
      message
    }

  - AgentFinished {
      status: AgentStatus
    }
```

事件复用的领域模型：

| 领域模型 | 被哪些 event 复用 |
| --- | --- |
| `CommandRequest` | `ToolChosen`、`ToolApprovalStarted`、`CommandExecutionStarted`、`CommandExecutionFinished` |
| `ApprovalRequirement` | `ToolApprovalResolved` |
| `ApprovalScope` / `ApprovalPersistence` | `ApprovalRequirement::NeedsApproval`、`UserApprovalDecision::Approved` |
| `CapabilityKind` | `CommandRequest`、`CapabilityDescriptor` |
| `SandboxProfile` | `CommandRequest`、`ExecutionAttempt` |
| `NetworkPolicy` | `CommandRequest`、`ApprovalScope` |
| `ExecutionResult` / `ExecutionFailure` | `CommandExecutionFinished` |
| `RetryDecision` | `RetryEvaluated` |

事件专用薄枚举：

```text
UserApprovalDecision
  - Approved { persistence: ApprovalPersistence }
  - Rejected

ExecutionAttempt
  - SandboxFirst { sandbox_profile: SandboxProfile }
  - NoSandboxRetry { reason: String }

AgentStatus
  - Success
  - Failed
  - Cancelled
```

最小 ReAct loop 事件时间线：

```text
AgentStarted
ToolRegistryLoaded
UserQueryReceived
ModelStarted
ModelFinished
ToolChosen
ToolApprovalStarted
ToolApprovalResolved
CommandExecutionStarted
CommandExecutionFinished
RetryEvaluated?
CommandExecutionStarted?      // no-sandbox retry
CommandExecutionFinished?     // retry result
ModelStarted
ModelFinished
FinalOutputEmitted
AgentFinished
```

设计边界：

- 不拆 `ToolDenied`、`CommandNeedsApproval`、`ToolSkip` 三个事件；它们都是 `ToolApprovalResolved.requirement` 的不同值。
- 不把 `success/failed/in_sandbox/is_retried` 做成事件名；它们分别属于 `ExecutionResult` 和 `ExecutionAttempt`。
- 以后增加新的失败类型时，优先扩展 `ExecutionFailure`，不增加事件名。

## Main Execution State Machine

demo 的主状态机对标一个最小 ReAct loop：用户输入 query，模型选择工具，系统检查权限并执行工具，工具结果回灌给模型，直到模型不再选择工具并输出最终答复。

```text
AgentStarted
  -> ToolRegistryLoaded
  -> UserQueryReceived
  -> ModelStarted
  -> ModelFinished

ModelFinished
  -> no tool call:
      FinalOutputEmitted
      AgentFinished(Success)
  -> tool call:
      ToolChosen
      ToolApprovalStarted
      ToolApprovalResolved

ToolApprovalResolved
  -> Forbidden:
      AgentFinished(Failed)
  -> NeedsApproval + Rejected:
      AgentFinished(Failed)
  -> NeedsApproval + Approved:
      CommandExecutionStarted(SandboxFirst)
  -> Skip { bypass_sandbox: false }:
      CommandExecutionStarted(SandboxFirst)
  -> Skip { bypass_sandbox: true }:
      CommandExecutionStarted(NoSandboxFirst)

CommandExecutionFinished(first attempt)
  -> Success:
      append tool result as observation
      next loop: ModelStarted
  -> CommandFailed:
      append command failure as observation
      next loop: ModelStarted
  -> SandboxDenied:
      RetryEvaluated

RetryEvaluated
  -> DoNotRetry:
      append sandbox denied as observation
      next loop: ModelStarted
  -> RetryWithoutApproval:
      CommandExecutionStarted(NoSandboxRetry)
  -> RetryWithApproval:
      CommandRetryApprovalStarted
      CommandRetryApprovalResolved
        -> Rejected:
            append retry rejected as observation
            next loop: ModelStarted
        -> Approved:
            CommandExecutionStarted(NoSandboxRetry)

CommandExecutionFinished(retry attempt)
  -> Success:
      append tool result as observation
      next loop: ModelStarted
  -> CommandFailed:
      append command failure as observation
      next loop: ModelStarted
  -> SandboxDenied:
      append retry sandbox denied as observation
      next loop: ModelStarted
```

loop 边界：

- `CommandFailed` 不是 sandbox denial，也不触发 no-sandbox retry；它作为 observation 回灌给模型，让模型有机会修复命令。
- `SandboxDenied` 才进入 retry gate。
- 单个 tool call 最多一次 sandbox first attempt 和一次 no-sandbox retry attempt。
- agent loop 必须有 `max_turns`，避免模型持续修复命令导致无限循环。
- `Forbidden` 和首次审批拒绝是策略终止；它们不再交给模型绕路重试。

## Core Invariants

- 命令风险判断不能只看原始命令字符串；必须携带执行上下文。
- `Skip` 不等于跳过 sandbox；只有 `Skip { bypass_sandbox: true }` 或显式 require escalated 才能首轮不进 sandbox。
- sandbox 失败后不能自动裸跑；必须根据失败类型、审批策略和工具配置决定是否请求提升或直接结束。
- 用户的一次允许必须绑定到可解释粒度，例如规范化命令、cwd、sandbox permissions、additional permissions，以及是否只本次有效或 session 有效。

## Implementation Data Model Summary

08 阶段实现时以本设计中已经定稿的模型为准：

```text
CapabilityDescriptor
CapabilityPolicy
CommandRequest
ApprovalRequirement
ApprovalScope
ApprovalPersistence
UserApprovalDecision
ExecutionRunner
ExecutionAttempt
ExecutionResult
ExecutionFailure
RetryDecision
AgentEvent
AgentStatus
```

不再实现早期草案里的完整 Codex runtime 字段，例如 `hook_command`、完整 `env`、`process_id`、`tty`、`additional_permissions_preapproved`。这些字段属于 Codex 生产 runtime 的完整性，不属于 Phase 1 mini demo 的验收目标。

## Acceptance Test Plan

验收测试要证明的不是“跑了几条命令”，而是 demo 是否保留了安全本地命令执行系统的核心不变量。

| ID | 场景 | Given | When | Then |
| --- | --- | --- | --- | --- |
| AT-01 | 工具能力加载 | 内置四类 capability | agent 启动 | 事件包含 `ToolRegistryLoaded`，列出 `safe-read`、`safe-test`、`network-install`、`dangerous-shell`。 |
| AT-02 | `safe-read` 默认跳过审批但不跳过 sandbox | `safe-read` 请求，cwd 在 workspace 内，`ReadOnly` sandbox | 执行 `cat package.json` | `ToolApprovalResolved` 为 `Skip { bypass_sandbox: false }`，首次 attempt 是 `SandboxFirst`，执行成功。 |
| AT-03 | `dangerous-shell` 直接禁止 | `dangerous-shell` 请求，例如 `curl ... \| sh` 或 `rm -rf` | agent 检查权限 | `ApprovalRequirement::Forbidden`，没有 `CommandExecutionStarted`，agent 以 failed 结束。 |
| AT-04 | `network-install` 审批被拒绝时不执行 | `network-install` 请求，用户拒绝 approval | agent 检查权限 | 事件包含 `NeedsApproval` 与 `Rejected`，没有任何执行 attempt。 |
| AT-05 | approval scope 绑定上下文 | `network-install` 请求，用户批准 session 级授权 | agent 生成 approval | `ApprovalScope` 同时包含 `command_prefix`、`cwd`、`sandbox_profile`、`network_policy`、`persistence`。 |
| AT-06 | 普通命令失败回灌给模型 | `safe-test` 请求，sandbox runner 返回 `CommandFailed` | agent 执行工具 | 不进入 retry gate；失败作为 observation 回灌，触发下一轮 `ModelStarted`。 |
| AT-07 | 模型可基于失败自我修复 | scripted model 第一轮选择失败的 `safe-test`，第二轮选择修正命令或输出解释 | agent loop 运行 | 事件中出现至少两轮 model turn，最终输出包含修复结果或失败解释。 |
| AT-08 | sandbox denied 与命令失败区分 | runner 返回 `SandboxDenied` 而不是 `CommandFailed` | agent 执行工具 | 进入 `RetryEvaluated`，不把它当普通命令失败处理。 |
| AT-09 | 不允许 retry 时回灌 denied | `safe-read` 越界访问 workspace 外路径 | runner 返回 `SandboxDenied` | `RetryDecision::DoNotRetry`，denied 作为 observation 回灌给模型。 |
| AT-10 | retry 需要审批且被拒绝 | `safe-test` sandbox denied，policy 允许请求 retry，用户拒绝 | agent 评估 retry | 事件包含 `RetryEvaluated::RetryWithApproval` 和 rejected，不启动 `NoSandboxRetry`。 |
| AT-11 | retry 审批通过后无沙箱再试 | `safe-test` sandbox denied，用户批准 retry | agent 评估 retry | 第二次 attempt 是 `NoSandboxRetry`，最多只 retry 一次，结果回灌给模型。 |
| AT-12 | session 授权可减少重复审批 | 已有匹配 `ApprovalScope` 的 session 授权 | 再次执行相同 command prefix、cwd、sandbox、network scope | 不再次请求普通 approval；仍然按策略进入 sandbox first attempt。 |
| AT-13 | scope 不匹配不能复用授权 | session 授权的 cwd 或 network policy 与本次请求不同 | 再次执行类似命令 | 不能复用旧 approval，必须重新进入 approval path。 |
| AT-14 | max turns 防止无限修复 | scripted model 持续选择失败命令 | agent loop 运行 | 达到 `max_turns` 后发出 `AgentFinished(Failed)`，避免无限循环。 |

Phase 1 最小必过集合：

```text
AT-01, AT-02, AT-03, AT-04, AT-06, AT-08, AT-09, AT-11, AT-14
```

Phase 1 完整必过集合：

```text
AT-01 ... AT-14
```

## Explicit Non-Goals

- 不复刻完整 TUI。
- 不实现真实 LLM provider。
- 不覆盖所有平台 sandbox 细节。
- 不实现完整 MCP elicitation。
- 不把所有 shell 语法解析做成生产级解析器。
