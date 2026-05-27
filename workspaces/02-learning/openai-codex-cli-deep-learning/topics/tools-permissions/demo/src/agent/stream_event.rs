use std::pin::Pin;

use futures_core::Stream;

pub type EventStream = Pin<Box<dyn Stream<Item = anyhow::Result<StreamEvent>> + Send + 'static>>;

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
    Completed,
    Error(String),
}

#[derive(Debug)]
pub struct ToolCallFinished {
    pub index: i64,
    pub call_id: String,
    pub name: String,
    pub arguments: String,
}
