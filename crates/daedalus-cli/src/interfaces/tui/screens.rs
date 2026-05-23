use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::{Color, Frame, Line, Modifier, Rect, Span, Style};
use ratatui::widgets::{
    Block, BorderType, Borders, Gauge, List, ListItem, ListState, Padding, Paragraph, Wrap,
};

use crate::interfaces::tui::app::{TuiOverview, TuiTaskSummary};

const BG: Color = Color::Rgb(24, 22, 20);
const PANEL_BG: Color = Color::Rgb(31, 29, 27);
const PANEL_BG_SOFT: Color = Color::Rgb(54, 48, 42);
const TEXT: Color = Color::Rgb(220, 216, 207);
const TEXT_STRONG: Color = Color::Rgb(244, 241, 234);
const MUTED: Color = Color::Rgb(136, 128, 117);
const LEARNING: Color = Color::Rgb(217, 119, 87);
const COMPLETED: Color = Color::Rgb(147, 125, 194);
const ABANDONED: Color = Color::Rgb(203, 100, 100);
const PAUSED: Color = Color::Rgb(203, 155, 92);
const SECONDARY: Color = Color::Rgb(180, 171, 160);
const TERTIARY: Color = Color::Rgb(170, 143, 214);
const CURRENT_ACCENT: Color = Color::Rgb(124, 159, 191);
const PROGRESS_ACCENT: Color = Color::Rgb(119, 172, 125);
const MISSING_ACCENT: Color = Color::Rgb(192, 101, 92);
const TODO_ACCENT: Color = Color::Rgb(97, 166, 154);
const NEXT_ACCENT: Color = Color::Rgb(203, 155, 92);
const CONTROLS_ACCENT: Color = Color::Rgb(139, 132, 122);

/// 绘制 workspace 任务选择页。
pub fn draw_selector(frame: &mut Frame<'_>, area: Rect, tasks: &[TuiTaskSummary], selected: usize) {
    frame.render_widget(Block::default().style(Style::default().bg(BG)), area);

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .margin(1)
        .split(area);

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "DAEDALUS",
            Style::default().fg(LEARNING).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  workspace radar"),
    ]))
    .alignment(Alignment::Center)
    .block(chrome_block(" Select Learning Task ", LEARNING));
    frame.render_widget(header, root[0]);

    let counts = bucket_count_line(tasks);
    let summary = Paragraph::new(counts)
        .alignment(Alignment::Center)
        .block(chrome_block(" 02 / 03 / 04 Scan ", COMPLETED));
    frame.render_widget(summary, root[1]);

    render_task_list(frame, root[2], tasks, selected);
    render_selector_footer(frame, root[3]);
}

/// 绘制 TUI 只读总览页面。
pub fn draw_overview(
    frame: &mut Frame<'_>,
    area: Rect,
    overview: &TuiOverview,
    can_return_to_selector: bool,
) {
    frame.render_widget(Block::default().style(Style::default().bg(BG)), area);

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
        .constraints([
            Constraint::Percentage(32),
            Constraint::Percentage(28),
            Constraint::Percentage(40),
        ])
        .split(body[1]);

    render_list(
        frame,
        left[0],
        bucket_missing_title(&overview.workspace_bucket),
        &overview.missing_artifacts,
        bucket_missing_accent(&overview.workspace_bucket),
        bucket_missing_empty(&overview.workspace_bucket),
    );
    render_next_action(frame, left[1], overview);
    render_list(
        frame,
        right[0],
        " Review Focus ",
        &overview.review_summary,
        TERTIARY,
        "No review plans yet.",
    );
    let knowledge_values = if overview.knowledge_summary.is_empty() {
        overview.todo_summary.clone()
    } else {
        overview.knowledge_summary.clone()
    };
    render_list(
        frame,
        right[1],
        " Knowledge Focus ",
        &knowledge_values,
        bucket_todo_accent(&overview.workspace_bucket),
        bucket_todo_empty(&overview.workspace_bucket),
    );
    let transitions_title = bucket_transition_title(&overview.workspace_bucket);
    let transitions = if overview.closure_summary.is_empty() {
        overview.recent_transitions.clone()
    } else {
        let mut values = overview.closure_summary.clone();
        values.extend(overview.recent_transitions.clone());
        values
    };
    render_list(
        frame,
        right[2],
        transitions_title,
        &transitions,
        TERTIARY,
        "No transition history yet.",
    );

    render_footer(frame, root[3], can_return_to_selector);
}

fn render_header(frame: &mut Frame<'_>, area: Rect, overview: &TuiOverview) {
    let title = Line::from(vec![
        Span::styled(
            "DAEDALUS",
            Style::default()
                .fg(bucket_accent(&overview.workspace_bucket))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {}  ", bucket_label(&overview.workspace_bucket)),
            Style::default().fg(TEXT),
        ),
        Span::styled(
            format!("[{}]", overview.lifecycle),
            Style::default()
                .fg(status_color(&overview.lifecycle))
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let header = Paragraph::new(title)
        .alignment(Alignment::Center)
        .block(chrome_block(
            bucket_header_title(&overview.workspace_bucket),
            bucket_accent(&overview.workspace_bucket),
        ));
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
                .fg(TEXT_STRONG)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("    "),
        Span::styled("Topic  ", muted()),
        Span::styled(
            overview.active_topic.as_str(),
            Style::default().fg(TODO_ACCENT),
        ),
        Span::raw("    "),
        Span::styled("Path  ", muted()),
        Span::styled(
            overview.task_dir.display().to_string(),
            Style::default().fg(MUTED),
        ),
        Span::raw("    "),
        Span::styled("Phase  ", muted()),
        Span::styled(
            overview.current_phase.as_str(),
            Style::default().fg(CURRENT_ACCENT),
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
            Style::default().fg(SECONDARY),
        ),
    ]);
    let current = Paragraph::new(vec![Line::raw(""), status])
        .alignment(Alignment::Center)
        .block(chrome_block(
            bucket_current_title(&overview.workspace_bucket),
            CURRENT_ACCENT,
        ));
    frame.render_widget(current, chunks[0]);

    let gauge = Gauge::default()
        .block(chrome_block(
            bucket_progress_title(&overview.workspace_bucket),
            bucket_progress_accent(&overview.workspace_bucket),
        ))
        .gauge_style(
            Style::default()
                .fg(bucket_progress_accent(&overview.workspace_bucket))
                .bg(PANEL_BG_SOFT)
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
        .style(Style::default().fg(TEXT))
        .wrap(Wrap { trim: true })
        .block(chrome_block(
            bucket_next_title(&overview.workspace_bucket),
            bucket_next_accent(&overview.workspace_bucket),
        ));
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
    let lines: Vec<Line<'_>> = if values.is_empty() {
        vec![Line::from(Span::styled(
            empty_message,
            Style::default().fg(MUTED),
        ))]
    } else {
        values
            .iter()
            .map(|value| {
                Line::from(vec![
                    Span::styled(">> ", Style::default().fg(accent)),
                    Span::styled(value.as_str(), Style::default().fg(TEXT)),
                ])
            })
            .collect()
    };
    let list = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(chrome_block(title, accent));
    frame.render_widget(list, area);
}

fn render_task_list(frame: &mut Frame<'_>, area: Rect, tasks: &[TuiTaskSummary], selected: usize) {
    let items: Vec<ListItem<'_>> = tasks
        .iter()
        .map(|task| {
            let progress = if task.total_stage_count == 0 {
                "0/0".to_owned()
            } else {
                format!("{}/{}", task.done_stage_count, task.total_stage_count)
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:<13}", task.bucket),
                    Style::default()
                        .fg(bucket_accent(&task.bucket))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("{:<24}", task.task_name), Style::default().fg(TEXT)),
                Span::styled(
                    format!("{:<18}", task.active_topic),
                    Style::default().fg(TODO_ACCENT),
                ),
                Span::styled(format!("{:<11}", task.lifecycle), muted()),
                Span::styled(
                    format!("{:<18}", task.current_phase),
                    Style::default().fg(CURRENT_ACCENT),
                ),
                Span::styled(
                    format!("{:<8}", task.current_status),
                    Style::default().fg(status_color(&task.current_status)),
                ),
                Span::styled(progress, Style::default().fg(PROGRESS_ACCENT)),
            ]))
        })
        .collect();
    let list = List::new(items)
        .block(chrome_block(" Learning Tasks ", TERTIARY))
        .highlight_symbol(">> ")
        .highlight_style(
            Style::default()
                .fg(TEXT_STRONG)
                .bg(Color::Rgb(83, 63, 55))
                .add_modifier(Modifier::BOLD),
        );
    let mut state = ListState::default();
    if !tasks.is_empty() {
        state.select(Some(selected.min(tasks.len() - 1)));
    }
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_selector_footer(frame: &mut Frame<'_>, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("↑/↓ j/k", Style::default().fg(PAUSED)),
        Span::raw(" select    "),
        Span::styled("enter", Style::default().fg(LEARNING)),
        Span::raw(" open    "),
        Span::styled("q / esc", Style::default().fg(PAUSED)),
        Span::raw(" quit"),
    ]))
    .alignment(Alignment::Center)
    .block(chrome_block(" Controls ", CONTROLS_ACCENT));
    frame.render_widget(footer, area);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, can_return_to_selector: bool) {
    let mut spans = vec![
        Span::styled("q / esc", Style::default().fg(PAUSED)),
        Span::raw(" quit    "),
    ];
    if can_return_to_selector {
        spans.extend([
            Span::styled("b / backspace", Style::default().fg(LEARNING)),
            Span::raw(" back    "),
        ]);
    }
    spans.extend([
        Span::styled("readonly", Style::default().fg(COMPLETED)),
        Span::raw(" view    "),
        Span::styled("state changes go through daedalus CLI", muted()),
    ]);
    let footer = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Center)
        .block(chrome_block(" Controls ", CONTROLS_ACCENT));
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
        .style(Style::default().bg(PANEL_BG))
        .padding(Padding::new(1, 1, 0, 0))
}

fn muted() -> Style {
    Style::default().fg(MUTED)
}

fn status_color(status: &str) -> Color {
    match status {
        "active" => PROGRESS_ACCENT,
        "done" | "completed" => COMPLETED,
        "blocked" | "abandoned" => ABANDONED,
        "paused" => PAUSED,
        _ => TEXT,
    }
}

fn bucket_count_line(tasks: &[TuiTaskSummary]) -> Line<'_> {
    let mut spans = Vec::new();
    for bucket in ["02-learning", "03-completed", "04-abandoned"] {
        let count = tasks.iter().filter(|task| task.bucket == bucket).count();
        spans.push(Span::styled(
            format!(" {bucket} "),
            Style::default()
                .fg(bucket_accent(bucket))
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(format!("{count}  "), muted()));
    }
    Line::from(spans)
}

fn bucket_label(bucket: &str) -> &'static str {
    match bucket {
        "02-learning" => "learning cockpit",
        "03-completed" => "archive vault",
        "04-abandoned" => "recovery bay",
        _ => "learning cockpit",
    }
}

fn bucket_header_title(bucket: &str) -> &'static str {
    match bucket {
        "02-learning" => " System Online ",
        "03-completed" => " Knowledge Archived ",
        "04-abandoned" => " Recovery Review ",
        _ => " System Online ",
    }
}

fn bucket_current_title(bucket: &str) -> &'static str {
    match bucket {
        "02-learning" => " Current Vector ",
        "03-completed" => " Closure Snapshot ",
        "04-abandoned" => " Stop Point ",
        _ => " Current Vector ",
    }
}

fn bucket_progress_title(bucket: &str) -> &'static str {
    match bucket {
        "02-learning" => " Stage Progress ",
        "03-completed" => " Completion Trace ",
        "04-abandoned" => " Progress Before Stop ",
        _ => " Stage Progress ",
    }
}

fn bucket_missing_title(bucket: &str) -> &'static str {
    match bucket {
        "02-learning" => " Missing Artifacts ",
        "03-completed" => " Archive Audit ",
        "04-abandoned" => " Recovery Gaps ",
        _ => " Missing Artifacts ",
    }
}

fn bucket_missing_empty(bucket: &str) -> &'static str {
    match bucket {
        "02-learning" => "All required artifacts are present.",
        "03-completed" => "Archive is structurally complete.",
        "04-abandoned" => "No obvious recovery gaps.",
        _ => "All required artifacts are present.",
    }
}

fn bucket_next_title(bucket: &str) -> &'static str {
    match bucket {
        "02-learning" => " Next Action ",
        "03-completed" => " Reuse / Knowledge Export ",
        "04-abandoned" => " Revival Decision ",
        _ => " Next Action ",
    }
}

fn bucket_todo_empty(bucket: &str) -> &'static str {
    match bucket {
        "02-learning" => "No open todo items.",
        "03-completed" => "No follow-up ideas.",
        "04-abandoned" => "No reactivation todos.",
        _ => "No open todo items.",
    }
}

fn bucket_transition_title(bucket: &str) -> &'static str {
    match bucket {
        "02-learning" => " Recent Transitions ",
        "03-completed" => " Closure Trail ",
        "04-abandoned" => " Abandon Trail ",
        _ => " Recent Transitions ",
    }
}

fn bucket_missing_accent(bucket: &str) -> Color {
    match bucket {
        "03-completed" => COMPLETED,
        "04-abandoned" => ABANDONED,
        _ => MISSING_ACCENT,
    }
}

fn bucket_next_accent(bucket: &str) -> Color {
    match bucket {
        "03-completed" => COMPLETED,
        "04-abandoned" => PAUSED,
        _ => NEXT_ACCENT,
    }
}

fn bucket_progress_accent(bucket: &str) -> Color {
    match bucket {
        "03-completed" => COMPLETED,
        "04-abandoned" => PAUSED,
        _ => PROGRESS_ACCENT,
    }
}

fn bucket_todo_accent(bucket: &str) -> Color {
    match bucket {
        "03-completed" => TERTIARY,
        "04-abandoned" => PAUSED,
        _ => TODO_ACCENT,
    }
}

fn bucket_accent(bucket: &str) -> Color {
    match bucket {
        "02-learning" => LEARNING,
        "03-completed" => COMPLETED,
        "04-abandoned" => ABANDONED,
        _ => TERTIARY,
    }
}
