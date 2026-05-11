use std::path::PathBuf;

use clap::{Args, Subcommand};
use enum_dispatch::enum_dispatch;
use tracing::debug;

use crate::application::close_task::CloseTaskAction;
use crate::application::render::render_state;
use crate::application::transition_stage::{StageAction, TransitionStageOptions, transition_stage};
use crate::domain::{ApprovalSource, DaedalusError, Result};
use crate::infrastructure::workspace_fs;
use crate::interfaces::agent_cli::commands::task::execute_close_task;
use crate::interfaces::agent_cli::context::AgentCliContext;
use crate::interfaces::agent_cli::executor::CmdExecutor;
use crate::interfaces::agent_cli::presenter::{print_render, print_transition};

const FINAL_STAGE_ID: &str = "10-archivist";

/// 状态命令。
#[derive(Debug, Args)]
pub struct StateCommand {
    /// 具体状态子命令。
    #[command(subcommand)]
    pub command: StateSubcommand,
}

impl CmdExecutor for StateCommand {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        self.command.execute(ctx).await
    }
}

/// 状态子命令集合。
#[derive(Debug, Subcommand)]
#[enum_dispatch(CmdExecutor)]
pub enum StateSubcommand {
    /// 进入阶段。
    Enter(EnterArgs),
    /// 完成阶段。
    Complete(CompleteArgs),
    /// 阻塞阶段。
    Block(BlockArgs),
    /// 恢复阶段。
    Resume(ResumeArgs),
    /// 回退到某个已到达阶段。
    Rollback(RollbackArgs),
    /// 重新渲染 `state.md`。
    Render(RenderArgs),
}

/// 进入阶段命令参数。
#[derive(Debug, Args)]
pub struct EnterArgs {
    /// 阶段 ID。
    pub stage_id: String,
    /// 显式学习任务目录。
    #[arg(long)]
    pub task_dir: Option<PathBuf>,
    /// 状态流转原因。
    #[arg(long)]
    pub reason: Option<String>,
}

impl CmdExecutor for EnterArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        debug!(stage = %self.stage_id, "进入学习阶段");
        execute_transition(
            ctx,
            self.task_dir,
            self.stage_id,
            StageAction::Enter,
            self.reason,
        )
        .await
    }
}

/// 完成阶段命令参数。
#[derive(Debug, Args)]
pub struct CompleteArgs {
    /// 阶段 ID。
    pub stage_id: String,
    /// 显式学习任务目录。
    #[arg(long)]
    pub task_dir: Option<PathBuf>,
    /// 是否强制完成。
    #[arg(long)]
    pub force: bool,
    /// 强制完成或普通完成的原因。
    #[arg(long)]
    pub reason: Option<String>,
    /// 强制完成时的批准来源。
    #[arg(long)]
    pub approval_source: Option<String>,
}

impl CmdExecutor for CompleteArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        debug!(stage = %self.stage_id, force = self.force, "完成学习阶段");
        let is_final_stage = self.stage_id == FINAL_STAGE_ID;
        let task_dir = self.task_dir;
        let stage_id = self.stage_id;
        let reason = self.reason;
        let approval_source = self
            .approval_source
            .as_deref()
            .map(|value| {
                ApprovalSource::parse(value)
                    .ok_or_else(|| DaedalusError::InvalidApprovalSource(value.to_owned()))
            })
            .transpose()?;
        if is_final_stage {
            let close_reason = reason.clone().unwrap_or_default();
            if close_reason.trim().is_empty() {
                return Err(DaedalusError::TaskLifecycleReasonRequired);
            }
            execute_close_task(
                ctx,
                task_dir,
                CloseTaskAction::Complete,
                close_reason,
                true,
                self.force,
                approval_source,
            )
            .await?;
        } else {
            execute_transition(
                ctx,
                task_dir,
                stage_id,
                StageAction::Complete {
                    force: self.force,
                    approval_source,
                },
                reason,
            )
            .await?;
        }
        Ok(())
    }
}

/// 阻塞阶段命令参数。
#[derive(Debug, Args)]
pub struct BlockArgs {
    /// 阶段 ID。
    pub stage_id: String,
    /// 显式学习任务目录。
    #[arg(long)]
    pub task_dir: Option<PathBuf>,
    /// 阻塞原因。
    #[arg(long)]
    pub reason: String,
}

impl CmdExecutor for BlockArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        debug!(stage = %self.stage_id, "阻塞学习阶段");
        execute_transition(
            ctx,
            self.task_dir,
            self.stage_id,
            StageAction::Block,
            Some(self.reason),
        )
        .await
    }
}

/// 恢复阶段命令参数。
#[derive(Debug, Args)]
pub struct ResumeArgs {
    /// 阶段 ID。
    pub stage_id: String,
    /// 显式学习任务目录。
    #[arg(long)]
    pub task_dir: Option<PathBuf>,
    /// 状态流转原因。
    #[arg(long)]
    pub reason: Option<String>,
}

impl CmdExecutor for ResumeArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        debug!(stage = %self.stage_id, "恢复学习阶段");
        execute_transition(
            ctx,
            self.task_dir,
            self.stage_id,
            StageAction::Resume,
            self.reason,
        )
        .await
    }
}

/// 回退阶段命令参数。
#[derive(Debug, Args)]
pub struct RollbackArgs {
    /// 回退目标阶段 ID。
    pub stage_id: String,
    /// 显式学习任务目录。
    #[arg(long)]
    pub task_dir: Option<PathBuf>,
    /// 回退原因。
    #[arg(long)]
    pub reason: String,
}

impl CmdExecutor for RollbackArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        debug!(stage = %self.stage_id, "回退学习阶段");
        execute_transition(
            ctx,
            self.task_dir,
            self.stage_id,
            StageAction::Rollback,
            Some(self.reason),
        )
        .await
    }
}

/// 渲染状态摘要命令参数。
#[derive(Debug, Args)]
pub struct RenderArgs {
    /// 显式学习任务目录；缺省时由当前目录或唯一 active task 推导。
    pub task_dir: Option<PathBuf>,
}

impl CmdExecutor for RenderArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        debug!("渲染 state.md");
        let task_dir = workspace_fs::default_task_dir(self.task_dir)?;
        let output = render_state(&task_dir)?;
        print_render(&output, ctx.format);
        Ok(())
    }
}

async fn execute_transition(
    ctx: AgentCliContext,
    task_dir: Option<PathBuf>,
    stage_id: String,
    action: StageAction,
    reason: Option<String>,
) -> Result<()> {
    let task_dir = workspace_fs::default_task_dir(task_dir)?;
    let output = transition_stage(TransitionStageOptions {
        task_dir,
        stage_id,
        action,
        reason,
        actor: "daedalus-cli".to_owned(),
    })?;
    print_transition(&output, ctx.format);
    Ok(())
}
