#[cfg(target_os = "macos")]
use std::{fs, path::PathBuf, sync::Arc};

#[cfg(target_os = "macos")]
use codex_mini_demo::{
    agent::{
        react::EventSender,
        stream_event::{StreamEvent, ToolCallFinished},
    },
    model::{
        approval::{ApprovalPersistence, ApprovalPolicy, NetworkPolicy, SandboxProfile},
        capability::{
            CapabilityDescriptor, CapabilityKind, CapabilityPolicy, DefaultDecision, RetryPolicy,
        },
        event::UserApprovalDecision,
    },
    tool::{
        runtime::{ToolRuntime, ToolRuntimeContext, ToolRuntimeResult},
        shell::{
            approval::{ApprovalGateway, ToolApprovalResult},
            execution::os_execution_runner::OsExecutionRunner,
            registry::CapabilityRegistry,
        },
    },
};

#[cfg(target_os = "macos")]
use tokio::sync::mpsc;

#[cfg(target_os = "macos")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let workspace = example_workspace()?;

    let mut registry = CapabilityRegistry::new();
    registry.register(CapabilityDescriptor {
        kind: CapabilityKind::SafeTest,
        name: "example-mkdir".to_string(),
        description: "Example command that writes a directory".to_string(),
        command_prefixes: vec![vec!["mkdir".to_string()]],
        policy: CapabilityPolicy {
            default_decision: DefaultDecision::Allow,
            first_attempt_sandbox: SandboxProfile::ReadOnly,
            network_policy: NetworkPolicy::Deny,
            retry_policy: RetryPolicy::WithApproval,
        },
    });

    let (approval_tx, approval_rx) = mpsc::channel(16);
    let runtime = Arc::new(ToolRuntime::new(ToolRuntimeContext {
        cwd: workspace.clone(),
        approval_policy: ApprovalPolicy::OnFailure,
        registry,
        execution_runner: Arc::new(OsExecutionRunner::default()),
        approval_gateway: Arc::new(ApprovalGateway::new(approval_rx)),
    }));

    let (event_tx, event_rx) = mpsc::channel(64);
    let call = ToolCallFinished {
        index: 0,
        call_id: "call_os_runtime".to_string(),
        name: "run_command".to_string(),
        arguments: serde_json::json!({
            "command": "mkdir approved-dir",
            "justification": "verify OsExecutionRunner through ToolRuntime"
        })
        .to_string(),
    };

    let (results, events) =
        run_with_auto_approval(runtime, call, event_tx, event_rx, approval_tx).await?;

    assert!(matches!(
        results.as_slice(),
        [(_, ToolRuntimeResult::Finished { .. })]
    ));
    assert!(workspace.join("approved-dir").is_dir());
    assert!(
        events
            .iter()
            .any(|event| matches!(event, StreamEvent::CommandExecutionFailed { .. }))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, StreamEvent::CommandNeedsApproval { .. }))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, StreamEvent::CommandExecutionFinished { .. }))
    );

    println!("ok: ToolRuntime used OsExecutionRunner for sandbox-denied approval retry");
    fs::remove_dir_all(&workspace)?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() {
    println!("skipped: sandbox-exec is only available on macOS");
}

#[cfg(target_os = "macos")]
async fn run_with_auto_approval(
    runtime: Arc<ToolRuntime>,
    call: ToolCallFinished,
    event_tx: EventSender,
    mut event_rx: mpsc::Receiver<anyhow::Result<StreamEvent>>,
    approval_tx: mpsc::Sender<ToolApprovalResult>,
) -> anyhow::Result<(Vec<(ToolCallFinished, ToolRuntimeResult)>, Vec<StreamEvent>)> {
    let mut run = Box::pin(runtime.batch_run(vec![call], &event_tx));
    let mut events = Vec::new();

    loop {
        tokio::select! {
            results = &mut run => {
                while let Ok(event) = event_rx.try_recv() {
                    events.push(event?);
                }
                return Ok((results, events));
            }
            event = event_rx.recv() => {
                let event = event
                    .ok_or_else(|| anyhow::anyhow!("event channel closed before runtime finished"))??;
                if let StreamEvent::CommandNeedsApproval { approval_id, .. } = &event {
                    approval_tx
                        .send(ToolApprovalResult {
                            approval_id: approval_id.clone(),
                            decision: UserApprovalDecision::Approved {
                                persistence: ApprovalPersistence::Once,
                            },
                        })
                        .await?;
                }
                events.push(event);
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn example_workspace() -> anyhow::Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!(
        "codex-mini-demo-os-runtime-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    fs::create_dir_all(&dir)?;
    Ok(dir)
}
