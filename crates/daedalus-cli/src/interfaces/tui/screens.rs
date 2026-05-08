use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Frame, Rect};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::interfaces::tui::app::TuiOverview;

/// 绘制 TUI 只读总览页面。
pub fn draw_overview(frame: &mut Frame<'_>, area: Rect, overview: &TuiOverview) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Percentage(30),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let current = Paragraph::new(format!(
        "Task: {}\nCurrent phase: {}\nStatus: {}\nPress q to quit.",
        overview.task_name, overview.current_phase, overview.current_status
    ))
    .block(Block::default().title("Current").borders(Borders::ALL));
    frame.render_widget(current, chunks[0]);

    render_list(
        frame,
        chunks[1],
        "Missing Artifacts",
        &overview.missing_artifacts,
    );
    render_list(frame, chunks[2], "Todo", &overview.todo_summary);
    render_list(
        frame,
        chunks[3],
        "Recent Transitions",
        &overview.recent_transitions,
    );
}

fn render_list(frame: &mut Frame<'_>, area: Rect, title: &str, values: &[String]) {
    let items: Vec<ListItem<'_>> = if values.is_empty() {
        vec![ListItem::new("None")]
    } else {
        values
            .iter()
            .map(|value| ListItem::new(value.as_str()))
            .collect()
    };
    let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(list, area);
}
