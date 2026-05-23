use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};

use crate::application::knowledge::{
    ExportKnowledgeOptions, ExtractKnowledgeOptions, PromoteKnowledgeOptions,
    export_project_knowledge, extract_topic_knowledge, list_knowledge, promote_topic_knowledge,
    validate_knowledge,
};
use crate::domain::{DaedalusError, KnowledgePromotionTarget, Result};
use crate::infrastructure::{state_toml, workspace_fs};
use crate::interfaces::agent_cli::context::AgentCliContext;
use crate::interfaces::agent_cli::executor::CmdExecutor;
use crate::interfaces::agent_cli::presenter::{
    print_knowledge, print_knowledge_list, print_knowledge_validation,
};

/// Knowledge 命令。
#[derive(Debug, Args)]
pub struct KnowledgeCommand {
    #[command(subcommand)]
    pub command: KnowledgeSubcommand,
}

impl CmdExecutor for KnowledgeCommand {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        self.command.execute(ctx).await
    }
}

/// Knowledge 子命令。
#[derive(Debug, Subcommand)]
pub enum KnowledgeSubcommand {
    /// 创建 topic knowledge extraction candidates。
    Extract(KnowledgeTopicArgs),
    /// 将 topic candidates 晋升到 shared。
    Promote(KnowledgePromoteArgs),
    /// 创建 knowledge-base candidate。
    Export(KnowledgeExportArgs),
    /// 列出 knowledge-system 产物。
    List(KnowledgeListArgs),
    /// 校验 knowledge-system 产物。
    Validate(KnowledgeListArgs),
}

/// promotion target CLI 参数。
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum KnowledgeTargetArg {
    Shared,
    KnowledgeBase,
}

impl From<KnowledgeTargetArg> for KnowledgePromotionTarget {
    fn from(value: KnowledgeTargetArg) -> Self {
        match value {
            KnowledgeTargetArg::Shared => KnowledgePromotionTarget::Shared,
            KnowledgeTargetArg::KnowledgeBase => KnowledgePromotionTarget::KnowledgeBase,
        }
    }
}

/// topic knowledge 参数。
#[derive(Debug, Args)]
pub struct KnowledgeTopicArgs {
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
    #[arg(long)]
    pub topic: Option<String>,
}

impl CmdExecutor for KnowledgeTopicArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        let project_dir = workspace_fs::default_project_dir(self.project_dir)?;
        let topic_slug = resolve_topic(&project_dir, self.topic)?;
        let output = extract_topic_knowledge(ExtractKnowledgeOptions {
            repo_root: ctx.repo_root,
            project_dir,
            topic_slug,
        })?;
        print_knowledge(&output, ctx.format);
        Ok(())
    }
}

/// promote 参数。
#[derive(Debug, Args)]
pub struct KnowledgePromoteArgs {
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
    #[arg(long)]
    pub topic: Option<String>,
    #[arg(long, value_enum, default_value_t = KnowledgeTargetArg::Shared)]
    pub to: KnowledgeTargetArg,
}

impl CmdExecutor for KnowledgePromoteArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        let target: KnowledgePromotionTarget = self.to.into();
        if target != KnowledgePromotionTarget::Shared {
            return Err(DaedalusError::InvalidKnowledgeOperation(
                "knowledge promote currently supports --to shared; use knowledge export for knowledge-base candidates".to_owned(),
            ));
        }
        let project_dir = workspace_fs::default_project_dir(self.project_dir)?;
        let topic_slug = resolve_topic(&project_dir, self.topic)?;
        let output = promote_topic_knowledge(PromoteKnowledgeOptions {
            repo_root: ctx.repo_root,
            project_dir,
            topic_slug,
        })?;
        print_knowledge(&output, ctx.format);
        Ok(())
    }
}

/// export 参数。
#[derive(Debug, Args)]
pub struct KnowledgeExportArgs {
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = KnowledgeTargetArg::KnowledgeBase)]
    pub to: KnowledgeTargetArg,
}

impl CmdExecutor for KnowledgeExportArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        let target: KnowledgePromotionTarget = self.to.into();
        if target != KnowledgePromotionTarget::KnowledgeBase {
            return Err(DaedalusError::InvalidKnowledgeOperation(
                "knowledge export currently supports --to knowledge-base".to_owned(),
            ));
        }
        let project_dir = workspace_fs::default_project_dir(self.project_dir)?;
        let output = export_project_knowledge(ExportKnowledgeOptions {
            repo_root: ctx.repo_root,
            project_dir,
        })?;
        print_knowledge(&output, ctx.format);
        Ok(())
    }
}

/// list / validate 参数。
#[derive(Debug, Args)]
pub struct KnowledgeListArgs {
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
}

impl KnowledgeListArgs {
    async fn execute_list(self, ctx: AgentCliContext) -> Result<()> {
        let project_dir = workspace_fs::default_project_dir(self.project_dir)?;
        let output = list_knowledge(project_dir, ctx.repo_root)?;
        print_knowledge_list(&output, ctx.format);
        Ok(())
    }

    async fn execute_validate(self, ctx: AgentCliContext) -> Result<()> {
        let project_dir = workspace_fs::default_project_dir(self.project_dir)?;
        let output = validate_knowledge(project_dir, ctx.repo_root)?;
        if !output.is_ok() {
            return Err(DaedalusError::WorkspaceValidationFailed(output.issues));
        }
        print_knowledge_validation(&output, ctx.format);
        Ok(())
    }
}

impl CmdExecutor for KnowledgeSubcommand {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        match self {
            Self::Extract(args) => args.execute(ctx).await,
            Self::Promote(args) => args.execute(ctx).await,
            Self::Export(args) => args.execute(ctx).await,
            Self::List(args) => args.execute_list(ctx).await,
            Self::Validate(args) => args.execute_validate(ctx).await,
        }
    }
}

fn resolve_topic(project_dir: &std::path::Path, topic: Option<String>) -> Result<String> {
    if let Some(topic) = topic {
        return Ok(topic);
    }
    let doc = state_toml::load_state_doc(&state_toml::state_path(project_dir))?;
    state_toml::active_topic(&doc).ok_or(DaedalusError::NoActiveTopic)
}
