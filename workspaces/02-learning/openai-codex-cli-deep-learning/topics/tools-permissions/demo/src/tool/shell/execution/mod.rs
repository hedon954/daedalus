pub mod simulated_execution_runner;

use async_trait::async_trait;

use crate::model::{
    command_request::CommandRequest, event::ExecutionAttempt, execution::ExecutionResult,
};

/// 命令运行器。
///
/// Phase 1 使用模拟 runner 验证状态机；Phase 2 可以把同一接口替换为真实
/// OS sandbox + local execution 组合。
#[async_trait]
pub trait ExecutionRunner: Send + Sync {
    /// 运行命令
    async fn run(&self, request: &CommandRequest, attempt: &ExecutionAttempt) -> ExecutionResult;
}
