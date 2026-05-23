use std::path::PathBuf;

use clap::{Args, Subcommand};
use enum_dispatch::enum_dispatch;
use tracing::debug;

use crate::application::close_task::{CloseTaskAction, CloseTaskOptions, close_task};
use crate::domain::{ApprovalSource, Result};
use crate::infrastructure::workspace_fs;
use crate::interfaces::agent_cli::context::AgentCliContext;
use crate::interfaces::agent_cli::executor::CmdExecutor;
use crate::interfaces::agent_cli::presenter::print_close_task;

/// 任务生命周期命令。
#[derive(Debug, Args)]
pub struct TaskCommand {
    /// 具体任务生命周期子命令。
    #[command(subcommand)]
    pub command: TaskSubcommand,
}

impl CmdExecutor for TaskCommand {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        self.command.execute(ctx).await
    }
}

/// 任务生命周期子命令集合。
#[derive(Debug, Subcommand)]
#[enum_dispatch(CmdExecutor)]
pub enum TaskSubcommand {
    /// 完成学习任务并移动到 completed。
    Complete(TaskCompleteArgs),
    /// 放弃学习任务并移动到 abandoned。
    Abandon(TaskAbandonArgs),
}

/// 完成学习任务命令参数。
#[derive(Debug, Args)]
pub struct TaskCompleteArgs {
    /// 显式学习任务目录；缺省时由当前目录或唯一 active task 推导。
    pub task_dir: Option<PathBuf>,
    /// 完成原因。
    #[arg(long)]
    pub reason: String,
}

impl CmdExecutor for TaskCompleteArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        debug!("完成学习任务并移动到 completed");
        execute_close_task(
            ctx,
            self.task_dir,
            CloseTaskAction::Complete,
            self.reason,
            false,
            false,
            None,
        )
        .await
    }
}

/// 放弃学习任务命令参数。
#[derive(Debug, Args)]
pub struct TaskAbandonArgs {
    /// 显式学习任务目录；缺省时由当前目录或唯一 active task 推导。
    pub task_dir: Option<PathBuf>,
    /// 放弃原因。
    #[arg(long)]
    pub reason: String,
}

impl CmdExecutor for TaskAbandonArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        debug!("放弃学习任务并移动到 abandoned");
        execute_close_task(
            ctx,
            self.task_dir,
            CloseTaskAction::Abandon,
            self.reason,
            false,
            false,
            None,
        )
        .await
    }
}

pub async fn execute_close_task(
    ctx: AgentCliContext,
    task_dir: Option<PathBuf>,
    action: CloseTaskAction,
    reason: String,
    complete_final_stage: bool,
    final_stage_force: bool,
    final_stage_approval_source: Option<ApprovalSource>,
) -> Result<()> {
    let task_dir = workspace_fs::default_task_dir(task_dir)?;
    let output = close_task(CloseTaskOptions {
        repo_root: ctx.repo_root,
        task_dir,
        action,
        reason,
        actor: "daedalus-cli".to_owned(),
        complete_final_stage,
        final_stage_force,
        final_stage_approval_source,
    })?;
    print_close_task(&output, ctx.format);
    Ok(())
}
