use std::path::PathBuf;

use clap::Args;
use tracing::debug;

use crate::application::validate_workspace::validate_workspace;
use crate::domain::{DaedalusError, Result};
use crate::infrastructure::workspace_fs;
use crate::interfaces::agent_cli::context::AgentCliContext;
use crate::interfaces::agent_cli::executor::CmdExecutor;
use crate::interfaces::agent_cli::presenter::print_validation;

/// 校验学习任务 workspace 命令参数。
#[derive(Debug, Args)]
pub struct ValidateCommand {
    /// 显式学习任务目录；缺省时由当前目录或唯一 active task 推导。
    pub task_dir: Option<PathBuf>,
}

impl CmdExecutor for ValidateCommand {
    async fn execute(self, ctx: AgentCliContext) -> Result<()> {
        debug!("校验学习任务 workspace");
        let task_dir = workspace_fs::default_task_dir(self.task_dir)?;
        let output = validate_workspace(&task_dir, Some(&ctx.repo_root))?;
        if !output.is_ok() {
            return Err(DaedalusError::WorkspaceValidationFailed(output.issues));
        }
        print_validation(&output, ctx.format);
        Ok(())
    }
}
