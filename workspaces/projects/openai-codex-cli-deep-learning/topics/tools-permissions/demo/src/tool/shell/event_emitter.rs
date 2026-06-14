use crate::{
    agent::{react::EventSender, stream_event::StreamEvent},
    model::{
        approval::ApprovalScope,
        event::{ExecutionAttempt, RetryDecision},
    },
    tool::shell::approval::ToolCallContext,
};

pub struct CommandEventEmitter {
    tx: EventSender,
    context: ToolCallContext,
}

impl CommandEventEmitter {
    pub fn new(tx: EventSender, context: ToolCallContext) -> Self {
        Self { tx, context }
    }

    pub async fn needs_approval(
        &self,
        approval_id: String,
        reason: String,
        scope: ApprovalScope,
    ) -> anyhow::Result<()> {
        self.tx
            .send(Ok(StreamEvent::CommandNeedsApproval {
                approval_id: approval_id,
                index: self.context.index,
                call_id: self.context.call_id.clone(),
                name: self.context.tool_name.clone(),
                reason,
                scope,
            }))
            .await?;

        Ok(())
    }

    pub async fn execution_started(&self, attempt: &ExecutionAttempt) {
        _ = self
            .tx
            .send(Ok(StreamEvent::CommandExecutionStarted {
                index: self.context.index,
                call_id: self.context.call_id.clone(),
                name: self.context.tool_name.clone(),
                attempt: attempt.clone(),
            }))
            .await;
    }
    pub async fn execution_finished(&self, attempt: &ExecutionAttempt, output: String) {
        _ = self
            .tx
            .send(Ok(StreamEvent::CommandExecutionFinished {
                index: self.context.index,
                call_id: self.context.call_id.clone(),
                name: self.context.tool_name.clone(),
                attempt: attempt.clone(),
                output,
            }))
            .await;
    }

    pub async fn execution_failed(&self, attempt: &ExecutionAttempt, error: String) {
        _ = self
            .tx
            .send(Ok(StreamEvent::CommandExecutionFailed {
                index: self.context.index,
                call_id: self.context.call_id.clone(),
                name: self.context.tool_name.clone(),
                attempt: attempt.clone(),
                error,
            }))
            .await;
    }

    pub async fn retry_evaluated(&self, decision: RetryDecision) {
        _ = self
            .tx
            .send(Ok(StreamEvent::CommandRetryEvaluated {
                index: self.context.index,
                call_id: self.context.call_id.clone(),
                name: self.context.tool_name.clone(),
                decision,
            }))
            .await;
    }
}
