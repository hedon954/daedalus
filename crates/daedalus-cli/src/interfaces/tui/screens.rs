use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::{Color, Frame, Line, Modifier, Rect, Span, Style};
use ratatui::widgets::{
    Block, BorderType, Borders, Gauge, List, ListItem, Padding, Paragraph, Wrap,
};

use crate::interfaces::tui::app::TuiOverview;

/// 绘制 TUI 只读总览页面。
pub fn draw_overview(frame: &mut Frame<'_>, area: Rect, overview: &TuiOverview) {
    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Rgb(9, 12, 22))),
        area,
    );

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .margin(1)
        .split(area);

    render_header(frame, root[0], overview);
    render_current_panel(frame, root[1], overview);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(root[2]);
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
        .split(body[0]);
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(body[1]);

    render_list(
        frame,
        left[0],
        "Missing Artifacts",
        &overview.missing_artifacts,
        Color::Rgb(255, 184, 108),
        "All required artifacts are present.",
    );
    render_next_action(frame, left[1], overview);
    render_list(
        frame,
        right[0],
        "Todo Focus",
        &overview.todo_summary,
        Color::Rgb(139, 233, 253),
        "No open todo items.",
    );
    render_list(
        frame,
        right[1],
        "Recent Transitions",
        &overview.recent_transitions,
        Color::Rgb(189, 147, 249),
        "No transition history.",
    );

    render_footer(frame, root[3]);
}

fn render_header(frame: &mut Frame<'_>, area: Rect, overview: &TuiOverview) {
    let title = Line::from(vec![
        Span::styled(
            "DAEDALUS",
            Style::default()
                .fg(Color::Rgb(80, 250, 123))
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  learning cockpit  "),
        Span::styled(
            format!("[{}]", overview.lifecycle),
            Style::default()
                .fg(status_color(&overview.lifecycle))
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let header = Paragraph::new(title)
        .alignment(Alignment::Center)
        .block(chrome_block("  system online  ", Color::Rgb(80, 250, 123)));
    frame.render_widget(header, area);
}

fn render_current_panel(frame: &mut Frame<'_>, area: Rect, overview: &TuiOverview) {
    let progress = if overview.total_stage_count == 0 {
        0.0
    } else {
        overview.done_stage_count as f64 / overview.total_stage_count as f64
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Length(3)])
        .split(area);

    let status = Line::from(vec![
        Span::styled("Task  ", muted()),
        Span::styled(
            overview.task_name.as_str(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("    "),
        Span::styled("Phase  ", muted()),
        Span::styled(
            overview.current_phase.as_str(),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw("    "),
        Span::styled("Status  ", muted()),
        Span::styled(
            overview.current_status.as_str(),
            Style::default()
                .fg(status_color(&overview.current_status))
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("    "),
        Span::styled("Bucket  ", muted()),
        Span::styled(
            overview.workspace_bucket.as_str(),
            Style::default().fg(Color::Rgb(255, 121, 198)),
        ),
    ]);
    let current = Paragraph::new(vec![Line::raw(""), status])
        .alignment(Alignment::Center)
        .block(chrome_block(" Current Vector ", Color::Rgb(139, 233, 253)));
    frame.render_widget(current, chunks[0]);

    let gauge = Gauge::default()
        .block(chrome_block(" Stage Progress ", Color::Rgb(80, 250, 123)))
        .gauge_style(
            Style::default()
                .fg(Color::Rgb(80, 250, 123))
                .bg(Color::Rgb(40, 42, 54))
                .add_modifier(Modifier::BOLD),
        )
        .ratio(progress)
        .label(format!(
            "{}/{} stages complete",
            overview.done_stage_count, overview.total_stage_count
        ));
    frame.render_widget(gauge, chunks[1]);
}

fn render_next_action(frame: &mut Frame<'_>, area: Rect, overview: &TuiOverview) {
    let paragraph = Paragraph::new(overview.next_action.as_str())
        .style(Style::default().fg(Color::Rgb(248, 248, 242)))
        .wrap(Wrap { trim: true })
        .block(chrome_block(" Next Action ", Color::Rgb(80, 250, 123)));
    frame.render_widget(paragraph, area);
}

fn render_list(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    values: &[String],
    accent: Color,
    empty_message: &str,
) {
    let items: Vec<ListItem<'_>> = if values.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            empty_message,
            Style::default().fg(Color::Rgb(98, 114, 164)),
        )))]
    } else {
        values
            .iter()
            .map(|value| {
                ListItem::new(Line::from(vec![
                    Span::styled(">> ", Style::default().fg(accent)),
                    Span::styled(
                        value.as_str(),
                        Style::default().fg(Color::Rgb(248, 248, 242)),
                    ),
                ]))
            })
            .collect()
    };
    let list = List::new(items).block(chrome_block(title, accent));
    frame.render_widget(list, area);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("q / esc", Style::default().fg(Color::Rgb(255, 184, 108))),
        Span::raw(" quit    "),
        Span::styled("readonly", Style::default().fg(Color::Rgb(139, 233, 253))),
        Span::raw(" view    "),
        Span::styled("state changes go through daedalus CLI", muted()),
    ]))
    .alignment(Alignment::Center)
    .block(chrome_block(" Controls ", Color::Rgb(98, 114, 164)));
    frame.render_widget(footer, area);
}

fn chrome_block<'a>(title: &'a str, accent: Color) -> Block<'a> {
    Block::default()
        .title(Span::styled(
            title,
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(accent))
        .style(Style::default().bg(Color::Rgb(13, 17, 30)))
        .padding(Padding::new(1, 1, 0, 0))
}

fn muted() -> Style {
    Style::default().fg(Color::Rgb(98, 114, 164))
}

fn status_color(status: &str) -> Color {
    match status {
        "active" => Color::Rgb(80, 250, 123),
        "done" | "completed" => Color::Rgb(139, 233, 253),
        "blocked" | "abandoned" => Color::Rgb(255, 85, 85),
        "paused" => Color::Rgb(255, 184, 108),
        _ => Color::Rgb(248, 248, 242),
    }
}
