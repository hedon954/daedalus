# Codex 核心代码阅读记录

本文件是 `06-code-reader` 阶段的标准入口。记录原则：

```text
生产问题 -> 用户假设 -> 源码证据 -> 校准结论 -> 保护的不变量 -> trade-off -> 可迁移模式
```

注意：

- `notes/` 只记录用户已经回答、复述、校正或验证过的学习轨迹。
- Agent 的源码结论必须有源码、测试、注释或运行观察作为证据锚点。
- 工程直觉如果尚未核对源码，只能写成假设。

## 1. Auth / Approval / Sandbox：本地执行如何避免失控

### 1.1 生产问题

工业级 Agent CLI 不是只“调用工具”，而是在用户本机执行有副作用的操作。真实风险是：

- 执行用户没有授权的命令。
- 修改或删除授权边界之外的文件。
- 触碰没有 Git 或备份兜底的重要目录、系统文件、密钥或隐私数据。
- 在用户没有充分理解副作用时，把模型生成的命令当成可信操作直接执行。

核心问题：

> Codex 如何避免本地 Agent 变成不受控的远程执行器？

### 1.2 用户初始假设

用户认为最危险的是执行了不该执行的命令，尤其是操作授权范围之外的文件夹。工作区内误删通常还能靠 Git 恢复；工作区外的目录可能没有版本控制，甚至可能包含重要系统文件，因此风险更高。

用户最初倾向于按“危险等级”分层：

1. 用户明确禁止的命令应该最先拦截。
2. `rm -rf` 等高度危险命令任何时候都需要 double check，可以用黑名单机制加小模型判断危险程度。
3. 用户白名单命令可以直接执行。
4. 无害的简单读命令可以直接执行。
5. 不明确的情况寻求用户同意。

后来用户校正到：这更像权限强弱分层，不一定是源码里的责任边界分层。

### 1.3 总体校准

用户对核心风险的判断成立。真正危险的不是命令执行失败，而是 Agent 越过用户理解和授权的边界，对不可恢复或未授权资源产生副作用。

但 Codex 源码不是按单一“命令危险等级”组织，而是按责任边界组织：

```text
这个阶段能知道什么信息？
这个阶段能做什么决定？
这个阶段失败后谁来兜底？
```

当前专题的核心结论：

> Codex 不把安全性押在一次“命令危险判断”上，而是把工具调用拆成结构门禁、请求级审批、sandbox-first 执行、失败后受控升级和审计回流多个责任边界。

## 2. 责任边界总览

### 2.1 已读源码入口

- `core/src/tools/registry.rs`
- `core/src/tools/sandboxing.rs`
- `core/src/tools/orchestrator.rs`
- `core/src/tools/runtimes/shell.rs`
- `core/src/tools/runtimes/unified_exec.rs`
- `core/src/exec_policy.rs`
- `core/src/exec_policy_tests.rs`

### 2.2 分层模型

```text
ToolRegistry
-> ToolRuntime / Approvable
-> ExecPolicyManager
-> ToolOrchestrator
-> SandboxAttempt
-> Audit / hooks / telemetry / tool response
```

#### ToolRegistry：结构性门禁

`ToolRegistry::dispatch_any` 处理工具分发前后的结构性问题：

- 工具是否存在。
- payload 是否和工具类型匹配。
- 是否触发 `pre_tool_use_hooks`。
- 工具是否 mutating。
- mutating tool 是否需要等待 `tool_call_gate`。
- 成功后是否触发 `post_tool_use_hooks`。
- telemetry / dispatch trace / post hook 是否记录工具结果或补充上下文。

这一层不是审批中心，而是保证工具调用能被正确、安全地分发。

#### ToolRuntime / Approvable：请求级审批需求

具体 runtime 可以给出本次请求的审批需求、approval key、permission hook payload 和 first-attempt sandbox override。

#### ExecPolicyManager：命令级策略判断

`create_exec_approval_requirement_for_command` 把命令、policy、sandbox 上下文和 prefix rule 合并为 `ExecApprovalRequirement`。

#### ToolOrchestrator：审批、sandbox 和 retry 编排

主流程：

```text
approval requirement
-> request approval if needed
-> select initial sandbox
-> first attempt
-> if SandboxErr::Denied
-> check escalation policy
-> request approval for retry if allowed
-> second attempt with SandboxType::None
```

#### SandboxAttempt：策略落到真实执行环境

`SandboxAttempt` 把策略转换为 OS / exec-server 能执行的约束：

- sandbox 类型
- permission profile
- cwd
- managed network
- Linux / Windows sandbox 参数
- network denial cancellation token

## 3. `ExecApprovalRequirement`：审批和沙箱不是一个维度

### 3.1 证据锚点

- `core/src/tools/sandboxing.rs`
  - `ExecApprovalRequirement`
  - `sandbox_override_for_first_attempt`
- `core/src/tools/orchestrator.rs`
  - first attempt sandbox 选择

### 3.2 源码概念

`ExecApprovalRequirement` 有三个结果：

```rust
pub(crate) enum ExecApprovalRequirement {
    Skip {
        bypass_sandbox: bool,
        proposed_execpolicy_amendment: Option<ExecPolicyAmendment>,
    },
    NeedsApproval {
        reason: Option<String>,
        proposed_execpolicy_amendment: Option<ExecPolicyAmendment>,
    },
    Forbidden { reason: String },
}
```

它不是“命令危险等级枚举”，而是告诉 `ToolOrchestrator` 对这一次 tool call 应该采取什么审批策略。

### 3.3 用户复述

用户源码阅读后总结：

- 默认编排规则是工具尽量在沙箱里执行。
- 第一次执行前会通过 `sandbox_override_for_first_attempt` 判断是否可以跳过沙箱。
- 两种情况可能跳过 first-attempt sandbox：
  1. 命令明确要求跳过沙箱，例如 `SandboxPermissions::RequireEscalated`。
  2. 命令审批结论是 `ExecApprovalRequirement::Skip { bypass_sandbox: true, .. }`。

用户对枚举的理解：

- `Skip` 表示不需要审批，但不一定跳过沙箱；只有 `bypass_sandbox = true` 才会第一次就不进沙箱。
- `NeedsApproval` 表示需要审批，包含原因，也可能附带 `proposed_execpolicy_amendment`。
- `Forbidden` 表示直接拒绝，必须说明原因。

### 3.4 校准结论

用户理解成立。关键区分是：

```text
SkipApproval != BypassSandbox
```

`Skip` 只表示跳过审批；是否绕过 sandbox 还要看：

- `bypass_sandbox`
- `sandbox_permissions.requires_escalated_permissions()`
- `sandbox_mode_for_first_attempt`

`proposed_execpolicy_amendment` 也不是“已经加上的 allow 前缀”，而是审批 UI / 后续策略更新可以使用的建议 amendment。是否真正写入或生效，还取决于用户选择和审批流程。

### 3.5 保护的不变量

- 不需要审批不等于可以裸跑。
- 明确策略允许或显式请求提升权限，才可能绕过 first-attempt sandbox。
- Forbidden 代表策略上不允许继续，不能假装可以问用户绕过。

## 4. 为什么危险命令黑名单不够

### 4.1 引导问题

1. 一个命令是否危险，只看命令字符串够不够？还需要哪些上下文？
2. 如果命令在 sandbox 里失败，系统应该自动裸跑、直接拒绝，还是重新请求批准？为什么？
3. 用户点了一次“允许”，这个允许应该绑定到什么粒度：命令名、完整命令、cwd、权限 profile，还是一次性 call？

### 4.2 用户阶段性复述

用户总结：

1. 只看命令字符串不够。还需要知道当前 `cwd`、沙箱权限边界、权限配置、命令解析可靠性、每个 command segment 的规则覆盖情况、当前命令是否扩大权限范围，以及“问用户要权限”的策略是什么。
2. sandbox 失败后也要根据错误情况具体分析。命令错误这类问题应直接报错；命令正确但权限不足时，再判断能不能向用户要权限。如果不能，直接失败；如果能，再请求用户批准。
3. 用户点了允许，不仅仅是允许一个命令名。系统首先要把命令转为规范化命令，还要确定授权命令的工作目录范畴、沙箱权限，并根据命令解析可靠性判断是否生成更通用的权限声明建议。例如 here-doc 这类复杂解析回退不生成 future amendment。

### 4.3 源码证据

`ExecApprovalRequest` 不只带 `command`，还带：

```text
approval_policy
permission_profile
file_system_sandbox_policy
sandbox_cwd
sandbox_permissions
prefix_rule
```

`ShellRuntime::ApprovalKey` 包含：

```text
canonicalized command
cwd
sandbox_permissions
additional_permissions
```

`UnifiedExecApprovalKey` 额外包含：

```text
tty
```

### 4.4 校准结论

命令风险不是命令名属性，而是组合属性：

```text
命令风险 =
  命令本身
  + cwd
  + sandbox 边界
  + approval policy
  + requested permissions
  + 解析可靠性
  + 每个 command segment 的规则覆盖情况
```

允许一次，是 approval key；允许一类，是 exec policy amendment；允许绕过 sandbox，需要每个实际执行段都有明确 allow。

## 5. `exec_policy`：从 shell 命令到 `Allow / Prompt / Forbidden`

### 5.1 证据锚点

- `core/src/exec_policy.rs`
  - `ExecApprovalRequest`
  - `create_exec_approval_requirement_for_command`
  - `commands_for_exec_policy`
  - `render_decision_for_unmatched_command`
  - `try_derive_execpolicy_amendment_for_prompt_rules`
  - `try_derive_execpolicy_amendment_for_allow_rules`
  - `derive_requested_execpolicy_amendment_from_prefix_rule`
  - `prefix_rule_would_approve_all_commands`

### 5.2 主链路

```text
ExecApprovalRequest
-> commands_for_exec_policy
-> exec_policy.check_multiple_with_options
-> Decision::{Allow, Prompt, Forbidden}
-> ExecApprovalRequirement::{Skip, NeedsApproval, Forbidden}
```

`commands_for_exec_policy` 会尽量把完整 shell argv 拆成一组实际会执行的 commands，而不是只对整条字符串做前缀匹配。

### 5.3 用户复述

用户阅读后理解：

- `create_exec_approval_requirement_for_command` 会先通过 `commands_for_exec_policy` 尽量把完整 shell argv 拆成实际子命令，再逐条匹配 exec policy rules，合成 `Allow / Prompt / Forbidden`。
- 复合命令不会被简单前缀匹配放过，而是尽量拆分后分别判断，这对复合命令权限控制很关键。
- shell / unified exec 这类 runtime 会在 request 中携带 `create_exec_approval_requirement_for_command` 的结果，并通过 `exec_approval_requirement` 返回给 orchestrator。
- 没有自定义 `exec_approval_requirement` 的工具，会走 `default_exec_approval_requirement`，它只根据全局 `AskForApproval` 和 `FileSystemSandboxPolicy` 决定审批需求。

### 5.4 校准结论

`default_exec_approval_requirement` 不会让前面的判断白做：

```text
exec_policy 负责判断“这条具体命令应该 Allow / Prompt / Forbidden”；
runtime 把这个判断挂到 request 上；
orchestrator 优先使用 runtime 给出的判断；
如果 runtime 没给，才用 default_exec_approval_requirement 兜底。
```

`Decision` 的合成可以粗略理解为取更严格结果，但具体排序和 matched rules 记录应以 `check_multiple_with_options` 为准，不能只停留在“max”这个抽象。

## 6. `AskForApproval::Granular`：问用户本身也要被授权

### 6.1 证据锚点

- `core/src/tools/sandboxing.rs`
  - `default_exec_approval_requirement`
  - `Approvable::wants_no_sandbox_approval`
- `core/src/exec_policy.rs`
  - `prompt_is_rejected_by_policy`
- `codex-rs/protocol/src/protocol.rs`
  - `GranularApprovalConfig`

### 6.2 源码概念

`GranularApprovalConfig` 把不同审批请求拆成开关：

```rust
pub struct GranularApprovalConfig {
    pub sandbox_approval: bool,
    pub rules: bool,
    pub skill_approval: bool,
    pub request_permissions: bool,
    pub mcp_elicitations: bool,
}
```

### 6.3 用户复述

用户复述：

> 不是所有情况都应该问用户；“问用户”这件事本身，也要先走一套类似权限判断的逻辑。

### 6.4 校准结论

这个理解成立。`AskForApproval::Granular` 的核心不是“更频繁地问用户”，而是把不同来源的审批请求拆开控制：

- policy rule 触发的 prompt
- sandbox / 权限升级触发的 prompt
- skill approval
- request permissions
- MCP elicitation

这样系统不会把“需要审批”直接等同于“可以打扰用户”。用户确认不是万能兜底；有些策略会要求“这类事别问用户，直接不准做”。

## 7. sandbox-first 与失败后受控升级

### 7.1 证据锚点

- `core/src/tools/orchestrator.rs`
  - first attempt sandbox 选择
  - `SandboxErr::Denied`
  - `wants_no_sandbox_approval`
  - `start_approval_async`
- `core/src/tools/sandboxing.rs`
  - `Approvable::wants_no_sandbox_approval`

### 7.2 用户复述

用户源码初读判断：

- `Skip` 不一定不进 sandbox。
- approval key 绑定命令、cwd、沙箱权限、额外权限。
- sandbox 失败后，如果是网络权限，一般会询问用户是否提升网络权限；如果找不到审批管线，就返回错误结果。
- 如果工具明确 `escalate_on_failure = false`，直接结束。
- `OnFailure` / `UnlessTrusted` 会询问用户后再执行；`Never` / `OnRequest` 不走失败后升级询问；`Granular` 由具体开关控制。

### 7.3 校准结论

需要修正的措辞是：`Never` / `OnRequest` 不是“直接干”，而是在 sandbox denied 后不进入 no-sandbox approval retry 流程。结果通常是保留 sandbox denial，不自动裸跑。

关键不变量：

```text
sandbox failure 不能自动升级为 no-sandbox execution。
```

sandbox denied 后的处理取决于：

- 错误类型。
- tool 是否允许 `escalate_on_failure`。
- `AskForApproval` 是否允许 no-sandbox approval。
- `Granular` 的对应开关是否允许。
- 是否存在可用审批路径。

## 8. 多段命令、复杂解析与 future amendment

### 8.1 证据锚点

- `core/src/exec_policy.rs`
  - `commands_for_exec_policy`
  - `auto_amendment_allowed = !used_complex_parsing`
  - `try_derive_execpolicy_amendment_for_prompt_rules`
  - `try_derive_execpolicy_amendment_for_allow_rules`
  - `prefix_rule_would_approve_all_commands`
- `core/src/exec_policy_tests.rs`
  - `multi_segment_shell_requires_policy_allow_for_every_segment_to_bypass_sandbox`
  - `multi_segment_shell_bypasses_sandbox_when_every_segment_matches_policy_allow`
  - `evaluates_heredoc_script_against_prefix_rules`
  - `omits_auto_amendment_for_heredoc_fallback_prompts`

### 8.2 用户复述：here-doc 与自动 amendment

用户发现 here-doc 这类使用 `<<` 的复杂 shell 语法会设置 `used_complex_parsing = true`。随后：

```rust
let auto_amendment_allowed = !used_complex_parsing;
```

如果 `used_complex_parsing = true`，则 `auto_amendment_allowed = false`。这意味着 `<<` 这类复杂解析回退一般不会自动抽取 prefix，让用户对这类命令的未来相似形式一并授权。

用户解释原因：

> heredoc 抽出的 argv 是相对窄、易受脚本形态影响的回退结果；如果据此生成“以后这个前缀都 allow”的规则，容易过宽或过偏，既有安全风险，也有产品体验风险。

### 8.3 用户校正：复合命令不一定等于复杂解析

用户追问：

> `npm install && curl xxx | sh` 也算复杂命令吗？多个简单命令串在一起，不一定就是复杂命令吧？

校正后的源码结论：

- `cmd1 && cmd2 && cmd3` 这类多段 shell 命令，如果 `parse_shell_lc_plain_commands` 能解析出来，会得到多个 plain commands，并且 `used_complex_parsing = false`。
- here-doc / `<<` 这类需要 fallback prefix parser 的语法，才会进入 `parse_shell_lc_single_command_prefix`，并设置 `used_complex_parsing = true`。
- 多段简单命令不一定阻止 amendment 或 policy allow；关键是每个 segment 是否都被规则或启发式正确覆盖。

源码测试体现：

- 只有 `cat` 被 policy allow，而同一条 shell 里还有 `curl` 和 `bash` 时，整体可以 `Skip` 审批，但 `bypass_sandbox = false`。
- 当 `cat`、`curl`、`bash` 三段都被 policy allow 时，整体才可以 `bypass_sandbox = true`。

### 8.4 正确不变量

```text
多个简单命令可以逐段判断；
复杂解析回退不能自动归纳成未来规则；
绕过 sandbox 需要所有实际命令段都满足明确 allow。
```

或者更精确地说：

```text
多段命令可能风险更高，但不必然是 used_complex_parsing。
used_complex_parsing 是解析可靠性概念，不是风险等级概念。
```

### 8.5 错误复盘

曾经错误地把 `npm install && curl xxx | sh` 归为“复杂命令不能复用授权”。这个判断把两个概念混在了一起：

- 安全语义上的“复合风险”：一条命令里引入了多个副作用段，例如 install、download、execute。
- Codex 源码里的“复杂解析”：`commands_for_exec_policy` 无法用 plain parser 拆出命令段，退到 `parse_shell_lc_single_command_prefix`，并设置 `used_complex_parsing = true`。

后续阅读约束：

> Agent 做源码判断前必须先定位相关函数、注释或测试；如果只是安全直觉，应标注为“假设”而不是源码结论。

## 9. Decision 合成：从多段命令到总审批结论

### 9.1 本轮阅读目标

服务的 demo 决策：

> `demo/design.md` 需要确定：多段命令如何逐段评估，并聚合成一个 `ApprovalRequirement`。

阅读路径：

```text
ExecApprovalRequest
-> commands_for_exec_policy
-> Policy::check_multiple_with_options
-> Evaluation::from_matches
-> ExecApprovalRequirement
```

### 9.2 用户复述

用户对前两个问题的理解：

1. 如果复合命令中某一段是 `Forbidden`，总 Decision 应该是 `Forbidden`。源码证据是 `Evaluation::from_matches` 中：

```rust
let decision = matched_rules.iter().map(RuleMatch::decision).max();
```

同时 `Decision` 枚举顺序是：

```rust
pub enum Decision {
    Allow,
    Prompt,
    Forbidden,
}
```

因为 `Decision` 派生了 `Ord / PartialOrd`，`Forbidden` 是最严格结果。

2. `Decision` 和 `bypass_sandbox` 不是同一个层次。`Decision::Allow` 只会映射为 `ExecApprovalRequirement::Skip`，但 `bypass_sandbox` 还要额外判断每个 parsed command segment 是否都被 execpolicy 明确 allow。

用户引用的关键源码：

```rust
Decision::Allow => ExecApprovalRequirement::Skip {
    bypass_sandbox: commands.iter().all(|command| {
        exec_policy
            .matches_for_command_with_options(
                command,
                /*heuristics_fallback*/ None,
                &match_options,
            )
            .iter()
            .any(|rule_match| {
                is_policy_match(rule_match) && rule_match.decision() == Decision::Allow
            })
    }),
    proposed_execpolicy_amendment: if auto_amendment_allowed {
        try_derive_execpolicy_amendment_for_allow_rules(&evaluation.matched_rules)
    } else {
        None
    },
},
```

### 9.3 Agent 校准

这两个判断是准确的。

- `Evaluation::from_matches` 用 `max()` 把所有 matched rules 合成最严格 decision。
- `Decision::Allow` 仍然只是“不需要审批”的结论；是否跳过 sandbox 是更窄的条件，要求每个 parsed command segment 都命中明确的 policy allow rule，而不是只靠 safe heuristic。

第 3 个问题不是继续问 Codex 源码事实，而是在问 demo 设计接口：

> 如果我们要在 mini demo 里复现这个机制，`approval_requirement_for_command(...)` 应该接收什么，返回什么？

用户进一步回答：

> `approval_requirement_for_command` 要接收当前命令及其运行的上下文（cwd、permissions、approval policy、sandbox policy），输出 `ApprovalRequirement`：`skip(sandbox, amendment)`、`forbidden(reason)`、`needsapproval(reason, command, amendment)`。

Agent 校准：

- 这个接口抽象方向是对的：demo 不能只接收命令字符串，必须接收命令运行上下文。
- demo 的输出也不能只有 `Decision`，而应是更接近编排层能执行的 `ApprovalRequirement`。
- 需要保留一个源码差异：Codex 源码中的 `ExecApprovalRequirement::NeedsApproval` 变体本身包含 `reason` 和 `proposed_execpolicy_amendment`，不直接包含 `command`；`command` 来自外层 request/context。demo 可以把 `command` 放进 approval request 或 UI payload，但要知道这属于 demo 的接口选择。

### 9.4 当前验证状态

- 验证状态：`用户已复述` + `源码已核对` + `已映射到 demo 接口`
- demo delta：`approval_requirement_for_command` 接收 `CommandRequest` 和运行上下文，输出 `ApprovalRequirement`；`NeedsApproval` 的 command 字段在 demo 中可作为 approval request payload，而不是照搬源码枚举。

## 10. 保护的不变量

当前已验证的不变量：

- Agent 不能绕过用户授权边界直接裸跑高副作用命令。
- 工具调用必须先通过结构性门禁，再进入具体 runtime。
- `SkipApproval` 不等于 `NoSandbox`。
- sandbox failure 不能自动升级为无沙箱执行。
- no-sandbox retry 必须受到 approval policy、hook、guardian 或用户决策约束。
- “问用户”本身也要受策略控制。
- approval cache 不应只绑定命令名，而应绑定 command、cwd、sandbox permissions、additional permissions 等上下文。
- 复杂解析回退可以用于本次判断，但不能自动归纳成 future amendment。
- 绕过 sandbox 需要所有实际命令段都满足明确 allow。
- 工具副作用需要被 hook、telemetry、dispatch trace 和 turn state 记录。

## 11. 设计代价

- 权限逻辑分散在 registry、runtime trait、exec policy、orchestrator、sandbox attempt、hook、guardian、network approval 多处，读起来复杂。
- 代码不如单一“命令危险等级判断器”直观。
- 命令解析需要在安全性和可用性之间取舍：能拆则逐段判断；解析不可靠时保守处理 future amendment。
- 审批策略不只是 yes/no，还要区分来源：policy rule、sandbox escalation、skill、request permissions、MCP elicitation。

这种复杂度换来的是边界清晰：

```text
结构门禁
-> 请求级审批
-> 命令策略判断
-> 执行环境约束
-> 失败后受控升级
-> 审计回流
```

## 12. 可迁移模式

如果实现自己的 Agent/CLI，可以迁移这个分层：

```text
Tool Registry：工具存在性、payload 类型、hook、mutating 标记
Tool Runtime：请求级审批需求、approval key、sandbox preference
Exec Policy：命令解析、规则匹配、safe/dangerous heuristic、future amendment
Orchestrator：统一审批、sandbox-first、denied 后受控 retry
Sandbox Attempt：把策略落到真实执行环境
Audit/Trace：记录每次决策、失败和升级
```

第一版不建议照抄 Codex 的全部复杂度，可以先实现：

- `Forbidden / NeedsApproval / SkipApproval`
- `SkipApproval != BypassSandbox`
- approval key 绑定 `command + cwd + permissions`
- 工作区根目录边界
- sandbox first attempt
- denied 后必须重新审批才能提升权限
- 每次执行记录审计日志

## 13. 当前验证状态与下一步

### 已完成

- 用户已复述 `ExecApprovalRequirement` 的核心语义。
- 用户已复述 `Skip` 与 `bypass_sandbox` 的区别。
- 用户已复述为什么黑名单不够，以及命令风险需要上下文。
- 用户已复述 approval key 需要绑定 command、cwd、sandbox permissions、additional permissions。
- 用户已复述 sandbox denied 后不能自动裸跑。
- 用户已复述 `Granular`：问用户本身也要受策略控制。
- 用户已校正“复合命令不一定等于复杂解析”，并用源码测试支持。
- 用户已复述 Decision 合成的前两层：`Forbidden` 如何通过 `max()` 成为总决策，以及 `Decision::Allow` 为什么不等于 `bypass_sandbox = true`。

### 待补齐

- 将 `check_multiple_with_options` 的 Decision 合成映射成 demo 的函数输入/输出。
- 继续核对 shell / unified exec runtime 如何把 `ExecApprovalRequirement`、approval key、sandbox permissions 组装进 request。
- 视需要补做交互式 TUI 断点验证。
- 将本专题收敛成 mini demo 不变量清单。
