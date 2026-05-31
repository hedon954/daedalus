use crate::model::{
    approval::{ApprovalPersistence, ApprovalPolicy, ApprovalScope, NetworkPolicy, SandboxProfile},
    command_request::CommandRequest,
    event::RetryDecision,
    execution::{ExecutionFailure, NetworkApprovalContext},
};

/// 判断一次执行失败后是否允许进入 no-sandbox retry。
///
/// retry gate 只在 sandbox first 失败后运行。它必须区分命令自身失败和
/// 沙箱/网络拒绝：前者直接回灌给模型修复，后者才可能进入提权重试。
pub fn decide_retry(
    request: &CommandRequest,
    failure: &ExecutionFailure,
    command_scope: &ApprovalScope,
    already_retried: bool,
) -> RetryDecision {
    if already_retried {
        return RetryDecision::DoNotRetry {
            reason: format!("already retried, last failure was {failure:?}"),
        };
    }

    match failure {
        ExecutionFailure::CommandFailed { exit_code, stderr } => RetryDecision::DoNotRetry {
            reason: format!("command failed with exit code {exit_code}: {stderr}"),
        },
        ExecutionFailure::SandboxDenied {
            output,
            network_context,
        } => match request.approval_policy {
            ApprovalPolicy::Never => RetryDecision::DoNotRetry {
                reason: format!(
                    "approval policy is never, so it is not allowed to retry: {output}"
                ),
            },
            ApprovalPolicy::OnRequest => RetryDecision::DoNotRetry {
                reason: format!(
                    "approval policy is on-request; sandbox failure cannot trigger automatic retry approval: {output}"
                ),
            },
            ApprovalPolicy::OnFailure => {
                if let Some(network_context) = network_context {
                    decide_network_retry(request, output, network_context, command_scope)
                } else {
                    decide_non_network_sandbox_retry(request, output, command_scope)
                }
            }
        },
    }
}

fn decide_non_network_sandbox_retry(
    request: &CommandRequest,
    output: &str,
    command_scope: &ApprovalScope,
) -> RetryDecision {
    // TODO: 后续把 capability-level `RetryPolicy` 纳入这里，避免所有 sandbox denied
    // 都只由全局 `ApprovalPolicy` 决定。
    RetryDecision::RetryWithApproval {
        reason: format!("approval required: {output}"),
        approval_scope: ApprovalScope {
            command_prefix: command_scope.command_prefix.clone(),
            cwd: command_scope.cwd.clone(),
            sandbox_profile: SandboxProfile::NoSandbox,
            network_policy: request.network_policy,
            persistence: ApprovalPersistence::Once,
        },
    }
}

fn decide_network_retry(
    request: &CommandRequest,
    output: &str,
    network_context: &NetworkApprovalContext,
    command_scope: &ApprovalScope,
) -> RetryDecision {
    match request.network_policy {
        NetworkPolicy::Deny => RetryDecision::DoNotRetry {
            reason: format!("network denied: {output} for network context {network_context:?}"),
        },
        NetworkPolicy::Prompt => RetryDecision::RetryWithApproval {
            reason: format!("network prompt: {output} for network context {network_context:?}"),
            approval_scope: ApprovalScope {
                command_prefix: command_scope.command_prefix.clone(),
                cwd: command_scope.cwd.clone(),
                sandbox_profile: SandboxProfile::NoSandbox,
                network_policy: request.network_policy,
                persistence: ApprovalPersistence::Once,
            },
        },
        NetworkPolicy::Allow => RetryDecision::RetryWithoutApproval {
            reason: format!("network allowed: {output} for network context {network_context:?}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::capability::CapabilityKind;
    use std::path::PathBuf;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| item.to_string()).collect()
    }

    fn request(approval_policy: ApprovalPolicy, network_policy: NetworkPolicy) -> CommandRequest {
        CommandRequest {
            raw_command: "npm install vite".to_string(),
            argv: strings(&["npm", "install", "vite"]),
            cwd: PathBuf::from("/workspace"),
            capability: CapabilityKind::NetworkInstall,
            approval_policy,
            sandbox_profile: SandboxProfile::WorkspaceWrite,
            network_policy,
            justification: None,
        }
    }

    fn command_scope(network_policy: NetworkPolicy) -> ApprovalScope {
        ApprovalScope {
            command_prefix: strings(&["npm", "install"]),
            cwd: PathBuf::from("/workspace"),
            sandbox_profile: SandboxProfile::WorkspaceWrite,
            network_policy,
            persistence: ApprovalPersistence::Once,
        }
    }

    fn command_failed() -> ExecutionFailure {
        ExecutionFailure::CommandFailed {
            exit_code: 1,
            stderr: "test failed".to_string(),
        }
    }

    fn sandbox_denied_with_network() -> ExecutionFailure {
        ExecutionFailure::SandboxDenied {
            output: "network access denied by sandbox".to_string(),
            network_context: Some(NetworkApprovalContext {
                host: "registry.npmjs.org".to_string(),
                protocol: "https".to_string(),
            }),
        }
    }

    fn sandbox_denied_without_network() -> ExecutionFailure {
        ExecutionFailure::SandboxDenied {
            output: "filesystem write denied by sandbox".to_string(),
            network_context: None,
        }
    }

    #[test]
    fn command_failed_does_not_retry() {
        let request = request(ApprovalPolicy::OnFailure, NetworkPolicy::Prompt);
        let scope = command_scope(NetworkPolicy::Prompt);

        let decision = decide_retry(&request, &command_failed(), &scope, false);

        match decision {
            RetryDecision::DoNotRetry { reason } => {
                assert!(reason.contains("command failed"));
                assert!(reason.contains("exit code 1"));
            }
            other => panic!("expected no retry for command failure, got {other:?}"),
        }
    }

    #[test]
    fn already_retried_does_not_retry_again() {
        let request = request(ApprovalPolicy::OnFailure, NetworkPolicy::Prompt);
        let scope = command_scope(NetworkPolicy::Prompt);
        let failure = sandbox_denied_with_network();

        let decision = decide_retry(&request, &failure, &scope, true);

        match decision {
            RetryDecision::DoNotRetry { reason } => {
                assert!(reason.contains("already retried"));
                assert!(reason.contains("SandboxDenied"));
            }
            other => panic!("expected no retry after previous retry, got {other:?}"),
        }
    }

    #[test]
    fn sandbox_denied_with_never_policy_does_not_retry() {
        let request = request(ApprovalPolicy::Never, NetworkPolicy::Prompt);
        let scope = command_scope(NetworkPolicy::Prompt);
        let failure = sandbox_denied_with_network();

        let decision = decide_retry(&request, &failure, &scope, false);

        match decision {
            RetryDecision::DoNotRetry { reason } => {
                assert!(reason.contains("approval policy is never"));
                assert!(reason.contains("network access denied"));
            }
            other => panic!("expected no retry for never policy, got {other:?}"),
        }
    }

    #[test]
    fn sandbox_denied_with_on_request_policy_does_not_auto_retry() {
        let request = request(ApprovalPolicy::OnRequest, NetworkPolicy::Prompt);
        let scope = command_scope(NetworkPolicy::Prompt);
        let failure = sandbox_denied_with_network();

        let decision = decide_retry(&request, &failure, &scope, false);

        match decision {
            RetryDecision::DoNotRetry { reason } => {
                assert!(reason.contains("approval policy is on-request"));
                assert!(reason.contains("cannot trigger automatic retry approval"));
            }
            other => panic!("expected no automatic retry for on-request policy, got {other:?}"),
        }
    }

    #[test]
    fn sandbox_denied_with_network_prompt_retries_with_no_sandbox_approval() {
        let request = request(ApprovalPolicy::OnFailure, NetworkPolicy::Prompt);
        let scope = command_scope(NetworkPolicy::Prompt);
        let failure = sandbox_denied_with_network();

        let decision = decide_retry(&request, &failure, &scope, false);

        match decision {
            RetryDecision::RetryWithApproval {
                reason,
                approval_scope,
            } => {
                assert!(reason.contains("network prompt"));
                assert!(reason.contains("registry.npmjs.org"));
                assert_eq!(approval_scope.command_prefix, strings(&["npm", "install"]));
                assert_eq!(approval_scope.cwd, PathBuf::from("/workspace"));
                assert_eq!(approval_scope.sandbox_profile, SandboxProfile::NoSandbox);
                assert_eq!(approval_scope.network_policy, NetworkPolicy::Prompt);
                assert_eq!(approval_scope.persistence, ApprovalPersistence::Once);
            }
            other => panic!("expected retry with approval, got {other:?}"),
        }
    }

    #[test]
    fn sandbox_denied_with_network_allow_retries_without_approval() {
        let request = request(ApprovalPolicy::OnFailure, NetworkPolicy::Allow);
        let scope = command_scope(NetworkPolicy::Allow);
        let failure = sandbox_denied_with_network();

        let decision = decide_retry(&request, &failure, &scope, false);

        match decision {
            RetryDecision::RetryWithoutApproval { reason } => {
                assert!(reason.contains("network allowed"));
                assert!(reason.contains("registry.npmjs.org"));
            }
            other => panic!("expected retry without approval, got {other:?}"),
        }
    }

    #[test]
    fn sandbox_denied_with_network_deny_does_not_retry() {
        let request = request(ApprovalPolicy::OnFailure, NetworkPolicy::Deny);
        let scope = command_scope(NetworkPolicy::Deny);
        let failure = sandbox_denied_with_network();

        let decision = decide_retry(&request, &failure, &scope, false);

        match decision {
            RetryDecision::DoNotRetry { reason } => {
                assert!(reason.contains("network denied"));
                assert!(reason.contains("registry.npmjs.org"));
            }
            other => panic!("expected no retry for denied network, got {other:?}"),
        }
    }

    #[test]
    fn sandbox_denied_without_network_retries_with_no_sandbox_approval() {
        let request = request(ApprovalPolicy::OnFailure, NetworkPolicy::Deny);
        let scope = command_scope(NetworkPolicy::Deny);
        let failure = sandbox_denied_without_network();

        let decision = decide_retry(&request, &failure, &scope, false);

        match decision {
            RetryDecision::RetryWithApproval {
                reason,
                approval_scope,
            } => {
                assert!(reason.contains("approval required"));
                assert!(reason.contains("filesystem write denied"));
                assert_eq!(approval_scope.command_prefix, strings(&["npm", "install"]));
                assert_eq!(approval_scope.cwd, PathBuf::from("/workspace"));
                assert_eq!(approval_scope.sandbox_profile, SandboxProfile::NoSandbox);
                assert_eq!(approval_scope.network_policy, NetworkPolicy::Deny);
                assert_eq!(approval_scope.persistence, ApprovalPersistence::Once);
            }
            other => panic!("expected retry with approval, got {other:?}"),
        }
    }
}
