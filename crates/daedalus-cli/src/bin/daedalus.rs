use std::process::ExitCode;

use clap::Parser;
use daedalus_cli::domain::DaedalusError;
use daedalus_cli::interfaces::agent_cli::args::Cli;
use daedalus_cli::interfaces::agent_cli::context::AgentCliContext;
use daedalus_cli::interfaces::agent_cli::executor::CmdExecutor;
use daedalus_cli::interfaces::agent_cli::presenter::print_error;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> ExitCode {
    init_tracing();
    let cli = Cli::parse();
    let format = cli.format;
    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            print_error(&error, format);
            ExitCode::from(1)
        }
    }
}

async fn run(cli: Cli) -> Result<(), DaedalusError> {
    let ctx = AgentCliContext::discover(cli.format)?;
    cli.command.execute(ctx).await
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}
