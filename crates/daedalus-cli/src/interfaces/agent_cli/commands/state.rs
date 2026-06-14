use std::path::PathBuf;

use clap::{Args, Subcommand};
use enum_dispatch::enum_dispatch;
use tracing::debug;

use crate::application::project_navigation::sync_project_navigation;
use crate::application::render::render_state;
use crate::application::transition_stage::{StageAction, TransitionStageOptions, transition_stage};
use crate::domain::{ApprovalSource, DaedalusError, Result};
use crate::infrastructure::workspace_fs;
use crate::interfaces::agent_cli::context::AgentCliContext;
use crate::interfaces::agent_cli::executor::CmdExecutor;
use crate::interfaces::agent_cli::presenter::{print_render, print_transition};

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
    /// 显式 project 目录。
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
    /// 显式 topic slug。
    #[arg(long)]
    pub topic: Option<String>,
    /// 显式 topic 目录。
    #[arg(long)]
    pub topic_dir: Option<PathBuf>,
    /// 状态流转原因。
    #[arg(long)]
    pub reason: Option<String>,
}

impl CmdExecutor for EnterArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        debug!(stage = %self.stage_id, "进入学习阶段");
        execute_transition(
            ctx,
            self.project_dir,
            self.topic,
            self.topic_dir,
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
    /// 显式 project 目录。
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
    /// 显式 topic slug。
    #[arg(long)]
    pub topic: Option<String>,
    /// 显式 topic 目录。
    #[arg(long)]
    pub topic_dir: Option<PathBuf>,
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
        let stage_id = self.stage_id;
        let approval_source = self
            .approval_source
            .as_deref()
            .map(|value| {
                ApprovalSource::parse(value)
                    .ok_or_else(|| DaedalusError::InvalidApprovalSource(value.to_owned()))
            })
            .transpose()?;
        execute_transition(
            ctx,
            self.project_dir,
            self.topic,
            self.topic_dir,
            stage_id,
            StageAction::Complete {
                force: self.force,
                approval_source,
            },
            self.reason,
        )
        .await
    }
}

/// 阻塞阶段命令参数。
#[derive(Debug, Args)]
pub struct BlockArgs {
    /// 阶段 ID。
    pub stage_id: String,
    /// 显式 project 目录。
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
    /// 显式 topic slug。
    #[arg(long)]
    pub topic: Option<String>,
    /// 显式 topic 目录。
    #[arg(long)]
    pub topic_dir: Option<PathBuf>,
    /// 阻塞原因。
    #[arg(long)]
    pub reason: String,
}

impl CmdExecutor for BlockArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        debug!(stage = %self.stage_id, "阻塞学习阶段");
        execute_transition(
            ctx,
            self.project_dir,
            self.topic,
            self.topic_dir,
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
    /// 显式 project 目录。
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
    /// 显式 topic slug。
    #[arg(long)]
    pub topic: Option<String>,
    /// 显式 topic 目录。
    #[arg(long)]
    pub topic_dir: Option<PathBuf>,
    /// 状态流转原因。
    #[arg(long)]
    pub reason: Option<String>,
}

impl CmdExecutor for ResumeArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        debug!(stage = %self.stage_id, "恢复学习阶段");
        execute_transition(
            ctx,
            self.project_dir,
            self.topic,
            self.topic_dir,
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
    /// 显式 project 目录。
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
    /// 显式 topic slug。
    #[arg(long)]
    pub topic: Option<String>,
    /// 显式 topic 目录。
    #[arg(long)]
    pub topic_dir: Option<PathBuf>,
    /// 回退原因。
    #[arg(long)]
    pub reason: String,
}

impl CmdExecutor for RollbackArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        debug!(stage = %self.stage_id, "回退学习阶段");
        execute_transition(
            ctx,
            self.project_dir,
            self.topic,
            self.topic_dir,
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
    /// 显式 project 目录。
    pub project_dir: Option<PathBuf>,
    /// 渲染 project state.md；缺省渲染 active topic。
    #[arg(long)]
    pub project: bool,
    /// 显式 topic slug。
    #[arg(long)]
    pub topic: Option<String>,
    /// 显式 topic 目录。
    #[arg(long)]
    pub topic_dir: Option<PathBuf>,
}

impl CmdExecutor for RenderArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        debug!("渲染 state.md");
        let dir = if self.project {
            workspace_fs::default_project_dir(self.project_dir)?
        } else {
            workspace_fs::default_topic_dir(self.topic_dir, self.project_dir, self.topic)?
        };
        if self.project {
            sync_project_navigation(&dir)?;
        }
        let output = render_state(&dir)?;
        print_render(&output, ctx.format);
        Ok(())
    }
}

async fn execute_transition(
    ctx: AgentCliContext,
    project_dir: Option<PathBuf>,
    topic: Option<String>,
    topic_dir: Option<PathBuf>,
    stage_id: String,
    action: StageAction,
    reason: Option<String>,
) -> Result<()> {
    let task_dir = workspace_fs::default_topic_dir(topic_dir, project_dir, topic)?;
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
