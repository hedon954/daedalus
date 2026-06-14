use crate::{
    agent::{
        react::EventSender,
        stream_event::{StreamEvent, ToolCallFinished},
    },
    tool::runtime::ToolRuntimeResult,
};

pub struct ToolEventEmitter {
    tx: EventSender,
    call: ToolCallFinished,
}

impl ToolEventEmitter {
    pub fn new(tx: EventSender, call: ToolCallFinished) -> Self {
        Self { tx, call }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        _ = self
            .tx
            .send(Ok(StreamEvent::ToolRunStarted {
                index: self.call.index,
                call_id: self.call.call_id.clone(),
                name: self.call.name.clone(),
                arguments: self.call.arguments.clone(),
            }))
            .await?;
        Ok(())
    }

    pub async fn end(&self, result: &ToolRuntimeResult) {
        match result {
            ToolRuntimeResult::Finished { output } => self.finished(output.to_string()).await,
            ToolRuntimeResult::Failed { error } => self.failed(error.to_string()).await,
            ToolRuntimeResult::Denied { reason } => self.failed(reason.to_string()).await,
        }
    }

    async fn finished(&self, output: String) {
        _ = self
            .tx
            .send(Ok(StreamEvent::ToolRunFinished {
                index: self.call.index,
                call_id: self.call.call_id.clone(),
                name: self.call.name.clone(),
                output,
            }))
            .await;
    }
    async fn failed(&self, error: String) {
        _ = self
            .tx
            .send(Ok(StreamEvent::ToolRunFailed {
                index: self.call.index,
                call_id: self.call.call_id.clone(),
                name: self.call.name.clone(),
                error,
            }))
            .await;
    }
}
