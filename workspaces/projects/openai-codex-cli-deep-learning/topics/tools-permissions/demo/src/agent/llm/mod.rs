pub mod openai;

#[cfg(test)]
pub(crate) mod fake;

use async_trait::async_trait;

use crate::agent::stream_event::EventStream;

/// 传给 LLM 的一轮上下文。
///
/// 当前直接沿用 OpenAI-compatible chat messages，避免在 demo 早期重复设计一套消息协议。
pub struct LLmRequest {
    pub messages: Vec<serde_json::Value>,
}

/// LLM streaming 抽象。
///
/// ReAct loop 只依赖统一的 `EventStream`，不关心底层是 DeepSeek、
/// OpenAI-compatible API，还是测试用 FakeLlm。
#[async_trait]
pub trait Llm: Send + Sync {
    async fn stream(&self, request: LLmRequest) -> anyhow::Result<EventStream>;
}
