use crate::{
    model::{
        approval::SandboxProfile,
        command_request::CommandRequest,
        event::ExecutionAttempt,
        execution::{ExecutionFailure, ExecutionResult, NetworkApprovalContext},
    },
    sandbox::SandboxRunner,
    util::args_has_prefix,
};

/// 模拟沙箱运行器
pub struct SimulatedSandboxRunner {
    rules: Vec<SimulatedRule>,
}

#[derive(Debug, Clone)]
struct SimulatedRule {
    command_prefix: Vec<String>,
    attempt: SimulatedAttempt,
    result: SimulatedResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SimulatedAttempt {
    Sandbox(SandboxProfile),
    NoSandbox,
}

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

impl SimulatedSandboxRunner {
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
            ],
        }
    }
}

impl Default for SimulatedSandboxRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl SandboxRunner for SimulatedSandboxRunner {
    fn run(&self, request: &CommandRequest, attempt: &ExecutionAttempt) -> ExecutionResult {
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
    fn label(&self) -> String {
        match self {
            SimulatedAttempt::Sandbox(profile) => format!("sandbox:{profile:?}"),
            SimulatedAttempt::NoSandbox => "no-sandbox".to_string(),
        }
    }
}

impl SimulatedResult {
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

    #[test]
    fn read_command_succeeds_in_read_only_sandbox() {
        let runner = SimulatedSandboxRunner::new();
        let request = request(
            &["cat", "package.json"],
            CapabilityKind::SafeRead,
            SandboxProfile::ReadOnly,
            NetworkPolicy::Deny,
        );
        let attempt = ExecutionAttempt::SandboxFirst {
            sandbox_profile: SandboxProfile::ReadOnly,
        };

        let result = runner.run(&request, &attempt);

        match result {
            ExecutionResult::Success { stdout } => {
                assert!(stdout.contains("simulated read success"));
                assert!(stdout.contains("cat package.json"));
            }
            other => panic!("expected simulated success, got {other:?}"),
        }
    }

    #[test]
    fn test_command_failure_is_command_failed_not_sandbox_denied() {
        let runner = SimulatedSandboxRunner::new();
        let request = request(
            &["npm", "test", "--", "fail"],
            CapabilityKind::SafeTest,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Deny,
        );
        let attempt = ExecutionAttempt::SandboxFirst {
            sandbox_profile: SandboxProfile::WorkspaceWrite,
        };

        let result = runner.run(&request, &attempt);

        match result {
            ExecutionResult::Failure(ExecutionFailure::CommandFailed { exit_code, stderr }) => {
                assert_eq!(exit_code, 1);
                assert!(stderr.contains("simulated command failed"));
                assert!(stderr.contains("npm test -- fail"));
            }
            other => panic!("expected command failure, got {other:?}"),
        }
    }

    #[test]
    fn install_command_is_sandbox_denied_in_workspace_write_sandbox() {
        let runner = SimulatedSandboxRunner::new();
        let request = request(
            &["npm", "install", "vite"],
            CapabilityKind::NetworkInstall,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Prompt,
        );
        let attempt = ExecutionAttempt::SandboxFirst {
            sandbox_profile: SandboxProfile::WorkspaceWrite,
        };

        let result = runner.run(&request, &attempt);

        match result {
            ExecutionResult::Failure(ExecutionFailure::SandboxDenied {
                output,
                network_context,
            }) => {
                assert!(output.contains("network access denied by simulated sandbox"));
                assert!(output.contains("npm install vite"));
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

    #[test]
    fn install_command_succeeds_without_sandbox_on_retry() {
        let runner = SimulatedSandboxRunner::new();
        let request = request(
            &["npm", "install", "vite"],
            CapabilityKind::NetworkInstall,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Prompt,
        );
        let attempt = ExecutionAttempt::NoSandboxRetry {
            reason: "approved retry outside sandbox".to_string(),
        };

        let result = runner.run(&request, &attempt);

        match result {
            ExecutionResult::Success { stdout } => {
                assert!(stdout.contains("simulated install success without sandbox"));
                assert!(stdout.contains("npm install vite"));
            }
            other => panic!("expected no-sandbox success, got {other:?}"),
        }
    }

    #[test]
    fn unsupported_command_attempt_returns_command_failed() {
        let runner = SimulatedSandboxRunner::new();
        let request = request(
            &["cargo", "test"],
            CapabilityKind::SafeTest,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Deny,
        );
        let attempt = ExecutionAttempt::SandboxFirst {
            sandbox_profile: SandboxProfile::WorkspaceWrite,
        };

        let result = runner.run(&request, &attempt);

        match result {
            ExecutionResult::Failure(ExecutionFailure::CommandFailed { exit_code, stderr }) => {
                assert_eq!(exit_code, 127);
                assert!(stderr.contains("unsupported simulated command"));
                assert!(stderr.contains("cargo test"));
                assert!(stderr.contains("sandbox:WorkspaceWrite"));
                assert!(stderr.contains("no SimulatedRule matched argv prefix"));
            }
            other => panic!("expected unsupported command failure, got {other:?}"),
        }
    }
}
