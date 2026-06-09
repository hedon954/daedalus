use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
};

use crate::cli::app::{CliState, UiLine, UiMode};

pub fn draw(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(area);

    draw_header(frame, chunks[0], state);
    draw_log(frame, chunks[1], state);
    draw_input_or_approval(frame, chunks[2], state);
    draw_footer(frame, chunks[3], state);
}

fn draw_header(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    let text = format!("Codex Mini Agent | mode: {:?}", state.mode);
    let widget = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
    frame.render_widget(widget, area);
}

fn draw_log(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    let text = state
        .lines
        .iter()
        .map(UiLine::text)
        .collect::<Vec<_>>()
        .join("\n");

    let widget = Paragraph::new(text)
        .block(Block::default().title("Events").borders(Borders::ALL))
        .scroll((state.log_scroll, 0));

    frame.render_widget(widget, area);
}

fn draw_input_or_approval(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    match state.mode {
        UiMode::PendingApproval => draw_approval(frame, area, state),
        _ => draw_input(frame, area, state),
    }
}

fn draw_input(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    let widget = Paragraph::new(state.input.as_str())
        .block(Block::default().title("Prompt").borders(Borders::ALL));

    frame.render_widget(widget, area);
}

fn draw_approval(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    let text = match &state.pending_approval {
        Some(pending) => format!(
            "Approval required\nreason: {}\nscope: {}\n[a] once  [s] session  [r] reject",
            pending.reason, pending.scope_summary
        ),
        None => "Approval required, but missing pending approval detail".to_string(),
    };

    let widget =
        Paragraph::new(text).block(Block::default().title("Approval").borders(Borders::ALL));

    frame.render_widget(widget, area);
}

fn draw_footer(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    let keys = match state.mode {
        UiMode::PendingApproval => "a approve once | s approve session | r reject | q quit",
        _ => "enter submit | backspace delete | q quit",
    };

    let widget = Paragraph::new(keys).block(Block::default().borders(Borders::ALL));

    frame.render_widget(widget, area);
}
