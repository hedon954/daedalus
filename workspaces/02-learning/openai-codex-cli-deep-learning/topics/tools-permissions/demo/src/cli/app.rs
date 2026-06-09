#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliState {
    pub mode: UiMode,
    pub input: String,
    pub lines: Vec<UiLine>,
    pub log_scroll: u16,
    pub pending_approval: Option<PendingApprovalView>,
    pub should_quit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiMode {
    EditingPrompt,
    RunningAgent,
    PendingApproval,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiLine {
    User(String),
    Thinking(String),
    Model(String),
    Tool(String),
    Command(String),
    Approval(String),
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingApprovalView {
    pub approval_id: String,
    pub reason: String,
    pub scope_summary: String,
}

impl CliState {
    pub fn new() -> Self {
        Self {
            mode: UiMode::EditingPrompt,
            input: String::new(),
            lines: Vec::new(),
            log_scroll: 0,
            pending_approval: None,
            should_quit: false,
        }
    }

    pub fn push_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn push_line(&mut self, line: UiLine) {
        self.lines.push(line);
    }

    pub fn backspace(&mut self) {
        self.input.pop();
    }

    pub fn take_prompt(&mut self) -> Option<String> {
        if self.input.trim().is_empty() {
            return None;
        }
        Some(std::mem::take(&mut self.input))
    }

    pub fn append_model_delta(&mut self, delta: String) {
        match self.lines.last_mut() {
            Some(UiLine::Model(text)) => text.push_str(&delta),
            _ => self.lines.push(UiLine::Model(delta)),
        }
        self.scroll_to_bottom()
    }

    pub fn append_thinking_delta(&mut self, delta: String) {
        match self.lines.last_mut() {
            Some(UiLine::Thinking(text)) => text.push_str(&delta),
            _ => self.lines.push(UiLine::Thinking(delta)),
        }
        self.scroll_to_bottom()
    }

    pub fn scroll_log_up(&mut self) {
        self.log_scroll = self.log_scroll.saturating_sub(1);
    }

    pub fn scroll_log_down(&mut self) {
        let max = self.lines.len().saturating_sub(1) as u16;
        self.log_scroll = self.log_scroll.saturating_add(1).min(max);
    }

    pub fn scroll_to_bottom(&mut self) {
        self.log_scroll = self.lines.len().saturating_sub(1) as u16;
    }
}

impl UiLine {
    pub fn text(&self) -> String {
        match self {
            UiLine::User(text) => format!("> {text}"),
            UiLine::Thinking(text) => format!("[thinking] {text}"),
            UiLine::Model(text) => text.clone(),
            UiLine::Tool(text) => format!("[tool] {text}"),
            UiLine::Command(text) => format!("[command] {text}"),
            UiLine::Approval(text) => format!("[approval] {text}"),
            UiLine::Error(text) => format!("[error] {text}"),
        }
    }
}
