use async_trait::async_trait;
use tokio::process::Command;

use crate::{
    model::{
        approval::SandboxProfile,
        command_request::CommandRequest,
        event::ExecutionAttempt,
        execution::{ExecutionFailure, ExecutionResult},
    },
    tool::shell::execution::{
        ExecutionRunner, os_execution_runner::SandboxBackend::MacosSandboxExec,
    },
};

pub struct OsExecutionRunner {
    sandbox_backend: SandboxBackend,
}

pub enum SandboxBackend {
    /// MacOS 自带的 sandbox-exec
    MacosSandboxExec,
}

#[async_trait]
impl ExecutionRunner for OsExecutionRunner {
    async fn run(&self, request: &CommandRequest, attempt: &ExecutionAttempt) -> ExecutionResult {
        let output = match attempt {
            ExecutionAttempt::SandboxFirst { sandbox_profile } => match self.sandbox_backend {
                MacosSandboxExec => run_with_sandbox_exec(request, *sandbox_profile).await,
            },
            ExecutionAttempt::NoSandboxFirst { .. } | ExecutionAttempt::NoSandboxRetry { .. } => {
                run_without_sandbox(request).await
            }
        };

        map_output(attempt, output)
    }
}

impl OsExecutionRunner {
    pub fn new(sandbox_backend: SandboxBackend) -> Self {
        Self { sandbox_backend }
    }
}

#[cfg(target_os = "macos")]
impl Default for OsExecutionRunner {
    fn default() -> Self {
        Self {
            sandbox_backend: SandboxBackend::MacosSandboxExec,
        }
    }
}

async fn run_without_sandbox(request: &CommandRequest) -> std::io::Result<std::process::Output> {
    let Some(program) = request.argv.first() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty argv",
        ));
    };

    let mut command = Command::new(program);
    command.args(&request.argv[1..]);
    command.current_dir(&request.cwd);
    command.output().await
}

async fn run_with_sandbox_exec(
    request: &CommandRequest,
    sandbox_profile: SandboxProfile,
) -> std::io::Result<std::process::Output> {
    if matches!(sandbox_profile, SandboxProfile::NoSandbox) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "SandboxFirst cannot run with SandboxProfile::NoSandbox",
        ));
    }

    let Some(program) = request.argv.first() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty argv",
        ));
    };

    let profile = sandbox_exec_profile(sandbox_profile, request);

    let mut command = Command::new("/usr/bin/sandbox-exec");
    command.arg("-p");
    command.arg(profile);
    command.arg(program);
    command.args(&request.argv[1..]);
    command.current_dir(&request.cwd);
    command.output().await
}

fn map_output(
    attempt: &ExecutionAttempt,
    output: std::io::Result<std::process::Output>,
) -> ExecutionResult {
    let output = match output {
        Ok(output) => output,
        Err(err) => {
            return ExecutionResult::Failure(ExecutionFailure::CommandFailed {
                exit_code: 127,
                stderr: err.to_string(),
            });
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        return ExecutionResult::Success { stdout };
    }

    let exit_code = output.status.code().unwrap_or(1);
    classify_failed_output(attempt, exit_code, stdout, stderr)
}

fn sandbox_exec_profile(profile: SandboxProfile, request: &CommandRequest) -> String {
    match profile {
        SandboxProfile::ReadOnly => read_only_profile(),
        SandboxProfile::WorkspaceWrite => workspace_write_profile(&request.cwd),
        SandboxProfile::NoSandbox => {
            unreachable!("SandboxProfile::NoSandbox is rejected before profile generation")
        }
    }
}

fn read_only_profile() -> String {
    r#"
(version 1)
(deny default)
(allow process*)
(allow file-read*)
"#
    .to_string()
}

fn workspace_write_profile(cwd: &std::path::Path) -> String {
    let writable_root = std::fs::canonicalize(cwd).unwrap_or_else(|_| cwd.to_path_buf());
    let writable_root = sandbox_scheme_string(&writable_root.to_string_lossy());

    format!(
        r#"
(version 1)
(deny default)
(allow process*)
(allow file-read*)
(allow file-write* (subpath "{writable_root}"))
"#,
    )
}

fn sandbox_scheme_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str(r"\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str(r"\n"),
            '\r' => escaped.push_str(r"\r"),
            '\t' => escaped.push_str(r"\t"),
            other => escaped.push(other),
        }
    }
    escaped
}

fn classify_failed_output(
    attempt: &ExecutionAttempt,
    exit_code: i32,
    stdout: String,
    stderr: String,
) -> ExecutionResult {
    let combined = format!("{stdout}\n{stderr}");

    if matches!(attempt, ExecutionAttempt::SandboxFirst { .. })
        && looks_like_sandbox_denied(&combined)
    {
        return ExecutionResult::Failure(ExecutionFailure::SandboxDenied {
            output: combined,
            network_context: None,
        });
    }

    ExecutionResult::Failure(ExecutionFailure::CommandFailed {
        exit_code,
        stderr: combined,
    })
}

fn looks_like_sandbox_denied(output: &str) -> bool {
    let lower = output.to_ascii_lowercase();
    lower.contains("operation not permitted")
        || lower.contains("deny")
        || lower.contains("sandbox")
        || lower.contains("not permitted")
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use std::{
        env::{current_dir, temp_dir},
        fs,
        path::PathBuf,
    };

    use crate::model::{
        approval::{ApprovalPolicy, NetworkPolicy},
        capability::CapabilityKind,
    };

    use super::*;

    fn temp_workspace() -> anyhow::Result<PathBuf> {
        let path = temp_dir().join(format!(
            "codex-mini-demo-os-runner-test-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        if path.exists() {
            fs::remove_dir_all(&path)?;
        }
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    #[tokio::test]
    async fn no_sandbox_should_work() -> anyhow::Result<()> {
        let runner = OsExecutionRunner::default();
        let result = runner
            .run(
                &CommandRequest {
                    raw_command: "cat Cargo.toml".to_string(),
                    argv: vec!["cat".to_string(), "Cargo.toml".to_string()],
                    cwd: current_dir().unwrap(),
                    capability: CapabilityKind::SafeRead,
                    approval_policy: ApprovalPolicy::OnFailure,
                    sandbox_profile: SandboxProfile::ReadOnly,
                    network_policy: NetworkPolicy::Deny,
                    justification: None,
                },
                &ExecutionAttempt::NoSandboxFirst {
                    reason: "test".to_string(),
                },
            )
            .await;

        assert!(matches!(result, ExecutionResult::Success { .. }));
        Ok(())
    }

    #[tokio::test]
    async fn read_only_sandbox_should_work() -> anyhow::Result<()> {
        let runner = OsExecutionRunner::default();
        let result = runner
            .run(
                &CommandRequest {
                    raw_command: "cat Cargo.toml".to_string(),
                    argv: vec!["cat".to_string(), "Cargo.toml".to_string()],
                    cwd: current_dir().unwrap(),
                    capability: CapabilityKind::SafeRead,
                    approval_policy: ApprovalPolicy::OnFailure,
                    sandbox_profile: SandboxProfile::ReadOnly,
                    network_policy: NetworkPolicy::Deny,
                    justification: None,
                },
                &ExecutionAttempt::SandboxFirst {
                    sandbox_profile: SandboxProfile::ReadOnly,
                },
            )
            .await;

        assert!(matches!(result, ExecutionResult::Success { .. }));
        Ok(())
    }

    #[tokio::test]
    async fn read_only_sandbox_should_block_write_and_no_sandbox_should_work() -> anyhow::Result<()>
    {
        let runner = OsExecutionRunner::default();
        let cwd = temp_workspace()?;

        // read only sandbox should block write
        let result = runner
            .run(
                &CommandRequest {
                    raw_command: r#"sh -c "echo hi > check.log""#.to_string(),
                    argv: vec![
                        "sh".to_string(),
                        "-c".to_string(),
                        "echo hi > check.log".to_string(),
                    ],
                    cwd: cwd.clone(),
                    capability: CapabilityKind::DangerousShell,
                    approval_policy: ApprovalPolicy::OnFailure,
                    sandbox_profile: SandboxProfile::ReadOnly,
                    network_policy: NetworkPolicy::Prompt,
                    justification: None,
                },
                &ExecutionAttempt::SandboxFirst {
                    sandbox_profile: SandboxProfile::ReadOnly,
                },
            )
            .await;

        assert!(matches!(
            result,
            ExecutionResult::Failure(ExecutionFailure::SandboxDenied { .. })
        ));
        assert!(!cwd.join("check.log").exists());

        // no sandbox retry should work
        let result = runner
            .run(
                &CommandRequest {
                    raw_command: r#"sh -c "echo hi > check.log""#.to_string(),
                    argv: vec![
                        "sh".to_string(),
                        "-c".to_string(),
                        "echo hi > check.log".to_string(),
                    ],
                    cwd: cwd.clone(),
                    capability: CapabilityKind::DangerousShell,
                    approval_policy: ApprovalPolicy::OnFailure,
                    sandbox_profile: SandboxProfile::ReadOnly,
                    network_policy: NetworkPolicy::Prompt,
                    justification: None,
                },
                &ExecutionAttempt::NoSandboxRetry {
                    reason: "retry".to_string(),
                },
            )
            .await;

        assert!(matches!(result, ExecutionResult::Success { .. }));
        assert_eq!(fs::read_to_string(cwd.join("check.log"))?, "hi\n");
        fs::remove_dir_all(cwd)?;
        Ok(())
    }

    #[tokio::test]
    async fn workspace_write_sandbox_should_allow_write_in_cwd() -> anyhow::Result<()> {
        let runner = OsExecutionRunner::default();
        let cwd = temp_workspace()?;

        let result = runner
            .run(
                &CommandRequest {
                    raw_command: r#"sh -c "echo hi > workspace.log""#.to_string(),
                    argv: vec![
                        "sh".to_string(),
                        "-c".to_string(),
                        "echo hi > workspace.log".to_string(),
                    ],
                    cwd: cwd.clone(),
                    capability: CapabilityKind::SafeTest,
                    approval_policy: ApprovalPolicy::OnFailure,
                    sandbox_profile: SandboxProfile::WorkspaceWrite,
                    network_policy: NetworkPolicy::Deny,
                    justification: None,
                },
                &ExecutionAttempt::SandboxFirst {
                    sandbox_profile: SandboxProfile::WorkspaceWrite,
                },
            )
            .await;

        assert!(matches!(result, ExecutionResult::Success { .. }));
        assert_eq!(fs::read_to_string(cwd.join("workspace.log"))?, "hi\n");
        fs::remove_dir_all(cwd)?;
        Ok(())
    }

    #[tokio::test]
    async fn sandbox_first_with_no_sandbox_profile_should_fail_closed() -> anyhow::Result<()> {
        let runner = OsExecutionRunner::default();
        let result = runner
            .run(
                &CommandRequest {
                    raw_command: "cat Cargo.toml".to_string(),
                    argv: vec!["cat".to_string(), "Cargo.toml".to_string()],
                    cwd: current_dir().unwrap(),
                    capability: CapabilityKind::SafeRead,
                    approval_policy: ApprovalPolicy::OnFailure,
                    sandbox_profile: SandboxProfile::NoSandbox,
                    network_policy: NetworkPolicy::Deny,
                    justification: None,
                },
                &ExecutionAttempt::SandboxFirst {
                    sandbox_profile: SandboxProfile::NoSandbox,
                },
            )
            .await;

        assert!(matches!(
            result,
            ExecutionResult::Failure(ExecutionFailure::CommandFailed { .. })
        ));
        Ok(())
    }

    #[test]
    fn sandbox_scheme_string_should_escape_path_literals() {
        assert_eq!(
            sandbox_scheme_string(r#"/tmp/demo "quote" \ path"#),
            r#"/tmp/demo \"quote\" \\ path"#
        );
    }
}
