pub mod openai;

#[cfg(test)]
pub(crate) mod fake;

use async_trait::async_trait;

use crate::agent::stream_event::EventStream;

pub struct LLmRequest {
    pub messages: Vec<serde_json::Value>,
}

#[async_trait]
pub trait Llm: Send + Sync {
    async fn stream(&self, request: LLmRequest) -> anyhow::Result<EventStream>;
}
