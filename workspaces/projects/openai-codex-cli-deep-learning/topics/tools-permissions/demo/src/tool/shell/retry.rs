use crate::model::{
    approval::{ApprovalPersistence, ApprovalPolicy, ApprovalScope, NetworkPolicy, SandboxProfile},
    capability::RetryPolicy,
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
    retry_policy: RetryPolicy,
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
        } => match retry_policy {
            RetryPolicy::Never => RetryDecision::DoNotRetry {
                reason: format!("sandbox denied: {output} and retry policy is never"),
            },
            RetryPolicy::WithApproval => match request.approval_policy {
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
            RetryPolicy::WithoutApproval => {
                if let Some(network_context) = network_context {
                    decide_network_retry(request, output, network_context, command_scope)
                } else {
                    RetryDecision::RetryWithoutApproval {
                        reason: format!(
                            "sandbox denied: {output} but retry policy is without approval, so retry directly"
                        ),
                    }
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
        NetworkPolicy::Prompt => match request.approval_policy {
            ApprovalPolicy::OnFailure => RetryDecision::RetryWithApproval {
                reason: format!("network prompt: {output} for network context {network_context:?}"),
                approval_scope: ApprovalScope {
                    command_prefix: command_scope.command_prefix.clone(),
                    cwd: command_scope.cwd.clone(),
                    sandbox_profile: SandboxProfile::NoSandbox,
                    network_policy: request.network_policy,
                    persistence: ApprovalPersistence::Once,
                },
            },
            ApprovalPolicy::Never | ApprovalPolicy::OnRequest => RetryDecision::DoNotRetry {
                reason: format!(
                    "network prompt requires approval, but approval policy does not allow on-failure approval: {output}"
                ),
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

    fn request(
        argv: &[&str],
        capability: CapabilityKind,
        approval_policy: ApprovalPolicy,
        network_policy: NetworkPolicy,
    ) -> CommandRequest {
        CommandRequest {
            raw_command: argv.join(" "),
            argv: strings(argv),
            cwd: PathBuf::from("/workspace"),
            capability,
            approval_policy,
            sandbox_profile: SandboxProfile::WorkspaceWrite,
            network_policy,
            justification: None,
        }
    }

    fn command_scope(command_prefix: &[&str], network_policy: NetworkPolicy) -> ApprovalScope {
        ApprovalScope {
            command_prefix: strings(command_prefix),
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

    fn assert_do_not_retry(decision: RetryDecision) {
        assert!(matches!(decision, RetryDecision::DoNotRetry { .. }));
    }

    fn assert_retry_without_approval(decision: RetryDecision) {
        assert!(matches!(
            decision,
            RetryDecision::RetryWithoutApproval { .. }
        ));
    }

    fn retry_approval_scope(decision: RetryDecision) -> ApprovalScope {
        match decision {
            RetryDecision::RetryWithApproval {
                reason: _,
                approval_scope,
            } => approval_scope,
            other => panic!("expected retry with approval, got {other:?}"),
        }
    }

    fn assert_no_sandbox_scope(
        scope: ApprovalScope,
        command_prefix: &[&str],
        network_policy: NetworkPolicy,
    ) {
        assert_eq!(scope.command_prefix, strings(command_prefix));
        assert_eq!(scope.cwd, PathBuf::from("/workspace"));
        assert_eq!(scope.sandbox_profile, SandboxProfile::NoSandbox);
        assert_eq!(scope.network_policy, network_policy);
        assert_eq!(scope.persistence, ApprovalPersistence::Once);
    }

    #[test]
    fn command_failed_does_not_retry() {
        let request = request(
            &["npm", "test"],
            CapabilityKind::SafeTest,
            ApprovalPolicy::OnFailure,
            NetworkPolicy::Deny,
        );
        let scope = command_scope(&["npm", "test"], NetworkPolicy::Deny);

        let decision = decide_retry(
            &request,
            &command_failed(),
            &scope,
            RetryPolicy::WithApproval,
            false,
        );

        assert_do_not_retry(decision);
    }

    #[test]
    fn already_retried_does_not_retry_again() {
        let request = request(
            &["npm", "install", "vite"],
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnFailure,
            NetworkPolicy::Allow,
        );
        let scope = command_scope(&["npm", "install"], NetworkPolicy::Allow);
        let failure = sandbox_denied_with_network();

        let decision = decide_retry(
            &request,
            &failure,
            &scope,
            RetryPolicy::WithoutApproval,
            true,
        );

        assert_do_not_retry(decision);
    }

    #[test]
    fn safe_read_retry_policy_never_does_not_retry_after_sandbox_denied() {
        let request = request(
            &["cat", "/private/file"],
            CapabilityKind::SafeRead,
            ApprovalPolicy::OnFailure,
            NetworkPolicy::Deny,
        );
        let scope = command_scope(&["cat"], NetworkPolicy::Deny);
        let failure = sandbox_denied_without_network();

        let decision = decide_retry(&request, &failure, &scope, RetryPolicy::Never, false);

        assert_do_not_retry(decision);
    }

    #[test]
    fn retry_policy_never_wins_before_network_policy() {
        let request = request(
            &["npm", "install", "vite"],
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnFailure,
            NetworkPolicy::Allow,
        );
        let scope = command_scope(&["npm", "install"], NetworkPolicy::Allow);
        let failure = sandbox_denied_with_network();

        let decision = decide_retry(&request, &failure, &scope, RetryPolicy::Never, false);

        assert_do_not_retry(decision);
    }

    #[test]
    fn sandbox_denied_with_global_never_policy_does_not_retry_when_retry_requires_approval() {
        let request = request(
            &["npm", "install", "vite"],
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::Never,
            NetworkPolicy::Prompt,
        );
        let scope = command_scope(&["npm", "install"], NetworkPolicy::Prompt);
        let failure = sandbox_denied_with_network();

        let decision = decide_retry(&request, &failure, &scope, RetryPolicy::WithApproval, false);

        assert_do_not_retry(decision);
    }

    #[test]
    fn sandbox_denied_with_on_request_policy_does_not_auto_retry() {
        let request = request(
            &["npm", "install", "vite"],
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnRequest,
            NetworkPolicy::Prompt,
        );
        let scope = command_scope(&["npm", "install"], NetworkPolicy::Prompt);
        let failure = sandbox_denied_with_network();

        let decision = decide_retry(&request, &failure, &scope, RetryPolicy::WithApproval, false);

        assert_do_not_retry(decision);
    }

    #[test]
    fn sandbox_denied_with_network_prompt_retries_with_no_sandbox_approval() {
        let request = request(
            &["npm", "install", "vite"],
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnFailure,
            NetworkPolicy::Prompt,
        );
        let scope = command_scope(&["npm", "install"], NetworkPolicy::Prompt);
        let failure = sandbox_denied_with_network();

        let decision = decide_retry(&request, &failure, &scope, RetryPolicy::WithApproval, false);

        assert_no_sandbox_scope(
            retry_approval_scope(decision),
            &["npm", "install"],
            NetworkPolicy::Prompt,
        );
    }

    #[test]
    fn sandbox_denied_with_network_allow_retries_without_approval() {
        let request = request(
            &["npm", "install", "vite"],
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnFailure,
            NetworkPolicy::Allow,
        );
        let scope = command_scope(&["npm", "install"], NetworkPolicy::Allow);
        let failure = sandbox_denied_with_network();

        let decision = decide_retry(&request, &failure, &scope, RetryPolicy::WithApproval, false);

        assert_retry_without_approval(decision);
    }

    #[test]
    fn sandbox_denied_with_network_deny_does_not_retry() {
        let request = request(
            &["npm", "install", "vite"],
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnFailure,
            NetworkPolicy::Deny,
        );
        let scope = command_scope(&["npm", "install"], NetworkPolicy::Deny);
        let failure = sandbox_denied_with_network();

        let decision = decide_retry(&request, &failure, &scope, RetryPolicy::WithApproval, false);

        assert_do_not_retry(decision);
    }

    #[test]
    fn sandbox_denied_without_network_retries_with_no_sandbox_approval() {
        let request = request(
            &["npm", "test"],
            CapabilityKind::SafeTest,
            ApprovalPolicy::OnFailure,
            NetworkPolicy::Deny,
        );
        let scope = command_scope(&["npm", "test"], NetworkPolicy::Deny);
        let failure = sandbox_denied_without_network();

        let decision = decide_retry(&request, &failure, &scope, RetryPolicy::WithApproval, false);

        assert_no_sandbox_scope(
            retry_approval_scope(decision),
            &["npm", "test"],
            NetworkPolicy::Deny,
        );
    }

    #[test]
    fn retry_policy_without_approval_allows_direct_retry_for_non_network_sandbox_denial() {
        let request = request(
            &["cargo", "test"],
            CapabilityKind::SafeTest,
            ApprovalPolicy::OnRequest,
            NetworkPolicy::Deny,
        );
        let scope = command_scope(&["cargo", "test"], NetworkPolicy::Deny);
        let failure = sandbox_denied_without_network();

        let decision = decide_retry(
            &request,
            &failure,
            &scope,
            RetryPolicy::WithoutApproval,
            false,
        );

        assert_retry_without_approval(decision);
    }

    #[test]
    fn retry_policy_without_approval_does_not_override_network_deny() {
        let request = request(
            &["npm", "install", "vite"],
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnFailure,
            NetworkPolicy::Deny,
        );
        let scope = command_scope(&["npm", "install"], NetworkPolicy::Deny);
        let failure = sandbox_denied_with_network();

        let decision = decide_retry(
            &request,
            &failure,
            &scope,
            RetryPolicy::WithoutApproval,
            false,
        );

        assert_do_not_retry(decision);
    }

    #[test]
    fn retry_policy_without_approval_still_requires_approval_for_network_prompt() {
        let request = request(
            &["npm", "install", "vite"],
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnFailure,
            NetworkPolicy::Prompt,
        );
        let scope = command_scope(&["npm", "install"], NetworkPolicy::Prompt);
        let failure = sandbox_denied_with_network();

        let decision = decide_retry(
            &request,
            &failure,
            &scope,
            RetryPolicy::WithoutApproval,
            false,
        );

        assert_no_sandbox_scope(
            retry_approval_scope(decision),
            &["npm", "install"],
            NetworkPolicy::Prompt,
        );
    }

    #[test]
    fn retry_policy_without_approval_does_not_prompt_network_when_global_policy_disallows_it() {
        let request = request(
            &["npm", "install", "vite"],
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::Never,
            NetworkPolicy::Prompt,
        );
        let scope = command_scope(&["npm", "install"], NetworkPolicy::Prompt);
        let failure = sandbox_denied_with_network();

        let decision = decide_retry(
            &request,
            &failure,
            &scope,
            RetryPolicy::WithoutApproval,
            false,
        );

        assert_do_not_retry(decision);
    }
}
