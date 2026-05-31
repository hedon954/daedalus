pub mod simulated_sandbox_runner;

use crate::model::{
    command_request::CommandRequest, event::ExecutionAttempt, execution::ExecutionResult,
};

/// 沙箱运行器。
///
/// Phase 1 使用模拟 runner 验证状态机；Phase 2 可以把同一接口替换为真实 OS sandbox。
pub trait SandboxRunner {
    /// 运行命令
    fn run(&self, request: &CommandRequest, attempt: &ExecutionAttempt) -> ExecutionResult;
}
