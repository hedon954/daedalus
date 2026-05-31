use std::sync::Arc;

use crate::{
    model::{
        approval::ApprovalRequirement,
        command_request::CommandRequest,
        event::{ExecutionAttempt, RetryDecision},
        execution::{ExecutionFailure, ExecutionResult},
    },
    sandbox::SandboxRunner,
    tool::shell::{
        approval::{build_approval_scope, decide_approval},
        registry::MatchedCapability,
        retry::decide_retry,
        run_local::run_local_shell,
    },
};

mod approval;
pub mod registry;
mod retry;
mod run_local;

/// `run_command` tool 的参数结构。
///
/// 模型只能提供命令文本和解释文本；cwd、capability、sandbox、network policy
/// 都由 host 侧的 `ToolRuntime` 组装。
#[derive(Debug, serde::Deserialize)]
pub struct RunCommandArgs {
    pub command: String,
    #[serde(default)]
    pub justification: Option<String>,
}

/// shell command runtime 的终态。
///
/// Approval / sandbox / retry 都在 `run_shell_command` 内部被消费并折叠为这三个结果。
pub enum RunCommandResult {
    Finished { output: String },
    Failed { error: String },
    Denied { reason: String },
}

/// 执行 shell 命令。
///
/// 当前目标是先跑通单命令主链路：
/// approval gate -> sandbox first -> retry gate -> optional no-sandbox retry。
///
/// TODO: 补齐 `NeedsApproval` 和 `RetryWithApproval` 的 Phase 1 scripted approval。
/// TODO: 把 no-sandbox first / retry 也改为 runner attempt，而不是直接调用 `run_local_shell`。
pub fn run_shell_command(
    request: &CommandRequest,
    matched_capability: MatchedCapability,
    sandbox_runner: Arc<dyn SandboxRunner + Send + Sync + 'static>,
) -> RunCommandResult {
    match decide_approval(&request, &matched_capability) {
        ApprovalRequirement::Skip {
            bypass_sandbox,
            reason: skip_reason,
        } => {
            println!(
                "skip approval for running command: {}, because {}",
                request.raw_command, skip_reason
            );

            // 不需要审批但是需要先走一遍沙箱
            if !bypass_sandbox {
                let sandbox_first_execution_result = sandbox_runner.run(
                    &request,
                    &ExecutionAttempt::SandboxFirst {
                        sandbox_profile: matched_capability.capability.policy.first_attempt_sandbox,
                    },
                );
                match sandbox_first_execution_result {
                    ExecutionResult::Success { stdout } => {
                        return RunCommandResult::Finished { output: stdout };
                    }
                    ExecutionResult::Failure(failure) => match decide_retry(
                        request,
                        &failure,
                        &build_approval_scope(request, &matched_capability),
                        false,
                    ) {
                        RetryDecision::DoNotRetry {
                            reason: not_retry_reason,
                        } => {
                            println!(
                                "run [{}] failed in sandbox, but do not retry because: {}",
                                request.raw_command, not_retry_reason
                            );
                            return failure.into();
                        }
                        RetryDecision::RetryWithoutApproval { reason } => {
                            println!(
                                "retry [{}] without approval, reason: {}",
                                request.raw_command, reason,
                            );
                            return run_local_shell(request);
                        }
                        RetryDecision::RetryWithApproval {
                            reason: _reason,
                            approval_scope: _approval_scope,
                        } => {
                            // TODO: Phase 1 用 scripted approval / ApprovalDecider 决定是否重试。
                            unimplemented!()
                        }
                    },
                }
            }

            // 不需要审批且跳过沙箱，直接在本地运行
            return run_local_shell(request);
        }
        ApprovalRequirement::NeedsApproval {
            reason: _reason,
            approval_scope: _approval_scope,
        } => {
            // TODO: Phase 1 先在 shell runtime 内部消费审批；Phase 2 再透出 pause/resume 事件。
            unimplemented!()
        }
        ApprovalRequirement::Forbidden { reason } => RunCommandResult::Denied { reason },
    }
}

/// 将底层执行失败折叠成 command runtime 终态。
impl From<ExecutionFailure> for RunCommandResult {
    fn from(value: ExecutionFailure) -> Self {
        match value {
            ExecutionFailure::CommandFailed { exit_code, stderr } => RunCommandResult::Failed {
                error: format!("exit_code: {exit_code}, stderr: {stderr}"),
            },
            ExecutionFailure::SandboxDenied {
                output,
                network_context,
            } => RunCommandResult::Denied {
                reason: format!("denied: {}, network_context: {:?}", output, network_context),
            },
        }
    }
}
