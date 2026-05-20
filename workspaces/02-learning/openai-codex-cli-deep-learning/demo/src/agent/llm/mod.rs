pub mod openai;

use std::pin::Pin;

use async_trait::async_trait;
use futures_core::Stream;

use crate::agent::stream_event::StreamEvent;

pub type EventStream = Pin<Box<dyn Stream<Item = anyhow::Result<StreamEvent>> + Send + 'static>>;

pub struct LLmRequest {
    pub messages: Vec<serde_json::Value>,
}

#[async_trait]
pub trait Llm {
    async fn stream(&self, request: LLmRequest) -> anyhow::Result<EventStream>;
}
