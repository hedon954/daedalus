//! Agent CLI 命令集合。

pub mod init;
pub mod knowledge;
pub mod migrate;
pub mod review;
pub mod state;
pub mod task;
pub mod topic;
pub mod validate;

use clap::Subcommand;
use enum_dispatch::enum_dispatch;

pub use init::{InitCommand, InitKind, InitRepoLearningArgs};
pub use knowledge::{
    KnowledgeCommand, KnowledgeExportArgs, KnowledgeListArgs, KnowledgePromoteArgs,
    KnowledgeSubcommand, KnowledgeTargetArg, KnowledgeTopicArgs,
};
pub use migrate::{MigrateCommand, MigrateRepoLearningArgs, MigrateSubcommand};
pub use review::{
    ReviewCloseArgs, ReviewCommand, ReviewListArgs, ReviewModeArg, ReviewSessionCommand,
    ReviewSessionCompleteArgs, ReviewSessionStartArgs, ReviewSessionSubcommand, ReviewShowArgs,
    ReviewStartArgs, ReviewSubcommand,
};
pub use state::{
    BlockArgs, CompleteArgs, EnterArgs, RenderArgs, ResumeArgs, RollbackArgs, StateCommand,
    StateSubcommand,
};
pub use task::{TaskAbandonArgs, TaskCommand, TaskCompleteArgs, TaskSubcommand};
pub use topic::{
    TopicActivateArgs, TopicCloseArgs, TopicCommand, TopicListArgs, TopicNewArgs, TopicSubcommand,
    TopicValidateArgs,
};
pub use validate::ValidateCommand;

/// daedalus 顶层命令。
#[derive(Debug, Subcommand)]
#[enum_dispatch(CmdExecutor)]
pub enum Command {
    /// 初始化学习任务。
    Init(InitCommand),
    /// 迁移学习 workspace。
    Migrate(MigrateCommand),
    /// 管理复习计划。
    Review(ReviewCommand),
    /// 管理知识体系萃取和晋升。
    Knowledge(KnowledgeCommand),
    /// 状态流转和状态渲染命令。
    State(StateCommand),
    /// 完成或放弃学习任务。
    Task(TaskCommand),
    /// 管理 repo learning topics。
    Topic(TopicCommand),
    /// 校验学习任务 workspace。
    Validate(ValidateCommand),
}
