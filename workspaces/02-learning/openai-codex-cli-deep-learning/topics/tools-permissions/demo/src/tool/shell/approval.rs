use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::{mpsc, oneshot};

use crate::{
    model::{
        approval::{ApprovalPersistence, ApprovalPolicy, ApprovalRequirement, ApprovalScope},
        capability::DefaultDecision,
        command_request::CommandRequest,
        event::UserApprovalDecision,
    },
    tool::shell::registry::MatchedCapability,
};

#[derive(Debug, Clone)]
pub struct ToolApprovalRequest {
    pub reason: String,
    pub scope: ApprovalScope,
}

pub struct ToolApprovalResult {
    pub approval_id: String,
    pub decision: UserApprovalDecision,
}

pub type ToolApprovalResultReceiver = mpsc::Receiver<ToolApprovalResult>;

#[derive(Debug, Clone)]
pub struct ToolCallContext {
    pub index: i64,
    pub call_id: String,
    pub tool_name: String,
}

pub struct ApprovalGateway {
    pending: Arc<DashMap<String, oneshot::Sender<UserApprovalDecision>>>,
}

pub struct PendingApproval {
    pub approval_id: String,
    pub request: ToolApprovalRequest,
    decision_rx: oneshot::Receiver<UserApprovalDecision>,
}

impl ApprovalGateway {
    pub fn new(mut rx: ToolApprovalResultReceiver) -> ApprovalGateway {
        let pending = Arc::new(DashMap::new());

        let result = Self {
            pending: pending.clone(),
        };

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                if let Some((_, tx)) = pending.remove(&event.approval_id) {
                    _ = tx.send(event.decision);
                }
            }
        });

        result
    }

    pub fn create_pending_approval(&self, request: ToolApprovalRequest) -> PendingApproval {
        let approval_id = uuid::Uuid::new_v4().to_string();
        let (tx, rx) = oneshot::channel();
        self.pending.insert(approval_id.clone(), tx);

        PendingApproval {
            approval_id,
            request,
            decision_rx: rx,
        }
    }

    pub fn cancel(&self, approval_id: &str) {
        _ = self.pending.remove(approval_id);
    }
}

impl PendingApproval {
    pub async fn wait(self) -> UserApprovalDecision {
        self.decision_rx
            .await
            .unwrap_or(UserApprovalDecision::Rejected)
    }
}

/// 根据命令上下文和命中的 capability 计算初始审批需求。
///
/// 这个函数是 command runtime 的第一道 policy gate。它只回答
/// “能不能进入执行，以及是否要先审批”，不负责真正运行命令。
pub fn resolve_approval_requirement(
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
                    approval_scope: build_approval_scope(request, matched_capability),
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

/// 构造用户审批的作用域。
///
/// 注意 scope 使用 matched prefix，而不是完整 argv；这样可以表达
/// “本 session 允许 npm install 这一类命令”，同时仍然绑定 cwd / sandbox / network。
pub fn build_approval_scope(
    request: &CommandRequest,
    matched_capability: &MatchedCapability,
) -> ApprovalScope {
    ApprovalScope {
        command_prefix: matched_capability.command_prefix.clone(),
        cwd: request.cwd.clone(),
        sandbox_profile: request.sandbox_profile,
        network_policy: request.network_policy,
        persistence: ApprovalPersistence::Once,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{
            approval::{ApprovalPolicy, NetworkPolicy, SandboxProfile},
            capability::CapabilityKind,
            command_request::CommandRequest,
            event::UserApprovalDecision,
        },
        tool::shell::registry::CapabilityRegistry,
    };
    use std::path::PathBuf;
    use tokio::sync::mpsc;

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

    fn assert_skip_in_sandbox(decision: ApprovalRequirement) {
        match decision {
            ApprovalRequirement::Skip { bypass_sandbox, .. } => assert!(!bypass_sandbox),
            other => panic!("expected skip approval, got {other:?}"),
        }
    }

    fn assert_forbidden(decision: ApprovalRequirement) {
        match decision {
            ApprovalRequirement::Forbidden { .. } => {}
            other => panic!("expected forbidden decision, got {other:?}"),
        }
    }

    fn approval_scope(decision: ApprovalRequirement) -> ApprovalScope {
        match decision {
            ApprovalRequirement::NeedsApproval { approval_scope, .. } => approval_scope,
            other => panic!("expected approval request, got {other:?}"),
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

        let decision = resolve_approval_requirement(&request, &matched(&argv));

        assert_skip_in_sandbox(decision);
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

        let decision = resolve_approval_requirement(&request, &matched(&argv));
        let approval_scope = approval_scope(decision);

        assert_eq!(approval_scope.command_prefix, strings(&["npm", "install"]));
        assert_eq!(approval_scope.cwd, PathBuf::from("/workspace"));
        assert_eq!(
            approval_scope.sandbox_profile,
            SandboxProfile::WorkspaceWrite
        );
        assert_eq!(approval_scope.network_policy, NetworkPolicy::Prompt);
        assert_eq!(approval_scope.persistence, ApprovalPersistence::Once);
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

        let decision = resolve_approval_requirement(&request, &matched(&argv));

        assert_forbidden(decision);
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

        let decision = resolve_approval_requirement(&request, &matched(&argv));

        assert_forbidden(decision);
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

        let decision = resolve_approval_requirement(&request, &matched(&argv));
        let approval_scope = approval_scope(decision);

        assert_eq!(approval_scope.command_prefix, strings(&["npm", "install"]));
        assert_ne!(approval_scope.command_prefix, request.argv);
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

        let decision = resolve_approval_requirement(&request, &matched(&argv));
        let approval_scope = approval_scope(decision);

        assert_eq!(approval_scope.network_policy, NetworkPolicy::Allow);
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

        let decision = resolve_approval_requirement(&request, &matched(&argv));

        assert_forbidden(decision);
    }

    #[tokio::test]
    async fn approval_gateway_resolves_matching_internal_result() {
        let (result_tx, result_rx) = mpsc::channel(4);
        let gateway = ApprovalGateway::new(result_rx);
        let expected_reason = "network install requires approval";
        let scope = ApprovalScope {
            command_prefix: strings(&["npm", "install"]),
            cwd: PathBuf::from("/workspace"),
            sandbox_profile: SandboxProfile::WorkspaceWrite,
            network_policy: NetworkPolicy::Prompt,
            persistence: ApprovalPersistence::Once,
        };

        let pending = gateway.create_pending_approval(ToolApprovalRequest {
            reason: expected_reason.to_string(),
            scope,
        });
        let approval_id = pending.approval_id.clone();

        let approval_task = tokio::spawn(async move {
            assert_eq!(pending.request.reason, expected_reason);
            assert_eq!(
                pending.request.scope,
                ApprovalScope {
                    command_prefix: strings(&["npm", "install"]),
                    cwd: PathBuf::from("/workspace"),
                    sandbox_profile: SandboxProfile::WorkspaceWrite,
                    network_policy: NetworkPolicy::Prompt,
                    persistence: ApprovalPersistence::Once,
                }
            );
            pending.wait().await
        });

        result_tx
            .send(ToolApprovalResult {
                approval_id,
                decision: UserApprovalDecision::Approved {
                    persistence: ApprovalPersistence::Session,
                },
            })
            .await
            .expect("test should send internal approval result");

        assert_eq!(
            approval_task.await.expect("approval task should finish"),
            UserApprovalDecision::Approved {
                persistence: ApprovalPersistence::Session
            }
        );
    }
}
