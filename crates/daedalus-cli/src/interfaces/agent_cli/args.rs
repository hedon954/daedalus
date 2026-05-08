use clap::{Parser, ValueEnum};

use crate::interfaces::agent_cli::commands::Command;

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
