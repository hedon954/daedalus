use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};

use crate::application::review::{
    CloseReviewOptions, CompleteReviewSessionOptions, StartReviewOptions,
    StartReviewSessionOptions, close_review, complete_review_session, list_reviews, render_review,
    show_review, start_review, start_review_session, validate_review,
};
use crate::domain::{DaedalusError, Result, ReviewLifecycle, ReviewMode, ReviewTarget};
use crate::infrastructure::{state_toml, workspace_fs};
use crate::interfaces::agent_cli::context::AgentCliContext;
use crate::interfaces::agent_cli::executor::CmdExecutor;
use crate::interfaces::agent_cli::presenter::{
    print_review, print_review_list, print_review_session, print_review_validation,
};

/// Review 命令。
#[derive(Debug, Args)]
pub struct ReviewCommand {
    #[command(subcommand)]
    pub command: ReviewSubcommand,
}

impl CmdExecutor for ReviewCommand {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        self.command.execute(ctx).await
    }
}

/// Review 子命令。
#[derive(Debug, Subcommand)]
pub enum ReviewSubcommand {
    /// 启动 review plan。
    Start(ReviewStartArgs),
    /// 列出 review plans。
    List(ReviewListArgs),
    /// 展示 review plan。
    Show(ReviewShowArgs),
    /// 完成 review plan。
    Complete(ReviewCloseArgs),
    /// 放弃 review plan。
    Abandon(ReviewCloseArgs),
    /// 渲染 review state.md。
    Render(ReviewShowArgs),
    /// 校验 review plan。
    Validate(ReviewShowArgs),
    /// 管理 review session。
    Session(ReviewSessionCommand),
}

/// Review mode CLI 参数。
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ReviewModeArg {
    Recall,
    Rebuild,
    Application,
    WeaknessRepair,
    Mixed,
}

impl From<ReviewModeArg> for ReviewMode {
    fn from(value: ReviewModeArg) -> Self {
        match value {
            ReviewModeArg::Recall => ReviewMode::Recall,
            ReviewModeArg::Rebuild => ReviewMode::Rebuild,
            ReviewModeArg::Application => ReviewMode::Application,
            ReviewModeArg::WeaknessRepair => ReviewMode::WeaknessRepair,
            ReviewModeArg::Mixed => ReviewMode::Mixed,
        }
    }
}

/// 启动 review 参数。
#[derive(Debug, Args)]
pub struct ReviewStartArgs {
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
    /// 对整个 project 启动复习。
    #[arg(long, conflicts_with = "topic")]
    pub project: bool,
    /// 对指定 topic 启动复习；缺省时使用 active topic。
    #[arg(long)]
    pub topic: Option<String>,
    #[arg(long, value_enum, default_value_t = ReviewModeArg::Mixed)]
    pub mode: ReviewModeArg,
    #[arg(long)]
    pub goal: String,
}

impl CmdExecutor for ReviewStartArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        let project_dir = workspace_fs::default_project_dir(self.project_dir)?;
        let target = resolve_review_target(&project_dir, self.project, self.topic)?;
        let output = start_review(StartReviewOptions {
            repo_root: ctx.repo_root,
            project_dir,
            target,
            mode: self.mode.into(),
            goal: self.goal,
            actor: "daedalus-cli".to_owned(),
        })?;
        print_review(&output, ctx.format);
        Ok(())
    }
}

/// 列出 review 参数。
#[derive(Debug, Args)]
pub struct ReviewListArgs {
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
}

impl CmdExecutor for ReviewListArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        let project_dir = workspace_fs::default_project_dir(self.project_dir)?;
        let output = list_reviews(project_dir)?;
        print_review_list(&output, ctx.format);
        Ok(())
    }
}

/// 指向单个 review 的参数。
#[derive(Debug, Args)]
pub struct ReviewShowArgs {
    pub review_id: String,
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
}

impl ReviewShowArgs {
    async fn execute_show(self, ctx: AgentCliContext) -> Result<()> {
        let project_dir = workspace_fs::default_project_dir(self.project_dir)?;
        let output = show_review(project_dir, self.review_id)?;
        print_review(&output, ctx.format);
        Ok(())
    }

    async fn execute_render(self, ctx: AgentCliContext) -> Result<()> {
        let project_dir = workspace_fs::default_project_dir(self.project_dir)?;
        let output = render_review(project_dir, self.review_id)?;
        print_review(&output, ctx.format);
        Ok(())
    }

    async fn execute_validate(self, ctx: AgentCliContext) -> Result<()> {
        let project_dir = workspace_fs::default_project_dir(self.project_dir)?;
        let output = validate_review(project_dir, self.review_id)?;
        if !output.is_ok() {
            return Err(DaedalusError::WorkspaceValidationFailed(output.issues));
        }
        print_review_validation(&output, ctx.format);
        Ok(())
    }
}

/// 关闭 review 参数。
#[derive(Debug, Args)]
pub struct ReviewCloseArgs {
    pub review_id: String,
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
    #[arg(long)]
    pub reason: String,
}

impl ReviewCloseArgs {
    async fn execute_with_lifecycle(
        self,
        ctx: AgentCliContext,
        lifecycle: ReviewLifecycle,
    ) -> Result<()> {
        let project_dir = workspace_fs::default_project_dir(self.project_dir)?;
        let output = close_review(CloseReviewOptions {
            project_dir,
            review_id: self.review_id,
            lifecycle,
            reason: self.reason,
            actor: "daedalus-cli".to_owned(),
        })?;
        print_review(&output, ctx.format);
        Ok(())
    }
}

/// Review session 命令。
#[derive(Debug, Args)]
pub struct ReviewSessionCommand {
    #[command(subcommand)]
    pub command: ReviewSessionSubcommand,
}

impl CmdExecutor for ReviewSessionCommand {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        self.command.execute(ctx).await
    }
}

/// Review session 子命令。
#[derive(Debug, Subcommand)]
pub enum ReviewSessionSubcommand {
    /// 开始一次 session。
    Start(ReviewSessionStartArgs),
    /// 完成一次 session。
    Complete(ReviewSessionCompleteArgs),
}

/// session start 参数。
#[derive(Debug, Args)]
pub struct ReviewSessionStartArgs {
    pub review_id: String,
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
    #[arg(long)]
    pub session_id: Option<String>,
}

impl CmdExecutor for ReviewSessionStartArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        let project_dir = workspace_fs::default_project_dir(self.project_dir)?;
        let output = start_review_session(StartReviewSessionOptions {
            project_dir,
            review_id: self.review_id,
            session_id: self.session_id,
            actor: "daedalus-cli".to_owned(),
        })?;
        print_review_session(&output, ctx.format);
        Ok(())
    }
}

/// session complete 参数。
#[derive(Debug, Args)]
pub struct ReviewSessionCompleteArgs {
    pub review_id: String,
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
    #[arg(long)]
    pub session_id: Option<String>,
    #[arg(long)]
    pub reason: String,
}

impl CmdExecutor for ReviewSessionCompleteArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        let project_dir = workspace_fs::default_project_dir(self.project_dir)?;
        let output = complete_review_session(CompleteReviewSessionOptions {
            project_dir,
            review_id: self.review_id,
            session_id: self.session_id,
            reason: self.reason,
            actor: "daedalus-cli".to_owned(),
        })?;
        print_review_session(&output, ctx.format);
        Ok(())
    }
}

impl CmdExecutor for ReviewSubcommand {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        match self {
            Self::Start(args) => args.execute(ctx).await,
            Self::List(args) => args.execute(ctx).await,
            Self::Show(args) => args.execute_show(ctx).await,
            Self::Complete(args) => {
                args.execute_with_lifecycle(ctx, ReviewLifecycle::Completed)
                    .await
            }
            Self::Abandon(args) => {
                args.execute_with_lifecycle(ctx, ReviewLifecycle::Abandoned)
                    .await
            }
            Self::Render(args) => args.execute_render(ctx).await,
            Self::Validate(args) => args.execute_validate(ctx).await,
            Self::Session(args) => args.execute(ctx).await,
        }
    }
}

impl CmdExecutor for ReviewSessionSubcommand {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        match self {
            Self::Start(args) => args.execute(ctx).await,
            Self::Complete(args) => args.execute(ctx).await,
        }
    }
}

fn resolve_review_target(
    project_dir: &std::path::Path,
    project: bool,
    topic: Option<String>,
) -> Result<ReviewTarget> {
    if project {
        return Ok(ReviewTarget::Project);
    }
    if let Some(topic) = topic {
        return Ok(ReviewTarget::Topic(topic));
    }
    let doc = state_toml::load_state_doc(&state_toml::state_path(project_dir))?;
    state_toml::active_topic(&doc)
        .map(ReviewTarget::Topic)
        .ok_or(DaedalusError::NoActiveTopic)
}
