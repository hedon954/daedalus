# Slice 6 Command Runtime Review Guide

这份 guide 记录当前 `ToolRuntime` / `tool::shell` 重构后的 review 结论。它不是最终实现说明，而是下一轮继续实现 `run_shell_command` 前的行动地图。

## Learning Navigation

- Final artifact: [`../../demo/README.md`](../../demo/README.md)
- Current stage: `08-demo-coder`
- Current slice: Slice 6 Agent Orchestrator
- Current gap: `ToolRuntime` 已能识别 `run_command` 并构造 `CommandRequest`，但 `run_shell_command` 仍未实现完整 approval / sandbox / retry 链路。
- After this: 完成 command runtime 后，ReAct loop 才真正恢复 Codex 学习里的权限 / 沙箱 / retry 不变量。

## Current Code Shape

当前结构方向是合理的：

```text
agent/react.rs
  -> ToolRuntime::run(call)

tool/runtime.rs
  -> find tool
  -> plan pure function or command
  -> execute plan

tool/function.rs
  -> add/sub pure function tools

tool/shell/
  -> run_command argument shape
  -> capability registry
  -> approval decision
  -> retry decision
  -> run_shell_command orchestration
```

`model/` 继续保留领域类型，例如 `CommandRequest`、`ApprovalRequirement`、`ExecutionAttempt`、`ExecutionResult` 和 `RetryDecision`。`tool/shell/` 只放把模型的 `run_command` tool call 落地为这些领域类型的实现逻辑。

## Naming Contract

模型可见的 tool name 应该是 `run_command`：

```text
ToolCallFinished.name == "run_command"
```

内部模块名可以继续叫 `tool/shell/`，因为它实现的是本地命令工具。

不要把模型可见工具名改成 `shell`，除非同步修改 `01-rust-openai-integration.md` 的 schema 和所有测试。当前已定接口是：

```json
{
  "command": "npm test",
  "justification": "为了验证当前 demo"
}
```

其中：

```text
command       -> 权限判断事实
justification -> 审批解释文本，不授予权限
```

## Review Findings

### 1. `ToolRuntimePlan::Deny` 已从当前设计中移除

最新决策：当前不要保留 `ToolRuntimePlan::Deny`。

原因是 `ToolRuntimePlan` 只表达“准备如何执行”，而 `Denied` 是 command runtime 经过 capability / approval / retry 后得到的执行结果。把安全拒绝放在 plan 层，会让 `plan_call` 提前承担它还没有判断过的权限语义。

当前分类：

```text
invalid JSON / missing field / empty command
  -> ToolRuntimePlan::Fail
  -> ToolRuntimeResult::Failed

command parsed successfully but no capability matched
  -> ToolRuntimePlan::Fail
  -> ToolRuntimeResult::Failed

capability matched but approval decision is Forbidden
  -> RunCommandResult::Denied

previous tool caused hard denial, later calls are not attempted
  -> ToolRuntimeResult::Skipped
```

也就是说：

```text
Fail = 没能形成有效计划，模型可以修正 tool arguments / command
Denied = 已进入 command runtime，但安全策略或用户审批不允许继续执行
```

如果后续希望 unknown command 呈现为安全拒绝，不要把 `Deny` 加回 `ToolRuntimePlan`；更好的做法是让 registry 显式匹配一个 `unsupported` / `dangerous-shell` capability，再由 approval policy 产出 `RunCommandResult::Denied`。

### 2. `run_shell_command` 通过参数接收 runner

当前代码已选择让 `run_shell_command` 通过参数接收 runner。这个方向是对的：command tool 的执行链路可以收口在 `tool/shell/`，同时测试仍能断言 “Forbidden 时 runner 没有执行”。

当前形态：

```rust
pub fn run_shell_command(
    request: CommandRequest,
    matched_capability: MatchedCapability,
    runner: Arc<dyn SandboxRunner + Send + Sync + 'static>,
) -> RunCommandResult
```

如果后续 runner 依赖继续增多，再考虑升级为：

```rust
pub struct ShellRuntime<R> {
    runner: R,
}
```

第一版 demo 直接沿用当前 `ToolRuntimeContext.runner` 注入即可。不要在 `run_shell_command` 里面偷偷创建 runner；那会让测试很难断言“Forbidden 时 runner 没有执行”。

### 3. `justification` 要和 schema 对齐

`01-rust-openai-integration.md` 中 `run_command` schema 要求：

```json
"required": ["command", "justification"]
```

因此当前 `RunCommandArgs` 最好让 `justification` 成为必填：

```rust
pub struct RunCommandArgs {
    pub command: String,
    pub justification: String,
}
```

如果保留 `Option<String>`，也应该在构造 `CommandRequest` 前验证：

```text
None or empty -> ToolRuntimePlan::Fail
```

注意：`justification` 只能进入 `CommandRequest.justification`、审批展示、event / log。它不应该改变 capability、sandbox、network 或 forbidden 判断。

### 4. `split_whitespace` 可以保留，但 dangerous heuristic 要补

Phase 1 可以接受简单命令解析，但当前 dangerous prefix 如果写成：

```text
["curl", "|", "sh"]
```

它匹配不到真实命令：

```text
curl https://example.com/install.sh | sh
```

因为简单切分后是：

```text
["curl", "https://example.com/install.sh", "|", "sh"]
```

下一步至少要补一个 Phase 1 heuristic：

```text
argv[0] == "curl" && argv contains "|" && argv contains "sh"
  -> dangerous-shell
```

或者更保守一点：

```text
argv[0] == "curl"
  -> dangerous-shell
```

这能保证 AT-03 不会因为 parser 太简化而漏掉高风险命令。

### 5. `ApprovalPolicy::OnRequest` 语义要统一

当前模型注释里已经写清楚：

```text
justification 只是解释文本，不能作为提权信号。
```

所以如果 demo 还没有 `require_escalated` 这类显式提权字段，`OnRequest` 不应该因为 `DefaultDecision::Prompt` 自动弹审批。

有两种可选路线：

```text
路线 A：当前 demo 没有显式提权信号
DefaultDecision::Prompt + OnRequest -> Forbidden

路线 B：把 run_command 本身定义成显式请求
DefaultDecision::Prompt + OnRequest -> NeedsApproval
并修改 ApprovalPolicy 注释，说明 run_command counts as explicit request
```

推荐路线 A，因为它更严格，也符合“justification 不是授权条件”的约束。

## Suggested `run_shell_command` Flow

`run_shell_command` 可以先写成一个直接的状态机，不急着拆太多 helper：

```text
decide_approval(request, matched_capability)
  -> Forbidden
       return Denied

  -> NeedsApproval
       Phase 1 先用 scripted approval / TODO gate
       rejected -> Denied
       approved -> continue

  -> Skip { bypass_sandbox: false }
       attempt = SandboxFirst { request.sandbox_profile }

  -> Skip { bypass_sandbox: true }
       attempt = NoSandboxFirst { reason }

runner.run(request, attempt)
  -> Success
       return Finished

  -> CommandFailed
       return Failed

  -> SandboxDenied
       decide_retry(request, failure, command_scope, already_retried=false)
         -> DoNotRetry
              return Failed
         -> RetryWithoutApproval
              runner.run(NoSandboxRetry)
         -> RetryWithApproval
              Phase 1 approval gate
```

第一版可以把 “NeedsApproval / RetryWithApproval 如何模拟用户批准” 做成最小策略。重点是不要让 `react.rs` 知道这些细节。

## Test Plan

接下来至少补这些测试：

- `run_command` 参数 JSON 非法 -> `Failed`，不锁错误文案。
- `run_command` 缺少 `justification` -> `Failed`。
- `run_command` 完全 unknown command -> `Failed`，不锁错误文案。
- `run_command` `cat package.json` -> safe-read -> sandbox success -> `Finished`。
- `run_command` `curl ... | sh` -> dangerous-shell -> `Denied`。
- command failed -> `Failed`，不 retry。
- sandbox denied + retry approval accepted -> no-sandbox retry success。
- sandbox denied + retry approval rejected -> `Denied` 或 `Failed`，但不执行 second attempt。

测试时继续遵守当前规则：错误路径默认只断言结果变体，不锁完整错误文案。

## Stop Rules

- 不实现完整 shell parser。
- 不实现真实用户审批 UI。
- 不接真实 OS sandbox。
- 不让模型直接传 `cwd`、`capability`、`sandbox_profile` 或 `network_policy`。
- 不把 `justification` 当成权限凭证。

## Immediate Next Action

下一步先做三件小事：

1. 保持 `ToolRuntimePlan = RunPureFunction / RunCommand / Fail`，不要把 `Deny` 加回 plan 层。
2. 在 `run_shell_command` 内实现 `Forbidden -> Denied`、`Skip -> runner.run`、`CommandFailed -> Failed` 的最小闭环。
3. 补 `run_command` unknown / dangerous / safe-read 三个测试，先证明 runner 是否被正确调用或跳过。
