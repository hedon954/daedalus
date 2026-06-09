#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliState {
    pub mode: UiMode,
    pub input: String,
    pub lines: Vec<UiLine>,
    pub log_scroll: u16,
    pub active_prompt: Option<String>,
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
    System(String),
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
            active_prompt: None,
            pending_approval: None,
            should_quit: false,
        }
    }

    pub fn push_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn push_line(&mut self, line: UiLine) {
        self.lines.push(line);
        self.scroll_to_bottom();
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

    pub fn scroll_log_page_up(&mut self) {
        self.log_scroll = self.log_scroll.saturating_sub(8);
    }

    pub fn scroll_log_page_down(&mut self) {
        let max = self.lines.len().saturating_sub(1) as u16;
        self.log_scroll = self.log_scroll.saturating_add(8).min(max);
    }

    pub fn scroll_log_home(&mut self) {
        self.log_scroll = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.log_scroll = self.lines.len().saturating_sub(1) as u16;
    }

    pub fn set_running(&mut self, prompt: String) {
        self.active_prompt = Some(prompt);
        self.mode = UiMode::RunningAgent;
    }

    pub fn set_ready(&mut self) {
        self.active_prompt = None;
        self.pending_approval = None;
        self.mode = UiMode::EditingPrompt;
    }

    pub fn status_text(&self) -> &'static str {
        match self.mode {
            UiMode::EditingPrompt => "ready",
            UiMode::RunningAgent => "running",
            UiMode::PendingApproval => "approval",
        }
    }

    pub fn input_hint(&self) -> &'static str {
        match self.mode {
            UiMode::EditingPrompt => "Type a prompt. Enter submits.",
            UiMode::RunningAgent => "Agent is running. Press q to quit.",
            UiMode::PendingApproval => "Approval required. Choose once, session, or reject.",
        }
    }
}

impl UiLine {
    pub fn text(&self) -> String {
        match self {
            UiLine::System(text) => format!("system: {text}"),
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

#[cfg(test)]
mod tests {
    use super::{CliState, UiLine, UiMode};

    #[test]
    fn take_prompt_ignores_blank_input_and_preserves_it() {
        let mut state = CliState::new();
        state.input = "   ".to_string();

        assert_eq!(state.take_prompt(), None);
        assert_eq!(state.input, "   ");
    }

    #[test]
    fn take_prompt_moves_non_blank_input_without_clone() {
        let mut state = CliState::new();
        state.input = "hello".to_string();

        assert_eq!(state.take_prompt(), Some("hello".to_string()));
        assert!(state.input.is_empty());
    }

    #[test]
    fn model_and_thinking_deltas_merge_only_with_same_last_line_kind() {
        let mut state = CliState::new();

        state.append_model_delta("he".to_string());
        state.append_model_delta("llo".to_string());
        state.append_thinking_delta("plan".to_string());
        state.append_model_delta("done".to_string());

        assert_eq!(
            state.lines,
            vec![
                UiLine::Model("hello".to_string()),
                UiLine::Thinking("plan".to_string()),
                UiLine::Model("done".to_string()),
            ]
        );
    }

    #[test]
    fn running_and_ready_modes_manage_active_prompt() {
        let mut state = CliState::new();

        state.set_running("do work".to_string());
        assert_eq!(state.mode, UiMode::RunningAgent);
        assert_eq!(state.active_prompt.as_deref(), Some("do work"));

        state.set_ready();
        assert_eq!(state.mode, UiMode::EditingPrompt);
        assert_eq!(state.active_prompt, None);
    }
}
