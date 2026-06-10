use std::{env::current_dir, sync::Arc};

use codex_mini_demo::{
    agent::{llm::openai::OpenAiCompatibleLlmBuilder, react::ReActAgent},
    cli,
    model::approval::ApprovalPolicy,
    tool::{
        function::{add_spec, sub_spec},
        runtime::{ToolRuntime, ToolRuntimeContext},
        shell::{
            approval::ApprovalGateway, execution::os_execution_runner::OsExecutionRunner,
            registry::CapabilityRegistry, run_command_spec,
        },
    },
};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let llm = OpenAiCompatibleLlmBuilder::default()
        .tools(vec![add_spec(), sub_spec(), run_command_spec()])
        .build()?;

    let (approval_tx, approval_rx) = mpsc::channel(16);

    let runtime = ToolRuntime::new(ToolRuntimeContext {
        cwd: current_dir()?,
        approval_policy: ApprovalPolicy::OnFailure,
        registry: CapabilityRegistry::new(),
        execution_runner: Arc::new(OsExecutionRunner::default()),
        approval_gateway: Arc::new(ApprovalGateway::new(approval_rx)),
    });

    let agent = Arc::new(ReActAgent::new(Arc::new(llm), None, Arc::new(runtime)));

    cli::event_loop::run_cli(agent, approval_tx).await
}
