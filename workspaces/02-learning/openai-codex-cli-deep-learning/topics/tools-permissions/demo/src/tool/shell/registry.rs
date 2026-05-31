use crate::{
    model::{
        approval::{NetworkPolicy, SandboxProfile},
        capability::{
            CapabilityDescriptor, CapabilityKind, CapabilityPolicy, DefaultDecision, RetryPolicy,
        },
    },
    util::args_has_prefix,
};

/// 能力注册表。
///
/// 负责把 host 支持的命令前缀映射为 capability。它是 command runtime
/// 的第一道边界：没有匹配到 capability 的命令不会进入 approval / sandbox。
pub struct CapabilityRegistry {
    /// 内置能力
    builtin_capabilities: Vec<CapabilityDescriptor>,
    /// 注册的能力
    capabilities: Vec<CapabilityDescriptor>,
}

/// 匹配到的能力。
///
/// `command_prefix` 保留本次命中的具体前缀，用于后续构造 approval scope。
pub struct MatchedCapability {
    /// 能力描述
    pub capability: CapabilityDescriptor,
    /// 命令前缀
    pub command_prefix: Vec<String>,
}

impl CapabilityRegistry {
    /// 创建包含内置能力的 registry。
    pub fn new() -> Self {
        Self {
            builtin_capabilities: vec![
                CapabilityDescriptor {
                    kind: CapabilityKind::SafeRead,
                    name: "safe-read".to_string(),
                    description: "Safe read command".to_string(),
                    command_prefixes: vec![vec!["ls".to_string()], vec!["cat".to_string()]],
                    policy: CapabilityPolicy {
                        default_decision: DefaultDecision::Allow,
                        first_attempt_sandbox: SandboxProfile::ReadOnly,
                        network_policy: NetworkPolicy::Deny,
                        retry_policy: RetryPolicy::Never,
                    },
                },
                CapabilityDescriptor {
                    kind: CapabilityKind::SafeTest,
                    name: "safe-test".to_string(),
                    description: "Safe test command".to_string(),
                    command_prefixes: vec![vec!["npm".to_string(), "test".to_string()]],
                    policy: CapabilityPolicy {
                        default_decision: DefaultDecision::Allow,
                        first_attempt_sandbox: SandboxProfile::WorkspaceWrite,
                        network_policy: NetworkPolicy::Deny,
                        retry_policy: RetryPolicy::WithApproval,
                    },
                },
                CapabilityDescriptor {
                    kind: CapabilityKind::NetworkInstall,
                    name: "network-install".to_string(),
                    description: "Network install command".to_string(),
                    command_prefixes: vec![vec!["npm".to_string(), "install".to_string()]],
                    policy: CapabilityPolicy {
                        default_decision: DefaultDecision::Prompt,
                        first_attempt_sandbox: SandboxProfile::WorkspaceWrite,
                        network_policy: NetworkPolicy::Prompt,
                        retry_policy: RetryPolicy::WithApproval,
                    },
                },
                CapabilityDescriptor {
                    kind: CapabilityKind::DangerousShell,
                    name: "dangerous-shell".to_string(),
                    description: "Dangerous shell command".to_string(),
                    command_prefixes: vec![vec![
                        "curl".to_string(),
                        "|".to_string(),
                        "sh".to_string(),
                    ]],
                    policy: CapabilityPolicy {
                        default_decision: DefaultDecision::Forbidden,
                        first_attempt_sandbox: SandboxProfile::ReadOnly,
                        network_policy: NetworkPolicy::Deny,
                        retry_policy: RetryPolicy::Never,
                    },
                },
            ],
            capabilities: vec![],
        }
    }

    /// 注册能力
    pub fn register(&mut self, capability: CapabilityDescriptor) {
        self.capabilities.push(capability);
    }

    /// 加载内置能力
    pub fn load_builtin_capabilities(&self) -> Vec<CapabilityDescriptor> {
        self.builtin_capabilities.clone()
    }

    /// 加载所有能力
    pub fn all_capabilities(&self) -> Vec<CapabilityDescriptor> {
        self.builtin_capabilities
            .iter()
            .cloned()
            .chain(self.capabilities.iter().cloned())
            .collect()
    }

    /// 根据 argv 匹配 capability。
    ///
    /// TODO: 当前只处理单个 argv；后续多命令阶段需要对 command segments 分别匹配，
    /// 再按最严格策略合成总决策。
    pub fn match_capability(&self, argv: &[String]) -> Option<MatchedCapability> {
        self.builtin_capabilities
            .iter()
            .chain(self.capabilities.iter())
            .find_map(|cd| {
                cd.command_prefixes
                    .iter()
                    .find(|prefix| args_has_prefix(argv, prefix))
                    .map(|matched_prefix| MatchedCapability {
                        capability: cd.clone(),
                        command_prefix: matched_prefix.clone(),
                    })
            })
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_capabilities_should_contains_all_kinds_of_capabilities() {
        let registry = CapabilityRegistry::new();
        let capabilities = registry.load_builtin_capabilities();
        assert!(capabilities.len() >= 4);
        assert!(
            capabilities
                .iter()
                .any(|c| c.kind == CapabilityKind::SafeRead
                    && c.policy.default_decision == DefaultDecision::Allow
                    && c.policy.first_attempt_sandbox == SandboxProfile::ReadOnly
                    && c.policy.network_policy == NetworkPolicy::Deny
                    && c.policy.retry_policy == RetryPolicy::Never)
        );
        assert!(
            capabilities
                .iter()
                .any(|c| c.kind == CapabilityKind::SafeTest
                    && c.policy.default_decision == DefaultDecision::Allow
                    && c.policy.first_attempt_sandbox == SandboxProfile::WorkspaceWrite
                    && c.policy.network_policy == NetworkPolicy::Deny
                    && c.policy.retry_policy == RetryPolicy::WithApproval)
        );
        assert!(
            capabilities
                .iter()
                .any(|c| c.kind == CapabilityKind::NetworkInstall
                    && c.policy.default_decision == DefaultDecision::Prompt
                    && c.policy.first_attempt_sandbox == SandboxProfile::WorkspaceWrite
                    && c.policy.network_policy == NetworkPolicy::Prompt
                    && c.policy.retry_policy == RetryPolicy::WithApproval)
        );
        assert!(
            capabilities
                .iter()
                .any(|c| c.kind == CapabilityKind::DangerousShell
                    && c.policy.default_decision == DefaultDecision::Forbidden
                    && c.policy.first_attempt_sandbox == SandboxProfile::ReadOnly
                    && c.policy.network_policy == NetworkPolicy::Deny
                    && c.policy.retry_policy == RetryPolicy::Never)
        );
    }

    #[test]
    fn match_capability_should_work() {
        let test_cases = vec![
            (vec!["cat", "package.json"], Some(CapabilityKind::SafeRead)),
            (
                vec!["npm", "test", "--", "-u"],
                Some(CapabilityKind::SafeTest),
            ),
            (
                vec!["npm", "install", "vite"],
                Some(CapabilityKind::NetworkInstall),
            ),
            (vec!["unknown"], None),
        ];
        let registry = CapabilityRegistry::new();
        for (argv, expected_kind) in test_cases {
            let argv: Vec<String> = argv.into_iter().map(|s| s.to_string()).collect();
            let capability = registry.match_capability(&argv);
            assert_eq!(capability.map(|c| c.capability.kind), expected_kind);
        }
    }
}
