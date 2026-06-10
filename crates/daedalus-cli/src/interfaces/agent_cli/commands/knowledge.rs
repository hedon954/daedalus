use clap::{Args, Subcommand, ValueEnum};

use crate::application::knowledge::{
    create_knowledge_template, link_check_knowledge, list_knowledge, rebuild_knowledge_index,
    validate_knowledge,
};
use crate::domain::{DaedalusError, Result};
use crate::interfaces::agent_cli::context::AgentCliContext;
use crate::interfaces::agent_cli::executor::CmdExecutor;
use crate::interfaces::agent_cli::presenter::{
    print_knowledge, print_knowledge_list, print_knowledge_validation,
};

/// Knowledge 命令。
///
/// CLI 只提供确定性的知识库底座能力。知识萃取、晋升、重组等认知工作由
/// daedalus knowledge skills 完成。
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
    /// 生成知识条目模板。
    Template(KnowledgeTemplateArgs),
    /// 重建 knowledge-base/index.toml。
    Index,
    /// 列出 knowledge-base 条目。
    List,
    /// 检查 knowledge-base 本地链接。
    LinkCheck,
    /// 校验 knowledge-base 结构和条目最低质量门槛。
    Validate,
}

/// 知识条目类型。
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum KnowledgeKindArg {
    Concept,
    Skill,
    Pattern,
    Problem,
    Case,
    SourceMap,
    Tree,
    Drill,
}

impl KnowledgeKindArg {
    fn as_str(self) -> &'static str {
        match self {
            Self::Concept => "concept",
            Self::Skill => "skill",
            Self::Pattern => "pattern",
            Self::Problem => "problem",
            Self::Case => "case",
            Self::SourceMap => "source-map",
            Self::Tree => "tree",
            Self::Drill => "drill",
        }
    }
}

/// template 参数。
#[derive(Debug, Args)]
pub struct KnowledgeTemplateArgs {
    #[arg(value_enum)]
    pub kind: KnowledgeKindArg,
    pub slug: String,
    #[arg(long)]
    pub title: Option<String>,
}

impl CmdExecutor for KnowledgeTemplateArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        let output = create_knowledge_template(
            &ctx.repo_root,
            self.kind.as_str(),
            &self.slug,
            self.title.as_deref(),
        )?;
        print_knowledge(&output, ctx.format);
        Ok(())
    }
}

impl CmdExecutor for KnowledgeSubcommand {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        match self {
            Self::Template(args) => args.execute(ctx).await,
            Self::Index => {
                let output = rebuild_knowledge_index(&ctx.repo_root)?;
                print_knowledge(&output, ctx.format);
                Ok(())
            }
            Self::List => {
                let output = list_knowledge(ctx.repo_root)?;
                print_knowledge_list(&output, ctx.format);
                Ok(())
            }
            Self::LinkCheck => {
                let output = link_check_knowledge(ctx.repo_root)?;
                if !output.is_ok() {
                    return Err(DaedalusError::WorkspaceValidationFailed(output.issues));
                }
                print_knowledge_validation(&output, ctx.format);
                Ok(())
            }
            Self::Validate => {
                let output = validate_knowledge(ctx.repo_root)?;
                if !output.is_ok() {
                    return Err(DaedalusError::WorkspaceValidationFailed(output.issues));
                }
                print_knowledge_validation(&output, ctx.format);
                Ok(())
            }
        }
    }
}
