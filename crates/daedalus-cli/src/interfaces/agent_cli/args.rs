use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

/// daedalus Agent-friendly CLI 的顶层参数。
#[derive(Debug, Parser)]
#[command(name = "daedalus", about = "Agent-friendly daedalus workspace CLI")]
pub struct Cli {
    /// 输出格式，默认是稳定文本。
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,

    /// 要执行的顶层命令。
    #[command(subcommand)]
    pub command: Command,
}

/// CLI 输出格式。
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum OutputFormat {
    /// 面向人和 Agent 的稳定文本输出。
    Text,
    /// 面向程序解析的 JSON 输出。
    Json,
}

/// daedalus 顶层命令。
#[derive(Debug, Subcommand)]
pub enum Command {
    /// 初始化学习任务。
    Init(InitCommand),
    /// 状态流转和状态渲染命令。
    State(StateCommand),
    /// 校验学习任务 workspace。
    Validate(TaskDirArgs),
}

/// 初始化命令。
#[derive(Debug, Args)]
pub struct InitCommand {
    /// 要初始化的学习任务类型。
    #[command(subcommand)]
    pub kind: InitKind,
}

/// 支持初始化的学习任务类型。
#[derive(Debug, Subcommand)]
pub enum InitKind {
    /// 初始化 repo learning 任务。
    RepoLearning(InitRepoLearningArgs),
}

/// `init repo-learning` 参数。
#[derive(Debug, Args)]
pub struct InitRepoLearningArgs {
    /// 学习任务名称。
    pub name: String,
    /// 是否允许已有 active task 时继续创建。
    #[arg(long)]
    pub allow_existing_active: bool,
    /// 使用绕过选项时的原因。
    #[arg(long)]
    pub reason: Option<String>,
}

/// 状态命令。
#[derive(Debug, Args)]
pub struct StateCommand {
    /// 具体状态子命令。
    #[command(subcommand)]
    pub command: StateSubcommand,
}

/// 状态子命令集合。
#[derive(Debug, Subcommand)]
pub enum StateSubcommand {
    /// 进入阶段。
    Enter(StageArgs),
    /// 完成阶段。
    Complete(CompleteArgs),
    /// 阻塞阶段。
    Block(BlockArgs),
    /// 恢复阶段。
    Resume(StageArgs),
    /// 重新渲染 `state.md`。
    Render(TaskDirArgs),
}

/// 通用阶段命令参数。
#[derive(Debug, Args)]
pub struct StageArgs {
    /// 阶段 ID。
    pub stage_id: String,
    /// 显式学习任务目录。
    #[arg(long)]
    pub task_dir: Option<PathBuf>,
    /// 状态流转原因。
    #[arg(long)]
    pub reason: Option<String>,
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

/// 可选学习任务目录参数。
#[derive(Debug, Args)]
pub struct TaskDirArgs {
    /// 显式学习任务目录；缺省时由当前目录或唯一 active task 推导。
    pub task_dir: Option<PathBuf>,
}
