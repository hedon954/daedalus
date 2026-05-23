use clap::{Args, Subcommand};
use enum_dispatch::enum_dispatch;
use tracing::debug;

use crate::application::init_task::{InitTaskOptions, init_repo_learning};
use crate::domain::Result;
use crate::interfaces::agent_cli::context::AgentCliContext;
use crate::interfaces::agent_cli::executor::CmdExecutor;
use crate::interfaces::agent_cli::presenter::print_init;

/// 初始化命令。
#[derive(Debug, Args)]
pub struct InitCommand {
    /// 要初始化的学习任务类型。
    #[command(subcommand)]
    pub kind: InitKind,
}

impl CmdExecutor for InitCommand {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        self.kind.execute(ctx).await
    }
}

/// 支持初始化的学习任务类型。
#[derive(Debug, Subcommand)]
#[enum_dispatch(CmdExecutor)]
pub enum InitKind {
    /// 初始化 repo learning 任务。
    RepoLearning(InitRepoLearningArgs),
}

/// `init repo-learning` 参数。
#[derive(Debug, Args)]
pub struct InitRepoLearningArgs {
    /// 学习任务名称。
    pub name: String,
    /// 初始专题 slug。
    #[arg(long)]
    pub topic: String,
    /// 初始专题标题。
    #[arg(long)]
    pub title: String,
    /// 是否允许已有 active task 时继续创建。
    #[arg(long)]
    pub allow_existing_active: bool,
    /// 使用绕过选项时的原因。
    #[arg(long)]
    pub reason: Option<String>,
}

impl CmdExecutor for InitRepoLearningArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        debug!(task = %self.name, "初始化 repo learning 任务");
        let output = init_repo_learning(InitTaskOptions {
            repo_root: ctx.repo_root,
            name: self.name,
            topic_slug: self.topic,
            topic_title: self.title,
            allow_existing_active: self.allow_existing_active,
            reason: self.reason,
        })?;
        print_init(&output, ctx.format);
        Ok(())
    }
}
