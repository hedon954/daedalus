# Orchestrator Retry

状态：`用户已复述 / 源码已核对`

## Learning Navigation

- Final artifact: [`demo/design.md`](../../demo/design.md)
- Current gap: sandbox denied 后的 retry 状态机
- Evidence needed: orchestrator 如何处理 sandbox denied、approval policy、escalate-on-failure 和 no-sandbox retry。

## User Question

`ToolOrchestrator` 在 sandbox denied 后，什么时候直接返回失败，什么时候重新请求审批，什么时候允许无沙箱再试一次？

## User Answer

用户复述：

> 这里主要是回答一个问题：要不要允许「无沙箱再试一次」，以及在这之前要不要再问用户一回。

用户总结的分支：

1. 有网络决策，但是拼不出审批用的 `network_approval_context`，没法走针对某 host 的网络放行这一套，只能直接拒绝并返回原来的结果，主要逻辑在 `network_approval_context_from_payload`。
2. 看 `AskForApproval` 全局配置策略。如果是 `Never` 或 `OnRequest`，一般直接拒绝服务；但如果之前的 `network_approval_context` 能拼出来，`OnRequest` 也可能先走一次审批。如果是 `OnFailure`、`UnlessTrusted`，或 `Granular { sandbox_approval = true }`，可以允许失败后无沙箱重试。
3. 算出 `retry_reason`，再判断重试前要不要问一次用户。如果不是严格模式下、之前已经审批过、且不是网络情况，可以直接重试；否则要再问用户。
4. 用户通过审批后，用 `run_attempt` 再次尝试执行。

## Agent Calibration

整体判断是准确的：这一段不是重新判断命令危险性，而是在 sandbox denied 之后编排“是否可以升级为无沙箱 retry，以及 retry 前是否需要新审批”。

需要校准两点：

- `AskForApproval::Never` 在 `wants_no_sandbox_approval` 中是 `false`，所以一般不会走无沙箱 retry。`should_bypass_approval` 对 `Never` 返回 `true`，但它位于更后面的“是否跳过 retry approval”判断；前面的 `wants_no_sandbox_approval` 已经把 `Never` 挡掉。
- `already_approved` 不是“之前已经重试过”，而是本次工具调用在首次 attempt 前已经通过过审批，例如 `NeedsApproval` 分支、严格审查下的 `Skip` 分支。它表示首次 attempt 已被批准，因此在非严格审查、非网络 retry 时，可以不再问一遍。

## Source Evidence

源码路径：

```text
ToolOrchestrator::run
-> run_attempt(first)
-> SandboxErr::Denied branch
-> network_approval_context_from_payload
-> wants_no_sandbox_approval
-> retry_reason
-> should_bypass_approval
-> request_approval if needed
-> run_attempt(escalated)
```

关键源码事实：

- `network_approval_context_from_payload` 只在 payload 是 ask decision、带 protocol、host 非空时返回 context；否则返回 `None`。
- `network_policy_decision.is_some()` 但 `network_approval_context.is_none()` 时，orchestrator 直接返回原始 sandbox denied。
- `tool.escalate_on_failure() == false` 时直接返回原始 sandbox denied。
- `tool.wants_no_sandbox_approval(approval_policy)` 控制是否允许为 no-sandbox retry 请求审批：
  - `OnFailure`：允许。
  - `UnlessTrusted`：允许。
  - `Never`：不允许。
  - `OnRequest`：不允许，但有一个网络审批例外。
  - `Granular`：取 `granular_config.sandbox_approval`。
- `OnRequest` 的网络审批例外需要同时满足：当前是 `OnRequest`、能构造 `network_approval_context`、默认 exec approval requirement 是 `NeedsApproval`。
- `retry_reason` 分两类：网络场景说明具体 host 被 policy blocked；非网络场景使用稳定文案 `command failed; retry without sandbox?`。
- `bypass_retry_approval` 需要同时满足：不是 strict auto review、runtime 认为可以跳过审批、且不是 network retry。
- 如果不能 bypass retry approval，则构造 `ApprovalCtx { retry_reason, network_approval_context }`，用 `request_approval` 走 hook / guardian / user approval。
- 审批通过后，构造 `SandboxAttempt { sandbox: SandboxType::None, ... }`，再 `run_attempt` 第二次执行。

## Demo Delta

demo 的 `SandboxRetryState` 可以收敛为：

```text
SandboxRetryState
  - first_attempt_result
  - sandbox_denied_output
  - network_approval_context
  - can_escalate_on_failure
  - can_request_no_sandbox_approval
  - retry_reason
  - retry_requires_approval
  - retry_approved
  - second_attempt_result
```

demo 状态机：

```text
first sandbox attempt
  -> ok: return output
  -> sandbox denied:
      if network decision exists but approval context cannot be built: return denied
      if tool does not escalate on failure: return denied
      if approval policy does not allow no-sandbox approval:
          allow only OnRequest + network approval context + default needs approval
          otherwise return denied
      build retry reason
      if strict review OR no previous approval OR network retry:
          request approval
          if rejected: return rejected
      run second attempt with SandboxType::None
```

核心不变量：

```text
sandbox denied 不能自动裸跑；
no-sandbox retry 必须先经过 policy gate；
network retry 必须绑定可解释的 host/protocol approval context；
严格自动审查下，sandboxed attempt 的 approval 不覆盖 no-sandbox retry。
```
