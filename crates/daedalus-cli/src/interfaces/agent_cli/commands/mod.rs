//! Agent CLI 命令集合。

pub mod init;
pub mod state;
pub mod task;
pub mod validate;

use clap::Subcommand;
use enum_dispatch::enum_dispatch;

pub use init::{InitCommand, InitKind, InitRepoLearningArgs};
pub use state::{
    BlockArgs, CompleteArgs, EnterArgs, RenderArgs, ResumeArgs, StateCommand, StateSubcommand,
};
pub use task::{TaskAbandonArgs, TaskCommand, TaskCompleteArgs, TaskSubcommand};
pub use validate::ValidateCommand;

/// daedalus 顶层命令。
#[derive(Debug, Subcommand)]
#[enum_dispatch(CmdExecutor)]
pub enum Command {
    /// 初始化学习任务。
    Init(InitCommand),
    /// 状态流转和状态渲染命令。
    State(StateCommand),
    /// 完成或放弃学习任务。
    Task(TaskCommand),
    /// 校验学习任务 workspace。
    Validate(ValidateCommand),
}
