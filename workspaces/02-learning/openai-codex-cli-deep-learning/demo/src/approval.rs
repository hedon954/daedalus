use crate::{
    model::{
        approval::{ApprovalPersistence, ApprovalPolicy, ApprovalRequirement, ApprovalScope},
        capability::DefaultDecision,
        command_request::CommandRequest,
    },
    registry::MatchedCapability,
};

pub fn decide_approval(
    request: &CommandRequest,
    matched_capability: &MatchedCapability,
) -> ApprovalRequirement {
    if request.capability != matched_capability.capability.kind {
        return ApprovalRequirement::Forbidden {
            reason: format!(
                "request capability {:?} does not match matched capability {:?}",
                request.capability, matched_capability.capability.kind
            ),
        };
    }

    match matched_capability.capability.policy.default_decision {
        DefaultDecision::Allow => ApprovalRequirement::Skip {
            bypass_sandbox: false, // 暂时默认不绕过沙箱
            reason: format!(
                "capability {} is allowed by default and will run in sandbox",
                matched_capability.capability.name
            ),
        },
        DefaultDecision::Prompt => match request.approval_policy {
            ApprovalPolicy::Never => ApprovalRequirement::Forbidden {
                reason: format!(
                    "capability {} is decided to prompt but the approval policy is never, so it is forbidden",
                    matched_capability.capability.name
                ),
            },
            ApprovalPolicy::OnRequest | ApprovalPolicy::OnFailure => {
                ApprovalRequirement::NeedsApproval {
                    reason: format!(
                        "capability {} requires approval before execution",
                        matched_capability.capability.name
                    ),
                    approval_scope: ApprovalScope {
                        command_prefix: matched_capability.command_prefix.clone(),
                        cwd: request.cwd.clone(),
                        sandbox_profile: request.sandbox_profile,
                        network_policy: request.network_policy,
                        persistence: ApprovalPersistence::Once,
                    },
                }
            }
        },
        DefaultDecision::Forbidden => ApprovalRequirement::Forbidden {
            reason: format!(
                "capability {} is forbidden by policy",
                matched_capability.capability.name
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        approval::{ApprovalPolicy, NetworkPolicy, SandboxProfile},
        capability::CapabilityKind,
        command_request::CommandRequest,
    };
    use crate::registry::CapabilityRegistry;
    use std::path::PathBuf;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| item.to_string()).collect()
    }

    fn matched(argv: &[&str]) -> MatchedCapability {
        CapabilityRegistry::new()
            .match_capability(&strings(argv))
            .expect("test command should match a capability")
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

    #[test]
    fn safe_read_skips_approval_but_not_sandbox() {
        let argv = ["cat", "package.json"];
        let request = request(
            &argv,
            CapabilityKind::SafeRead,
            ApprovalPolicy::OnRequest,
            SandboxProfile::ReadOnly,
            NetworkPolicy::Deny,
        );

        let decision = decide_approval(&request, &matched(&argv));

        match decision {
            ApprovalRequirement::Skip {
                bypass_sandbox,
                reason,
            } => {
                assert!(!bypass_sandbox);
                assert!(reason.contains("safe-read"));
                assert!(reason.contains("sandbox"));
            }
            other => panic!("expected skip approval, got {other:?}"),
        }
    }

    #[test]
    fn safe_test_reason_uses_matched_capability_name() {
        let argv = ["npm", "test", "--", "-u"];
        let request = request(
            &argv,
            CapabilityKind::SafeTest,
            ApprovalPolicy::OnRequest,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Deny,
        );

        let decision = decide_approval(&request, &matched(&argv));

        match decision {
            ApprovalRequirement::Skip { reason, .. } => {
                assert!(reason.contains("safe-test"));
                assert!(!reason.contains("safe-read"));
            }
            other => panic!("expected skip approval, got {other:?}"),
        }
    }

    #[test]
    fn network_install_needs_approval_with_scope_from_matched_prefix_and_request_context() {
        let argv = ["npm", "install", "vite"];
        let request = request(
            &argv,
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnRequest,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Prompt,
        );

        let decision = decide_approval(&request, &matched(&argv));

        match decision {
            ApprovalRequirement::NeedsApproval {
                reason,
                approval_scope,
            } => {
                assert!(reason.contains("network-install"));
                assert_eq!(approval_scope.command_prefix, strings(&["npm", "install"]));
                assert_eq!(approval_scope.cwd, PathBuf::from("/workspace"));
                assert_eq!(
                    approval_scope.sandbox_profile,
                    SandboxProfile::WorkspaceWrite
                );
                assert_eq!(approval_scope.network_policy, NetworkPolicy::Prompt);
                assert_eq!(approval_scope.persistence, ApprovalPersistence::Once);
            }
            other => panic!("expected approval request, got {other:?}"),
        }
    }

    #[test]
    fn prompt_policy_cannot_request_approval_when_global_policy_is_never() {
        let argv = ["npm", "install", "vite"];
        let request = request(
            &argv,
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::Never,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Prompt,
        );

        let decision = decide_approval(&request, &matched(&argv));

        match decision {
            ApprovalRequirement::Forbidden { reason } => {
                assert!(reason.contains("approval"));
                assert!(reason.contains("never"));
            }
            other => panic!("expected forbidden decision, got {other:?}"),
        }
    }

    #[test]
    fn dangerous_shell_is_forbidden_without_requesting_approval() {
        let argv = ["curl", "|", "sh"];
        let request = request(
            &argv,
            CapabilityKind::DangerousShell,
            ApprovalPolicy::OnRequest,
            SandboxProfile::ReadOnly,
            NetworkPolicy::Deny,
        );

        let decision = decide_approval(&request, &matched(&argv));

        match decision {
            ApprovalRequirement::Forbidden { reason } => {
                assert!(reason.contains("dangerous-shell"));
                assert!(reason.contains("forbidden"));
            }
            other => panic!("expected forbidden decision, got {other:?}"),
        }
    }

    #[test]
    fn approval_scope_uses_matched_prefix_not_full_argv() {
        let argv = ["npm", "install", "vite", "--save-dev"];
        let request = request(
            &argv,
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnRequest,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Prompt,
        );

        let decision = decide_approval(&request, &matched(&argv));

        match decision {
            ApprovalRequirement::NeedsApproval { approval_scope, .. } => {
                assert_eq!(approval_scope.command_prefix, strings(&["npm", "install"]));
                assert_ne!(approval_scope.command_prefix, request.argv);
            }
            other => panic!("expected approval request, got {other:?}"),
        }
    }

    #[test]
    fn approval_scope_preserves_network_policy_from_request() {
        let argv = ["npm", "install", "vite"];
        let request = request(
            &argv,
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnRequest,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Allow,
        );

        let decision = decide_approval(&request, &matched(&argv));

        match decision {
            ApprovalRequirement::NeedsApproval { approval_scope, .. } => {
                assert_eq!(approval_scope.network_policy, NetworkPolicy::Allow);
            }
            other => panic!("expected approval request, got {other:?}"),
        }
    }

    #[test]
    fn request_capability_must_match_matched_capability() {
        let argv = ["cat", "package.json"];
        let request = request(
            &argv,
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnRequest,
            SandboxProfile::ReadOnly,
            NetworkPolicy::Deny,
        );

        let decision = decide_approval(&request, &matched(&argv));
        match decision {
            ApprovalRequirement::Forbidden { reason } => {
                assert!(reason.contains("does not match"));
            }
            other => panic!("expected forbidden decision, got {other:?}"),
        }
    }
}
