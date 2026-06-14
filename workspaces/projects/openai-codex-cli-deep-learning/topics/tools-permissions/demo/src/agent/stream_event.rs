use std::pin::Pin;

use futures_core::Stream;

use crate::model::{
    approval::ApprovalScope,
    event::{ExecutionAttempt, RetryDecision},
};

pub type EventStream = Pin<Box<dyn Stream<Item = anyhow::Result<StreamEvent>> + Send + 'static>>;

/// Agent 对外暴露的流式事件。
///
/// 这些事件服务于 UI / logs / tests：模型输出、tool choice、tool run 和最终完成
/// 都应该能被外部观察。权限审批事件后续会继续补齐。
#[derive(Debug)]
pub enum StreamEvent {
    Started,
    ThinkingDelta(String),
    TextDelta(String),
    ToolCallArgumentsDelta {
        index: i64,
        call_id: String,
        name: Option<String>,
        delta: String,
    },
    ToolCallFinished(ToolCallFinished),
    ToolRunStarted {
        index: i64,
        call_id: String,
        name: String,
        arguments: String,
    },
    ToolRunFinished {
        index: i64,
        call_id: String,
        name: String,
        output: String,
    },
    ToolRunFailed {
        index: i64,
        call_id: String,
        name: String,
        error: String,
    },
    CommandNeedsApproval {
        approval_id: String,
        index: i64,
        call_id: String,
        name: String,
        reason: String,
        scope: ApprovalScope,
    },
    CommandExecutionStarted {
        index: i64,
        call_id: String,
        name: String,
        attempt: ExecutionAttempt,
    },
    CommandExecutionFinished {
        index: i64,
        call_id: String,
        name: String,
        attempt: ExecutionAttempt,
        output: String,
    },
    CommandExecutionFailed {
        index: i64,
        call_id: String,
        name: String,
        attempt: ExecutionAttempt,
        error: String,
    },
    CommandRetryEvaluated {
        index: i64,
        call_id: String,
        name: String,
        decision: RetryDecision,
    },
    Completed,
    Error(String),
}

/// 模型已经完成的一次 tool call。
///
/// 它代表“模型选择了哪个工具以及完整 arguments”，不代表工具已经执行。
#[derive(Debug, Clone)]
pub struct ToolCallFinished {
    pub index: i64,
    pub call_id: String,
    pub name: String,
    pub arguments: String,
}
