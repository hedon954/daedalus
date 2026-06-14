use async_trait::async_trait;

use crate::{
    model::{
        approval::SandboxProfile,
        command_request::CommandRequest,
        event::ExecutionAttempt,
        execution::{ExecutionFailure, ExecutionResult, NetworkApprovalContext},
    },
    tool::shell::execution::ExecutionRunner,
    util::args_has_prefix,
};

/// 模拟运行器。
///
/// 它不执行真实命令，只根据 `request.argv + ExecutionAttempt` 查规则表。
/// 这样 Phase 1 可以先验证 approval / sandbox / retry 状态机，而不被 OS sandbox 细节拖住。
pub struct SimulatedExecutionRunner {
    rules: Vec<SimulatedRule>,
}

/// 一条模拟执行规则。
///
/// `command_prefix + attempt` 同时命中才会返回指定结果。
#[derive(Debug, Clone)]
struct SimulatedRule {
    command_prefix: Vec<String>,
    attempt: SimulatedAttempt,
    result: SimulatedResult,
}

/// 模拟 runner 关心的尝试类型。
///
/// `NoSandboxFirst` 和 `NoSandboxRetry` 都折叠成 `NoSandbox`，因为 Phase 1
/// 只需要验证是否离开 sandbox，不区分具体来源。
#[derive(Debug, Clone, PartialEq, Eq)]
enum SimulatedAttempt {
    Sandbox(SandboxProfile),
    NoSandbox,
}

/// 模拟 runner 可返回的执行结果。
#[derive(Debug, Clone)]
enum SimulatedResult {
    Success {
        stdout: String,
    },
    CommandFailed {
        exit_code: i32,
        stderr: String,
    },
    SandboxDenied {
        output: String,
        network_context: Option<NetworkApprovalContext>,
    },
}

impl SimulatedExecutionRunner {
    /// 创建内置规则集。
    ///
    /// TODO: 后续可以把 rules 暴露为 test builder，让 `run_shell_command`
    /// 的整合测试更容易构造指定场景。
    pub fn new() -> Self {
        Self {
            rules: vec![
                // rule1: read with sandbox success
                SimulatedRule {
                    command_prefix: vec!["cat".into()],
                    attempt: SimulatedAttempt::Sandbox(SandboxProfile::ReadOnly),
                    result: SimulatedResult::Success {
                        stdout: "simulated read success".into(),
                    },
                },
                // rule2: test with sandbox failed
                SimulatedRule {
                    command_prefix: vec!["npm".into(), "test".into(), "--".into(), "fail".into()],
                    attempt: SimulatedAttempt::Sandbox(SandboxProfile::WorkspaceWrite),
                    result: SimulatedResult::CommandFailed {
                        exit_code: 1,
                        stderr: "simulated command failed".into(),
                    },
                },
                // rule3: install with sandbox denied
                SimulatedRule {
                    command_prefix: vec!["npm".into(), "install".into()],
                    attempt: SimulatedAttempt::Sandbox(SandboxProfile::WorkspaceWrite),
                    result: SimulatedResult::SandboxDenied {
                        output: "network access denied by simulated sandbox".into(),
                        network_context: Some(NetworkApprovalContext {
                            host: "registry.npmjs.org".into(),
                            protocol: "https".into(),
                        }),
                    },
                },
                // rule4: install without sandbox success
                SimulatedRule {
                    command_prefix: vec!["npm".into(), "install".into()],
                    attempt: SimulatedAttempt::NoSandbox,
                    result: SimulatedResult::Success {
                        stdout: "simulated install success without sandbox".into(),
                    },
                },
                // rule5: approval UX smoke test with sandbox success
                SimulatedRule {
                    command_prefix: vec!["echo".into(), "approval-test".into()],
                    attempt: SimulatedAttempt::Sandbox(SandboxProfile::ReadOnly),
                    result: SimulatedResult::Success {
                        stdout: "approval-test".into(),
                    },
                },
            ],
        }
    }
}

impl Default for SimulatedExecutionRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ExecutionRunner for SimulatedExecutionRunner {
    /// 根据命令前缀和 attempt 查找模拟结果。
    async fn run(&self, request: &CommandRequest, attempt: &ExecutionAttempt) -> ExecutionResult {
        let attempt = SimulatedAttempt::from(attempt);
        let rule = self.rules.iter().find(|rule| {
            args_has_prefix(&request.argv, &rule.command_prefix) && rule.attempt == attempt
        });
        match rule {
            None => ExecutionResult::Failure(ExecutionFailure::CommandFailed {
                exit_code: 127,
                stderr: format!(
                    "unsupported simulated command: `{}` for attempt `{}`; no SimulatedRule matched argv prefix",
                    request.raw_command,
                    attempt.label(),
                ),
            }),
            Some(rule) => rule.result.render(request),
        }
    }
}

impl From<&ExecutionAttempt> for SimulatedAttempt {
    /// 将完整 execution attempt 映射为模拟 runner 的简化维度。
    fn from(attempt: &ExecutionAttempt) -> Self {
        match attempt {
            ExecutionAttempt::SandboxFirst { sandbox_profile } => {
                SimulatedAttempt::Sandbox(*sandbox_profile)
            }
            ExecutionAttempt::NoSandboxFirst { .. } | ExecutionAttempt::NoSandboxRetry { .. } => {
                SimulatedAttempt::NoSandbox
            }
        }
    }
}

impl SimulatedAttempt {
    /// 生成用于错误信息的 attempt 标签。
    fn label(&self) -> String {
        match self {
            SimulatedAttempt::Sandbox(profile) => format!("sandbox:{profile:?}"),
            SimulatedAttempt::NoSandbox => "no-sandbox".to_string(),
        }
    }
}

impl SimulatedResult {
    /// 将模拟结果渲染为真实 runtime 使用的 `ExecutionResult`。
    fn render(&self, request: &CommandRequest) -> ExecutionResult {
        match self {
            SimulatedResult::Success { stdout } => ExecutionResult::Success {
                stdout: format!("{stdout}. raw_command: {}", request.raw_command),
            },
            SimulatedResult::CommandFailed { exit_code, stderr } => {
                ExecutionResult::Failure(ExecutionFailure::CommandFailed {
                    exit_code: *exit_code,
                    stderr: format!("{stderr}. raw_command: {}", request.raw_command),
                })
            }
            SimulatedResult::SandboxDenied {
                output,
                network_context,
            } => ExecutionResult::Failure(ExecutionFailure::SandboxDenied {
                output: format!("{output}. raw_command: {}", request.raw_command),
                network_context: network_context.clone(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        approval::{ApprovalPolicy, NetworkPolicy},
        capability::CapabilityKind,
    };
    use std::path::PathBuf;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| item.to_string()).collect()
    }

    fn request(
        argv: &[&str],
        capability: CapabilityKind,
        sandbox_profile: SandboxProfile,
        network_policy: NetworkPolicy,
    ) -> CommandRequest {
        CommandRequest {
            raw_command: argv.join(" "),
            argv: strings(argv),
            cwd: PathBuf::from("/workspace"),
            capability,
            approval_policy: ApprovalPolicy::OnRequest,
            sandbox_profile,
            network_policy,
            justification: None,
        }
    }

    #[tokio::test]
    async fn read_command_succeeds_in_read_only_sandbox() {
        let runner = SimulatedExecutionRunner::new();
        let request = request(
            &["cat", "package.json"],
            CapabilityKind::SafeRead,
            SandboxProfile::ReadOnly,
            NetworkPolicy::Deny,
        );
        let attempt = ExecutionAttempt::SandboxFirst {
            sandbox_profile: SandboxProfile::ReadOnly,
        };

        let result = runner.run(&request, &attempt).await;

        match result {
            ExecutionResult::Success { .. } => {}
            other => panic!("expected simulated success, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_command_failure_is_command_failed_not_sandbox_denied() {
        let runner = SimulatedExecutionRunner::new();
        let request = request(
            &["npm", "test", "--", "fail"],
            CapabilityKind::SafeTest,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Deny,
        );
        let attempt = ExecutionAttempt::SandboxFirst {
            sandbox_profile: SandboxProfile::WorkspaceWrite,
        };

        let result = runner.run(&request, &attempt).await;

        match result {
            ExecutionResult::Failure(ExecutionFailure::CommandFailed { exit_code, .. }) => {
                assert_eq!(exit_code, 1);
            }
            other => panic!("expected command failure, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn install_command_is_sandbox_denied_in_workspace_write_sandbox() {
        let runner = SimulatedExecutionRunner::new();
        let request = request(
            &["npm", "install", "vite"],
            CapabilityKind::NetworkInstall,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Prompt,
        );
        let attempt = ExecutionAttempt::SandboxFirst {
            sandbox_profile: SandboxProfile::WorkspaceWrite,
        };

        let result = runner.run(&request, &attempt).await;

        match result {
            ExecutionResult::Failure(ExecutionFailure::SandboxDenied {
                output: _,
                network_context,
            }) => {
                assert_eq!(
                    network_context,
                    Some(NetworkApprovalContext {
                        host: "registry.npmjs.org".to_string(),
                        protocol: "https".to_string(),
                    })
                );
            }
            other => panic!("expected sandbox denied, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn install_command_succeeds_without_sandbox_on_retry() {
        let runner = SimulatedExecutionRunner::new();
        let request = request(
            &["npm", "install", "vite"],
            CapabilityKind::NetworkInstall,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Prompt,
        );
        let attempt = ExecutionAttempt::NoSandboxRetry {
            reason: "approved retry outside sandbox".to_string(),
        };

        let result = runner.run(&request, &attempt).await;

        match result {
            ExecutionResult::Success { .. } => {}
            other => panic!("expected no-sandbox success, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn approval_test_command_succeeds_in_read_only_sandbox() {
        let runner = SimulatedExecutionRunner::new();
        let request = request(
            &["echo", "approval-test"],
            CapabilityKind::ApprovalTest,
            SandboxProfile::ReadOnly,
            NetworkPolicy::Deny,
        );
        let attempt = ExecutionAttempt::SandboxFirst {
            sandbox_profile: SandboxProfile::ReadOnly,
        };

        let result = runner.run(&request, &attempt).await;

        match result {
            ExecutionResult::Success { .. } => {}
            other => panic!("expected approval test success, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn unsupported_command_attempt_returns_command_failed() {
        let runner = SimulatedExecutionRunner::new();
        let request = request(
            &["cargo", "test"],
            CapabilityKind::SafeTest,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Deny,
        );
        let attempt = ExecutionAttempt::SandboxFirst {
            sandbox_profile: SandboxProfile::WorkspaceWrite,
        };

        let result = runner.run(&request, &attempt).await;

        match result {
            ExecutionResult::Failure(ExecutionFailure::CommandFailed { exit_code, .. }) => {
                assert_eq!(exit_code, 127);
            }
            other => panic!("expected unsupported command failure, got {other:?}"),
        }
    }
}
