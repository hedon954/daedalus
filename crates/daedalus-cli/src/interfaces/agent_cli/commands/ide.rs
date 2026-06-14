use clap::{Args, Subcommand};

use crate::application::ide::sync_rust_analyzer_linked_projects;
use crate::domain::Result;
use crate::interfaces::agent_cli::context::AgentCliContext;
use crate::interfaces::agent_cli::executor::CmdExecutor;
use crate::interfaces::agent_cli::presenter::print_rust_analyzer_sync;

/// IDE 派生配置命令。
#[derive(Debug, Args)]
pub struct IdeCommand {
    #[command(subcommand)]
    pub command: IdeSubcommand,
}

impl CmdExecutor for IdeCommand {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        self.command.execute(ctx).await
    }
}

/// IDE 子命令。
#[derive(Debug, Subcommand)]
pub enum IdeSubcommand {
    /// 同步 VSCode rust-analyzer linkedProjects。
    SyncRustAnalyzer(IdeSyncRustAnalyzerArgs),
}

/// 同步 VSCode rust-analyzer linkedProjects 参数。
#[derive(Debug, Args)]
pub struct IdeSyncRustAnalyzerArgs {}

impl CmdExecutor for IdeSyncRustAnalyzerArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        let output = sync_rust_analyzer_linked_projects(&ctx.repo_root)?;
        print_rust_analyzer_sync(&output, ctx.format);
        Ok(())
    }
}

impl CmdExecutor for IdeSubcommand {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        match self {
            Self::SyncRustAnalyzer(args) => args.execute(ctx).await,
        }
    }
}
