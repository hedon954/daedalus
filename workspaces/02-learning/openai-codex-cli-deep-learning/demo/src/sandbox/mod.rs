pub mod simulated_sandbox_runner;

use crate::model::{
    command_request::CommandRequest, event::ExecutionAttempt, execution::ExecutionResult,
};

/// 沙箱运行器
pub trait SandboxRunner {
    /// 运行命令
    fn run(&self, request: &CommandRequest, attempt: &ExecutionAttempt) -> ExecutionResult;
}
