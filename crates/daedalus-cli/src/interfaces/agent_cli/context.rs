use std::path::PathBuf;

use crate::domain::{DaedalusError, Result};
use crate::infrastructure::workspace_fs;
use crate::interfaces::agent_cli::args::OutputFormat;

/// Agent CLI 命令执行上下文。
///
/// 顶层入口只计算一次项目根目录和输出格式，具体命令通过上下文复用这些信息。
#[derive(Debug, Clone)]
pub struct AgentCliContext {
    /// 输出格式。
    pub format: OutputFormat,
    /// daedalus 项目根目录。
    pub repo_root: PathBuf,
}

impl AgentCliContext {
    /// 从当前工作目录发现 daedalus 项目根目录，并创建执行上下文。
    pub fn discover(format: OutputFormat) -> Result<Self> {
        let cwd = std::env::current_dir().map_err(|source| DaedalusError::Io {
            path: ".".into(),
            source,
        })?;
        let repo_root = workspace_fs::repo_root_from(&cwd)?;
        Ok(Self { format, repo_root })
    }
}
