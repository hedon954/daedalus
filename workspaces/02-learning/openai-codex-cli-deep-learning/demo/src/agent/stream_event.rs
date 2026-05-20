#[derive(Debug)]
pub enum StreamEvent {
    Started,
    ThinkingDelta(String),
    TextDelta(String),
    ToolCallStarted {
        index: i64,
        call_id: String,
        name: String,
    },
    ToolCallArgumentsDelta {
        index: i64,
        call_id: String,
        name: Option<String>,
        delta: String,
    },
    ToolCallFinished {
        index: i64,
        call_id: String,
        name: String,
        arguments: String,
    },
    Completed,
    Error(String),
}
