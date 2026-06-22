use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::application::migrate::{
    MigrateCourseLearningOptions, MigrateRepoLearningOptions, migrate_course_learning,
    migrate_repo_learning,
};
use crate::domain::Result;
use crate::infrastructure::workspace_fs;
use crate::interfaces::agent_cli::context::AgentCliContext;
use crate::interfaces::agent_cli::executor::CmdExecutor;
use crate::interfaces::agent_cli::presenter::print_course_learning_migration;
use crate::interfaces::agent_cli::presenter::print_migration;

/// 迁移命令。
#[derive(Debug, Args)]
pub struct MigrateCommand {
    #[command(subcommand)]
    pub command: MigrateSubcommand,
}

impl CmdExecutor for MigrateCommand {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        self.command.execute(ctx).await
    }
}

/// 迁移子命令。
#[derive(Debug, Subcommand)]
pub enum MigrateSubcommand {
    /// 将旧 repo-learning workspace 迁移为 multi-topic project。
    RepoLearningMultiTopic(MigrateRepoLearningArgs),
    /// 将已有 project 迁移为 course-learning 类型。
    CourseLearning(MigrateCourseLearningArgs),
}

impl CmdExecutor for MigrateSubcommand {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        match self {
            Self::RepoLearningMultiTopic(args) => args.execute(ctx).await,
            Self::CourseLearning(args) => args.execute(ctx).await,
        }
    }
}

/// repo-learning multi-topic 迁移参数。
#[derive(Debug, Args)]
pub struct MigrateRepoLearningArgs {
    /// 旧 workspace 目录；缺省时由当前目录或唯一 active project 推导。
    pub task_dir: Option<PathBuf>,
    /// 迁移后的初始 topic slug。
    #[arg(long)]
    pub topic: String,
    /// 迁移后的初始 topic title。
    #[arg(long)]
    pub title: String,
    /// 执行迁移；缺省只 dry-run。
    #[arg(long)]
    pub execute: bool,
}

/// course-learning 类型迁移参数。
#[derive(Debug, Args)]
pub struct MigrateCourseLearningArgs {
    /// 要迁移的 project 目录；缺省时由当前目录或唯一 active project 推导。
    pub project_dir: Option<PathBuf>,
    /// 课程主页或课程材料入口。
    #[arg(long)]
    pub course_url: String,
    /// 执行迁移；缺省只 dry-run。
    #[arg(long)]
    pub execute: bool,
}

impl CmdExecutor for MigrateRepoLearningArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        let task_dir = workspace_fs::default_project_dir(self.task_dir)?;
        let output = migrate_repo_learning(MigrateRepoLearningOptions {
            repo_root: ctx.repo_root,
            task_dir,
            topic_slug: self.topic,
            topic_title: self.title,
            execute: self.execute,
        })?;
        print_migration(&output, ctx.format);
        Ok(())
    }
}

impl CmdExecutor for MigrateCourseLearningArgs {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        let project_dir = workspace_fs::default_project_dir(self.project_dir)?;
        let output = migrate_course_learning(MigrateCourseLearningOptions {
            repo_root: ctx.repo_root,
            project_dir,
            course_url: self.course_url,
            execute: self.execute,
        })?;
        print_course_learning_migration(&output, ctx.format);
        Ok(())
    }
}
