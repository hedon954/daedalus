use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::application::topic::{
    ActivateTopicOptions, CloseTopicOptions, NewTopicOptions, activate_topic, close_topic,
    new_topic, validate_topic,
};
use crate::domain::{DaedalusError, Result, TopicLifecycle};
use crate::infrastructure::{state_toml, workspace_fs};
use crate::interfaces::agent_cli::context::AgentCliContext;
use crate::interfaces::agent_cli::executor::CmdExecutor;
use crate::interfaces::agent_cli::presenter::{
    print_topic, print_topic_list, print_topic_validation,
};

/// Topic 命令。
#[derive(Debug, Args)]
pub struct TopicCommand {
    #[command(subcommand)]
    pub command: TopicSubcommand,
}

impl CmdExecutor for TopicCommand {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        self.command.execute(ctx).await
    }
}

/// Topic 子命令。
#[derive(Debug, Subcommand)]
pub enum TopicSubcommand {
    /// 新建专题。
    New(TopicNewArgs),
    /// 列出专题。
    List(TopicListArgs),
    /// 激活专题。
    Activate(TopicActivateArgs),
    /// 完成专题。
    Complete(TopicCloseArgs),
    /// 主体学习完成，等待用户主动回顾。
    AwaitReflection(TopicCloseArgs),
    /// 放弃专题。
    Abandon(TopicCloseArgs),
    /// 校验专题。
    Validate(TopicValidateArgs),
}

/// 新建专题参数。
#[derive(Debug, Args)]
pub struct TopicNewArgs {
    pub slug: String,
    #[arg(long)]
    pub title: String,
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
}

impl CmdExecutor for TopicNewArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        let project_dir = workspace_fs::default_project_dir(self.project_dir)?;
        let output = new_topic(NewTopicOptions {
            repo_root: ctx.repo_root,
            project_dir,
            slug: self.slug,
            title: self.title,
            actor: "daedalus-cli".to_owned(),
        })?;
        print_topic(&output, ctx.format);
        Ok(())
    }
}

/// 列出专题参数。
#[derive(Debug, Args)]
pub struct TopicListArgs {
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
}

impl CmdExecutor for TopicListArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        let project_dir = workspace_fs::default_project_dir(self.project_dir)?;
        let doc = state_toml::load_state_doc(&state_toml::state_path(&project_dir))?;
        print_topic_list(&project_dir, &state_toml::topics(&doc), ctx.format);
        Ok(())
    }
}

/// 激活专题参数。
#[derive(Debug, Args)]
pub struct TopicActivateArgs {
    pub slug: String,
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
}

impl CmdExecutor for TopicActivateArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        let project_dir = workspace_fs::default_project_dir(self.project_dir)?;
        let output = activate_topic(ActivateTopicOptions {
            repo_root: ctx.repo_root,
            project_dir,
            slug: self.slug,
            actor: "daedalus-cli".to_owned(),
        })?;
        print_topic(&output, ctx.format);
        Ok(())
    }
}

/// 关闭专题参数。
#[derive(Debug, Args)]
pub struct TopicCloseArgs {
    pub slug: String,
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
    #[arg(long)]
    pub reason: String,
}

impl TopicCloseArgs {
    async fn execute_with_lifecycle(
        self,
        ctx: AgentCliContext,
        lifecycle: TopicLifecycle,
    ) -> Result<()> {
        let project_dir = workspace_fs::default_project_dir(self.project_dir)?;
        let output = close_topic(CloseTopicOptions {
            repo_root: ctx.repo_root,
            project_dir,
            slug: self.slug,
            lifecycle,
            reason: self.reason,
            actor: "daedalus-cli".to_owned(),
        })?;
        print_topic(&output, ctx.format);
        Ok(())
    }
}

/// 校验专题参数。
#[derive(Debug, Args)]
pub struct TopicValidateArgs {
    pub slug: String,
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
}

impl CmdExecutor for TopicValidateArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        let project_dir = workspace_fs::default_project_dir(self.project_dir)?;
        let issues = validate_topic(project_dir.clone(), self.slug.clone())?;
        if !issues.is_empty() {
            return Err(DaedalusError::WorkspaceValidationFailed(issues));
        }
        print_topic_validation(&project_dir, &self.slug, ctx.format);
        Ok(())
    }
}

impl CmdExecutor for TopicSubcommand {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        match self {
            Self::New(args) => args.execute(ctx).await,
            Self::List(args) => args.execute(ctx).await,
            Self::Activate(args) => args.execute(ctx).await,
            Self::Complete(args) => {
                args.execute_with_lifecycle(ctx, TopicLifecycle::Completed)
                    .await
            }
            Self::AwaitReflection(args) => {
                args.execute_with_lifecycle(ctx, TopicLifecycle::AwaitingReflection)
                    .await
            }
            Self::Abandon(args) => {
                args.execute_with_lifecycle(ctx, TopicLifecycle::Abandoned)
                    .await
            }
            Self::Validate(args) => args.execute(ctx).await,
        }
    }
}
