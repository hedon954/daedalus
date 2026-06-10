#[cfg(target_os = "macos")]
use std::{
    fs,
    path::{Path, PathBuf},
};

#[cfg(target_os = "macos")]
use codex_mini_demo::{
    model::{
        approval::{ApprovalPolicy, NetworkPolicy, SandboxProfile},
        capability::CapabilityKind,
        command_request::CommandRequest,
        event::ExecutionAttempt,
        execution::{ExecutionFailure, ExecutionResult},
    },
    tool::shell::execution::{ExecutionRunner, os_execution_runner::OsExecutionRunner},
};

#[cfg(target_os = "macos")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let workspace = example_workspace()?;
    fs::write(workspace.join("input.txt"), "hello sandbox\n")?;

    let runner = OsExecutionRunner::default();

    assert_success(
        runner
            .run(
                &request(&workspace, "cat input.txt", &["cat", "input.txt"]),
                &ExecutionAttempt::NoSandboxFirst {
                    reason: "example baseline".to_string(),
                },
            )
            .await,
    );
    println!("ok: no-sandbox read succeeded");

    assert_success(
        runner
            .run(
                &request(&workspace, "cat input.txt", &["cat", "input.txt"]),
                &ExecutionAttempt::SandboxFirst {
                    sandbox_profile: SandboxProfile::ReadOnly,
                },
            )
            .await,
    );
    println!("ok: read-only sandbox read succeeded");

    assert_sandbox_denied(
        runner
            .run(
                &request(
                    &workspace,
                    r#"sh -c "echo hi > blocked.txt""#,
                    &["sh", "-c", "echo hi > blocked.txt"],
                ),
                &ExecutionAttempt::SandboxFirst {
                    sandbox_profile: SandboxProfile::ReadOnly,
                },
            )
            .await,
    );
    assert!(!workspace.join("blocked.txt").exists());
    println!("ok: read-only sandbox blocked write");

    assert_success(
        runner
            .run(
                &request(
                    &workspace,
                    r#"sh -c "echo hi > workspace.txt""#,
                    &["sh", "-c", "echo hi > workspace.txt"],
                ),
                &ExecutionAttempt::SandboxFirst {
                    sandbox_profile: SandboxProfile::WorkspaceWrite,
                },
            )
            .await,
    );
    assert_eq!(fs::read_to_string(workspace.join("workspace.txt"))?, "hi\n");
    println!("ok: workspace-write sandbox allowed cwd write");

    assert_success(
        runner
            .run(
                &request(
                    &workspace,
                    r#"sh -c "echo hi > allowed.txt""#,
                    &["sh", "-c", "echo hi > allowed.txt"],
                ),
                &ExecutionAttempt::NoSandboxRetry {
                    reason: "example approved retry".to_string(),
                },
            )
            .await,
    );
    assert_eq!(fs::read_to_string(workspace.join("allowed.txt"))?, "hi\n");
    println!("ok: no-sandbox retry write succeeded");

    fs::remove_dir_all(&workspace)?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() {
    println!("skipped: sandbox-exec is only available on macOS");
}

#[cfg(target_os = "macos")]
fn example_workspace() -> anyhow::Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!(
        "codex-mini-demo-os-runner-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg(target_os = "macos")]
fn request(cwd: &Path, raw_command: &str, argv: &[&str]) -> CommandRequest {
    CommandRequest {
        raw_command: raw_command.to_string(),
        argv: argv.iter().map(|arg| arg.to_string()).collect(),
        cwd: cwd.to_path_buf(),
        capability: CapabilityKind::SafeRead,
        approval_policy: ApprovalPolicy::OnFailure,
        sandbox_profile: SandboxProfile::ReadOnly,
        network_policy: NetworkPolicy::Deny,
        justification: Some("os execution runner example".to_string()),
    }
}

#[cfg(target_os = "macos")]
fn assert_success(result: ExecutionResult) {
    match result {
        ExecutionResult::Success { .. } => {}
        other => panic!("expected success, got {other:?}"),
    }
}

#[cfg(target_os = "macos")]
fn assert_sandbox_denied(result: ExecutionResult) {
    match result {
        ExecutionResult::Failure(ExecutionFailure::SandboxDenied { .. }) => {}
        other => panic!("expected sandbox denial, got {other:?}"),
    }
}
