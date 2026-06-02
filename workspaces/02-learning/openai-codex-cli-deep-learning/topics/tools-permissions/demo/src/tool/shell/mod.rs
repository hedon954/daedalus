use std::sync::Arc;

use crate::{
    model::{
        approval::ApprovalRequirement,
        command_request::CommandRequest,
        event::{ExecutionAttempt, RetryDecision, UserApprovalDecision},
        execution::{ExecutionFailure, ExecutionResult},
    },
    tool::shell::{
        approval::{ApprovalDecider, build_approval_scope, resolve_approval_requirement},
        execution::ExecutionRunner,
        registry::MatchedCapability,
        retry::decide_retry,
    },
};

pub mod approval;
pub mod execution;
pub mod registry;
mod retry;

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
pub fn run_shell_command(
    request: &CommandRequest,
    matched_capability: MatchedCapability,
    execution_runner: Arc<dyn ExecutionRunner + Send + Sync + 'static>,
    approval_decider: Arc<dyn ApprovalDecider + Send + Sync + 'static>,
) -> RunCommandResult {
    match resolve_approval_requirement(&request, &matched_capability) {
        ApprovalRequirement::Skip {
            bypass_sandbox,
            reason,
        } => {
            // 不需要审批但是需要先走一遍沙箱
            if !bypass_sandbox {
                return run_sandbox_first_flow(
                    request,
                    matched_capability,
                    execution_runner,
                    approval_decider,
                );
            }

            // 不需要审批且跳过沙箱，直接在本地运行
            return run_no_sandbox_first(request, execution_runner.as_ref(), reason);
        }
        ApprovalRequirement::NeedsApproval {
            reason,
            approval_scope,
        } => match approval_decider.decide(&reason, &approval_scope) {
            UserApprovalDecision::Approved { persistence: _ } => {
                return run_sandbox_first_flow(
                    request,
                    matched_capability,
                    execution_runner,
                    approval_decider,
                );
            }
            UserApprovalDecision::Rejected => {
                return RunCommandResult::Denied { reason };
            }
        },
        ApprovalRequirement::Forbidden { reason } => RunCommandResult::Denied { reason },
    }
}

fn run_sandbox_first_flow(
    request: &CommandRequest,
    matched_capability: MatchedCapability,
    execution_runner: Arc<dyn ExecutionRunner + Send + Sync + 'static>,
    approval_decider: Arc<dyn ApprovalDecider + Send + Sync + 'static>,
) -> RunCommandResult {
    // 先在沙箱尝试执行
    let sandbox_first_execution_result = execution_runner.run(
        &request,
        &ExecutionAttempt::SandboxFirst {
            sandbox_profile: request.sandbox_profile,
        },
    );
    match sandbox_first_execution_result {
        // 沙箱成功直接返回
        ExecutionResult::Success { stdout } => {
            return RunCommandResult::Finished { output: stdout };
        }

        // 沙箱失败则判断是否要进行重试
        ExecutionResult::Failure(failure) => match decide_retry(
            request,
            &failure,
            &build_approval_scope(request, &matched_capability),
            matched_capability.capability.policy.retry_policy,
            false,
        ) {
            // 不重试直接返回失败
            RetryDecision::DoNotRetry { reason: _ } => {
                return failure.into();
            }
            // 重试且不需要审批则直接运行
            RetryDecision::RetryWithoutApproval { reason } => {
                return run_no_sandbox_retry(request, execution_runner.as_ref(), reason);
            }
            // 重新且需要审批则先走审批
            RetryDecision::RetryWithApproval {
                reason,
                approval_scope,
            } => match approval_decider.decide(&reason, &approval_scope) {
                // 审批通过则直接本机运行再次尝试
                UserApprovalDecision::Approved { persistence: _ } => {
                    return run_no_sandbox_retry(request, execution_runner.as_ref(), reason);
                }
                // 审批不通过则直接拒绝
                UserApprovalDecision::Rejected => {
                    return RunCommandResult::Denied { reason };
                }
            },
        },
    }
}

fn run_no_sandbox_first(
    request: &CommandRequest,
    execution_runner: &dyn ExecutionRunner,
    reason: String,
) -> RunCommandResult {
    execution_runner
        .run(request, &ExecutionAttempt::NoSandboxFirst { reason })
        .into()
}

fn run_no_sandbox_retry(
    request: &CommandRequest,
    execution_runner: &dyn ExecutionRunner,
    reason: String,
) -> RunCommandResult {
    execution_runner
        .run(request, &ExecutionAttempt::NoSandboxRetry { reason })
        .into()
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

impl From<ExecutionResult> for RunCommandResult {
    fn from(value: ExecutionResult) -> Self {
        match value {
            ExecutionResult::Success { stdout } => RunCommandResult::Finished { output: stdout },
            ExecutionResult::Failure(failure) => failure.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        approval::{
            ApprovalPersistence, ApprovalPolicy, ApprovalScope, NetworkPolicy, SandboxProfile,
        },
        capability::CapabilityKind,
        execution::NetworkApprovalContext,
    };
    use crate::tool::shell::registry::CapabilityRegistry;
    use std::{
        collections::VecDeque,
        path::PathBuf,
        sync::{Arc, Mutex},
    };

    struct RecordingExecutionRunner {
        results: Mutex<VecDeque<ExecutionResult>>,
        attempts: Mutex<Vec<ExecutionAttempt>>,
    }

    impl RecordingExecutionRunner {
        fn new(results: Vec<ExecutionResult>) -> Self {
            Self {
                results: Mutex::new(VecDeque::from(results)),
                attempts: Mutex::new(Vec::new()),
            }
        }

        fn attempts(&self) -> Vec<ExecutionAttempt> {
            self.attempts.lock().unwrap().clone()
        }
    }

    impl ExecutionRunner for RecordingExecutionRunner {
        fn run(&self, _request: &CommandRequest, attempt: &ExecutionAttempt) -> ExecutionResult {
            self.attempts.lock().unwrap().push(attempt.clone());
            self.results
                .lock()
                .unwrap()
                .pop_front()
                .expect("test runner should have enough scripted results")
        }
    }

    struct RecordingApprovalDecider {
        decisions: Mutex<VecDeque<UserApprovalDecision>>,
        scopes: Mutex<Vec<ApprovalScope>>,
    }

    impl RecordingApprovalDecider {
        fn new(decisions: Vec<UserApprovalDecision>) -> Self {
            Self {
                decisions: Mutex::new(VecDeque::from(decisions)),
                scopes: Mutex::new(Vec::new()),
            }
        }

        fn scopes(&self) -> Vec<ApprovalScope> {
            self.scopes.lock().unwrap().clone()
        }
    }

    impl ApprovalDecider for RecordingApprovalDecider {
        fn decide(&self, _reason: &str, scope: &ApprovalScope) -> UserApprovalDecision {
            self.scopes.lock().unwrap().push(scope.clone());
            self.decisions
                .lock()
                .unwrap()
                .pop_front()
                .expect("test approval decider should have enough scripted decisions")
        }
    }

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| item.to_string()).collect()
    }

    fn request(
        argv: &[&str],
        capability: CapabilityKind,
        approval_policy: ApprovalPolicy,
        sandbox_profile: SandboxProfile,
        network_policy: NetworkPolicy,
    ) -> CommandRequest {
        CommandRequest {
            raw_command: argv.join(" "),
            argv: strings(argv),
            cwd: PathBuf::from("/workspace"),
            capability,
            approval_policy,
            sandbox_profile,
            network_policy,
            justification: None,
        }
    }

    fn matched(argv: &[&str]) -> MatchedCapability {
        CapabilityRegistry::new()
            .match_capability(&strings(argv))
            .expect("test command should match a capability")
    }

    fn approved() -> UserApprovalDecision {
        UserApprovalDecision::Approved {
            persistence: ApprovalPersistence::Once,
        }
    }

    fn execution_runner(
        results: Vec<ExecutionResult>,
    ) -> (
        Arc<RecordingExecutionRunner>,
        Arc<dyn ExecutionRunner + Send + Sync + 'static>,
    ) {
        let runner = Arc::new(RecordingExecutionRunner::new(results));
        let trait_object = runner.clone() as Arc<dyn ExecutionRunner + Send + Sync + 'static>;
        (runner, trait_object)
    }

    fn approval_decider(
        decisions: Vec<UserApprovalDecision>,
    ) -> (
        Arc<RecordingApprovalDecider>,
        Arc<dyn ApprovalDecider + Send + Sync + 'static>,
    ) {
        let decider = Arc::new(RecordingApprovalDecider::new(decisions));
        let trait_object = decider.clone() as Arc<dyn ApprovalDecider + Send + Sync + 'static>;
        (decider, trait_object)
    }

    fn assert_finished(result: RunCommandResult, expected_output: &str) {
        match result {
            RunCommandResult::Finished { output } => assert_eq!(output, expected_output),
            other => panic!("expected command to finish, got {}", result_label(&other)),
        }
    }

    fn assert_failed(result: RunCommandResult) {
        match result {
            RunCommandResult::Failed { .. } => {}
            other => panic!("expected command failure, got {}", result_label(&other)),
        }
    }

    fn assert_denied(result: RunCommandResult) {
        match result {
            RunCommandResult::Denied { .. } => {}
            other => panic!("expected command denial, got {}", result_label(&other)),
        }
    }

    #[test]
    fn skip_without_bypass_runs_once_in_sandbox() {
        let argv = ["cat", "package.json"];
        let request = request(
            &argv,
            CapabilityKind::SafeRead,
            ApprovalPolicy::OnFailure,
            SandboxProfile::ReadOnly,
            NetworkPolicy::Deny,
        );
        let (runner, runner_trait) = execution_runner(vec![ExecutionResult::Success {
            stdout: "read ok".to_string(),
        }]);
        let (decider, decider_trait) = approval_decider(vec![]);

        let result = run_shell_command(&request, matched(&argv), runner_trait, decider_trait);

        assert_finished(result, "read ok");
        assert_eq!(
            runner.attempts(),
            vec![ExecutionAttempt::SandboxFirst {
                sandbox_profile: SandboxProfile::ReadOnly
            }]
        );
        assert!(decider.scopes().is_empty());
    }

    #[test]
    fn safe_read_sandbox_denied_does_not_retry_because_capability_retry_policy_is_never() {
        let argv = ["cat", "/private/file"];
        let request = request(
            &argv,
            CapabilityKind::SafeRead,
            ApprovalPolicy::OnFailure,
            SandboxProfile::ReadOnly,
            NetworkPolicy::Deny,
        );
        let (runner, runner_trait) = execution_runner(vec![ExecutionResult::Failure(
            ExecutionFailure::SandboxDenied {
                output: "read denied by sandbox".to_string(),
                network_context: None,
            },
        )]);
        let (decider, decider_trait) = approval_decider(vec![]);

        let result = run_shell_command(&request, matched(&argv), runner_trait, decider_trait);

        assert_denied(result);
        assert_eq!(
            runner.attempts(),
            vec![ExecutionAttempt::SandboxFirst {
                sandbox_profile: SandboxProfile::ReadOnly
            }]
        );
        assert!(decider.scopes().is_empty());
    }

    #[test]
    fn needs_approval_rejection_denies_without_running_command() {
        let argv = ["npm", "install", "vite"];
        let request = request(
            &argv,
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnRequest,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Prompt,
        );
        let (runner, runner_trait) = execution_runner(vec![]);
        let (decider, decider_trait) = approval_decider(vec![UserApprovalDecision::Rejected]);

        let result = run_shell_command(&request, matched(&argv), runner_trait, decider_trait);

        assert_denied(result);
        assert!(runner.attempts().is_empty());
        let scopes = decider.scopes();
        assert_eq!(scopes.len(), 1);
        assert_eq!(scopes[0].sandbox_profile, SandboxProfile::WorkspaceWrite);
    }

    #[test]
    fn needs_approval_approval_still_runs_sandbox_first() {
        let argv = ["npm", "install", "vite"];
        let request = request(
            &argv,
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnRequest,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Prompt,
        );
        let (runner, runner_trait) = execution_runner(vec![ExecutionResult::Success {
            stdout: "install ok in sandbox".to_string(),
        }]);
        let (decider, decider_trait) = approval_decider(vec![approved()]);

        let result = run_shell_command(&request, matched(&argv), runner_trait, decider_trait);

        assert_finished(result, "install ok in sandbox");
        assert_eq!(
            runner.attempts(),
            vec![ExecutionAttempt::SandboxFirst {
                sandbox_profile: SandboxProfile::WorkspaceWrite
            }]
        );
        let scopes = decider.scopes();
        assert_eq!(scopes.len(), 1);
        assert_eq!(scopes[0].sandbox_profile, SandboxProfile::WorkspaceWrite);
    }

    #[test]
    fn command_failure_in_sandbox_does_not_retry() {
        let argv = ["npm", "test", "--", "fail"];
        let request = request(
            &argv,
            CapabilityKind::SafeTest,
            ApprovalPolicy::OnFailure,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Deny,
        );
        let (runner, runner_trait) = execution_runner(vec![ExecutionResult::Failure(
            ExecutionFailure::CommandFailed {
                exit_code: 1,
                stderr: "test failed".to_string(),
            },
        )]);
        let (_decider, decider_trait) = approval_decider(vec![]);

        let result = run_shell_command(&request, matched(&argv), runner_trait, decider_trait);

        assert_failed(result);
        assert_eq!(
            runner.attempts(),
            vec![ExecutionAttempt::SandboxFirst {
                sandbox_profile: SandboxProfile::WorkspaceWrite
            }]
        );
    }

    #[test]
    fn sandbox_denied_with_network_allow_retries_without_second_approval() {
        let argv = ["npm", "install", "vite"];
        let request = request(
            &argv,
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnFailure,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Allow,
        );
        let (runner, runner_trait) = execution_runner(vec![
            ExecutionResult::Failure(ExecutionFailure::SandboxDenied {
                output: "network blocked by sandbox".to_string(),
                network_context: Some(NetworkApprovalContext {
                    host: "registry.npmjs.org".to_string(),
                    protocol: "https".to_string(),
                }),
            }),
            ExecutionResult::Success {
                stdout: "install ok without sandbox".to_string(),
            },
        ]);
        let (decider, decider_trait) = approval_decider(vec![approved()]);

        let result = run_shell_command(&request, matched(&argv), runner_trait, decider_trait);

        assert_finished(result, "install ok without sandbox");
        let attempts = runner.attempts();
        assert_eq!(attempts.len(), 2);
        assert_eq!(
            attempts[0],
            ExecutionAttempt::SandboxFirst {
                sandbox_profile: SandboxProfile::WorkspaceWrite
            }
        );
        assert!(matches!(
            attempts[1],
            ExecutionAttempt::NoSandboxRetry { .. }
        ));
        assert_eq!(decider.scopes().len(), 1);
    }

    #[test]
    fn sandbox_denied_with_network_prompt_requires_retry_approval_with_no_sandbox_scope() {
        let argv = ["npm", "install", "vite"];
        let request = request(
            &argv,
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnFailure,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Prompt,
        );
        let (runner, runner_trait) = execution_runner(vec![
            ExecutionResult::Failure(ExecutionFailure::SandboxDenied {
                output: "network blocked by sandbox".to_string(),
                network_context: Some(NetworkApprovalContext {
                    host: "registry.npmjs.org".to_string(),
                    protocol: "https".to_string(),
                }),
            }),
            ExecutionResult::Success {
                stdout: "install ok after approval".to_string(),
            },
        ]);
        let (decider, decider_trait) = approval_decider(vec![approved(), approved()]);

        let result = run_shell_command(&request, matched(&argv), runner_trait, decider_trait);

        assert_finished(result, "install ok after approval");
        let attempts = runner.attempts();
        assert_eq!(attempts.len(), 2);
        assert_eq!(
            attempts[0],
            ExecutionAttempt::SandboxFirst {
                sandbox_profile: SandboxProfile::WorkspaceWrite
            }
        );
        assert!(matches!(
            attempts[1],
            ExecutionAttempt::NoSandboxRetry { .. }
        ));
        let scopes = decider.scopes();
        assert_eq!(scopes.len(), 2);
        assert_eq!(scopes[0].sandbox_profile, SandboxProfile::WorkspaceWrite);
        assert_eq!(scopes[1].sandbox_profile, SandboxProfile::NoSandbox);
    }

    #[test]
    fn sandbox_denied_retry_approval_rejection_denies_without_no_sandbox_retry() {
        let argv = ["npm", "install", "vite"];
        let request = request(
            &argv,
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnFailure,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Prompt,
        );
        let (runner, runner_trait) = execution_runner(vec![ExecutionResult::Failure(
            ExecutionFailure::SandboxDenied {
                output: "network blocked by sandbox".to_string(),
                network_context: Some(NetworkApprovalContext {
                    host: "registry.npmjs.org".to_string(),
                    protocol: "https".to_string(),
                }),
            },
        )]);
        let (decider, decider_trait) =
            approval_decider(vec![approved(), UserApprovalDecision::Rejected]);

        let result = run_shell_command(&request, matched(&argv), runner_trait, decider_trait);

        assert_denied(result);
        assert_eq!(
            runner.attempts(),
            vec![ExecutionAttempt::SandboxFirst {
                sandbox_profile: SandboxProfile::WorkspaceWrite
            }]
        );
        let scopes = decider.scopes();
        assert_eq!(scopes.len(), 2);
        assert_eq!(scopes[1].sandbox_profile, SandboxProfile::NoSandbox);
    }

    fn result_label(result: &RunCommandResult) -> &'static str {
        match result {
            RunCommandResult::Finished { .. } => "Finished",
            RunCommandResult::Failed { .. } => "Failed",
            RunCommandResult::Denied { .. } => "Denied",
        }
    }
}
