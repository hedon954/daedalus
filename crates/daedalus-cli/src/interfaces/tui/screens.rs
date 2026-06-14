use markdown_tui::widget::{MarkdownStyles, MarkdownWidget};
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::{Color, Frame, Line, Modifier, Rect, Span, Style};
use ratatui::widgets::{
    Block, BorderType, Borders, Gauge, List, ListItem, ListState, Padding, Paragraph, Wrap,
};

use crate::interfaces::tui::app::{OverviewFocus, TuiDetail, TuiOverview, TuiTaskSummary};

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
    focus: OverviewFocus,
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
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(root[2]);
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(body[0]);
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(body[1]);

    render_next_action(frame, left[0], overview, focus == OverviewFocus::NextAction);
    render_evidence(frame, left[1], overview, focus == OverviewFocus::Evidence);

    render_reading_map(
        frame,
        right[0],
        overview,
        focus == OverviewFocus::ReadingMap,
    );
    render_list(
        frame,
        right[1],
        ListPanel {
            title: " Risks / Missing ",
            values: &overview.missing_artifacts,
            accent: bucket_missing_accent(&overview.workspace_bucket),
            empty_message: bucket_missing_empty(&overview.workspace_bucket),
            focused: focus == OverviewFocus::MissingArtifacts,
            max_items: 4,
        },
    );

    let review_knowledge = review_knowledge_preview(overview);
    let transitions_title = bucket_transition_title(&overview.workspace_bucket);
    let transitions = if overview.closure_summary.is_empty() {
        overview.recent_transitions.clone()
    } else {
        let mut values = overview.closure_summary.clone();
        values.extend(overview.recent_transitions.clone());
        values
    };
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(right[2]);
    render_list(
        frame,
        bottom[0],
        ListPanel {
            title: " Review / Knowledge ",
            values: &review_knowledge,
            accent: bucket_todo_accent(&overview.workspace_bucket),
            empty_message: "No review or knowledge focus yet.",
            focused: focus == OverviewFocus::ReviewKnowledge,
            max_items: 4,
        },
    );
    render_list(
        frame,
        bottom[1],
        ListPanel {
            title: transitions_title,
            values: &transitions,
            accent: TERTIARY,
            empty_message: "No transition history yet.",
            focused: focus == OverviewFocus::RecentTransitions,
            max_items: 4,
        },
    );

    render_footer(frame, root[3], can_return_to_selector);
}

/// 绘制只读详情页，支持完整内容滚动阅读。
pub fn draw_detail(
    frame: &mut Frame<'_>,
    area: Rect,
    detail: &TuiDetail,
    scroll: usize,
    can_return_to_selector: bool,
) {
    frame.render_widget(Block::default().style(Style::default().bg(BG)), area);
    let markdown = detail.markdown();
    let scroll = scroll.min(u16::MAX as usize);

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .margin(1)
        .split(area);

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            detail.title.as_str(),
            Style::default()
                .fg(NEXT_ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default().fg(TEXT)),
        Span::styled(detail.source_label.as_str(), muted()),
        Span::styled(
            format!(
                "  line {}/{}",
                scroll.saturating_add(1),
                detail.lines.len().max(1)
            ),
            muted(),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(chrome_block(" Read View ", NEXT_ACCENT));
    frame.render_widget(header, root[0]);

    let block = chrome_block(" Full Content ", CURRENT_ACCENT);
    let inner = block.inner(root[1]);
    frame.render_widget(block, root[1]);
    let markdown_widget = MarkdownWidget::new(&markdown)
        .scroll(scroll as u16)
        .styles(markdown_styles());
    frame.render_widget(markdown_widget, inner);

    render_detail_footer(frame, root[2], can_return_to_selector);
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

fn render_next_action(frame: &mut Frame<'_>, area: Rect, overview: &TuiOverview, focused: bool) {
    let paragraph = Paragraph::new(next_action_lines(overview))
        .wrap(Wrap { trim: true })
        .block(chrome_block_focused(
            " Next Action Card ",
            bucket_next_accent(&overview.workspace_bucket),
            focused,
        ));
    frame.render_widget(paragraph, area);
}

fn render_evidence(frame: &mut Frame<'_>, area: Rect, overview: &TuiOverview, focused: bool) {
    let evidence = overview.evidence_lines();
    let lines = preview_values(&evidence, 7, "No evidence available.");
    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .block(chrome_block_focused(
            " Evidence / Drift ",
            CURRENT_ACCENT,
            focused,
        ));
    frame.render_widget(paragraph, area);
}

fn render_reading_map(frame: &mut Frame<'_>, area: Rect, overview: &TuiOverview, focused: bool) {
    let reading_map = overview.reading_map_lines();
    let lines = preview_values(&reading_map, 6, "No reading entries available.");
    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .block(chrome_block_focused(
            " Reading Entrypoints ",
            NEXT_ACCENT,
            focused,
        ));
    frame.render_widget(paragraph, area);
}

fn next_action_lines(overview: &TuiOverview) -> Vec<Line<'_>> {
    let label_style = Style::default()
        .fg(bucket_next_accent(&overview.workspace_bucket))
        .add_modifier(Modifier::BOLD);

    overview
        .next_action
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            if let Some((label, value)) = line.split_once(':') {
                let value_style = if label == "Guide" {
                    Style::default().fg(MUTED)
                } else {
                    Style::default().fg(TEXT)
                };
                Line::from(vec![
                    Span::styled(format!("{}: ", label.trim()), label_style),
                    Span::styled(value.trim().to_owned(), value_style),
                ])
            } else {
                Line::from(Span::styled(line.to_owned(), Style::default().fg(TEXT)))
            }
        })
        .collect()
}

struct ListPanel<'a> {
    title: &'a str,
    values: &'a [String],
    accent: Color,
    empty_message: &'a str,
    focused: bool,
    max_items: usize,
}

fn render_list(frame: &mut Frame<'_>, area: Rect, panel: ListPanel<'_>) {
    let lines = preview_values(panel.values, panel.max_items, panel.empty_message);
    let list = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(chrome_block_focused(
            panel.title,
            panel.accent,
            panel.focused,
        ));
    frame.render_widget(list, area);
}

fn preview_values<'a>(
    values: &'a [String],
    max_items: usize,
    empty_message: &'a str,
) -> Vec<Line<'a>> {
    if values.is_empty() {
        return vec![Line::from(Span::styled(
            empty_message,
            Style::default().fg(MUTED),
        ))];
    }

    let mut lines: Vec<Line<'a>> = values
        .iter()
        .take(max_items)
        .map(|value| {
            Line::from(vec![
                Span::styled(">> ", Style::default().fg(NEXT_ACCENT)),
                Span::styled(value.as_str(), Style::default().fg(TEXT)),
            ])
        })
        .collect();
    if values.len() > max_items {
        lines.push(Line::from(Span::styled(
            format!(
                "+{} more, press enter for full view",
                values.len() - max_items
            ),
            muted(),
        )));
    }
    lines
}

fn review_knowledge_preview(overview: &TuiOverview) -> Vec<String> {
    let mut values = Vec::new();
    values.extend(
        overview
            .review_summary
            .iter()
            .map(|value| format!("review: {value}")),
    );
    values.extend(
        overview
            .knowledge_summary
            .iter()
            .map(|value| format!("knowledge: {value}")),
    );
    values.extend(
        overview
            .reflection_summary
            .iter()
            .map(|value| format!("reflection: {value}")),
    );
    values
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
        Span::styled("j/k", Style::default().fg(PAUSED)),
        Span::raw(" focus    "),
        Span::styled("enter", Style::default().fg(LEARNING)),
        Span::raw(" read    "),
        Span::styled("g/t/o", Style::default().fg(NEXT_ACCENT)),
        Span::raw(" guide/todo/outcome    "),
        Span::styled("r", Style::default().fg(TODO_ACCENT)),
        Span::raw(" refresh    "),
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

fn render_detail_footer(frame: &mut Frame<'_>, area: Rect, can_return_to_selector: bool) {
    let mut spans = vec![
        Span::styled("j/k u/d", Style::default().fg(PAUSED)),
        Span::raw(" scroll    "),
        Span::styled("pgup/pgdn", Style::default().fg(NEXT_ACCENT)),
        Span::raw(" page    "),
        Span::styled("home/end", Style::default().fg(TODO_ACCENT)),
        Span::raw(" jump    "),
        Span::styled("b / backspace", Style::default().fg(LEARNING)),
        Span::raw(" back    "),
        Span::styled("q / esc", Style::default().fg(PAUSED)),
        Span::raw(" quit    "),
    ];
    if can_return_to_selector {
        spans.push(Span::styled("selector available", muted()));
    } else {
        spans.push(Span::styled("readonly full view", muted()));
    }
    let footer = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Center)
        .block(chrome_block(" Controls ", CONTROLS_ACCENT));
    frame.render_widget(footer, area);
}

fn chrome_block<'a>(title: &'a str, accent: Color) -> Block<'a> {
    chrome_block_focused(title, accent, false)
}

fn chrome_block_focused<'a>(title: &'a str, accent: Color, focused: bool) -> Block<'a> {
    let title = if focused {
        format!(">>{}", title)
    } else {
        title.to_owned()
    };
    let border_style = if focused {
        Style::default()
            .fg(TEXT_STRONG)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(accent)
    };
    Block::default()
        .title(Span::styled(
            title,
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .style(Style::default().bg(PANEL_BG))
        .padding(Padding::new(1, 1, 0, 0))
}

fn muted() -> Style {
    Style::default().fg(MUTED)
}

fn markdown_styles() -> MarkdownStyles {
    MarkdownStyles {
        heading: [
            Style::default()
                .fg(NEXT_ACCENT)
                .add_modifier(Modifier::BOLD),
            Style::default()
                .fg(CURRENT_ACCENT)
                .add_modifier(Modifier::BOLD),
            Style::default()
                .fg(TODO_ACCENT)
                .add_modifier(Modifier::BOLD),
            Style::default().fg(TERTIARY).add_modifier(Modifier::BOLD),
            Style::default()
                .fg(PROGRESS_ACCENT)
                .add_modifier(Modifier::BOLD),
            Style::default().fg(SECONDARY).add_modifier(Modifier::BOLD),
        ],
        bold: Style::default()
            .fg(TEXT_STRONG)
            .add_modifier(Modifier::BOLD),
        italic: Style::default().fg(TEXT).add_modifier(Modifier::ITALIC),
        bold_italic: Style::default()
            .fg(TEXT_STRONG)
            .add_modifier(Modifier::BOLD | Modifier::ITALIC),
        inline_code: Style::default().fg(PROGRESS_ACCENT),
        code_block: Style::default().fg(SECONDARY).bg(PANEL_BG_SOFT),
        block_quote: Style::default().fg(MUTED).add_modifier(Modifier::ITALIC),
        rule: Style::default().fg(MUTED),
        text: Style::default().fg(TEXT),
    }
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
    for lifecycle in ["active", "idle", "abandoned"] {
        let count = tasks
            .iter()
            .filter(|task| task.lifecycle == lifecycle)
            .count();
        spans.push(Span::styled(
            format!(" {lifecycle} "),
            Style::default()
                .fg(status_color(lifecycle))
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(format!("{count}  "), muted()));
    }
    Line::from(spans)
}

fn bucket_label(bucket: &str) -> &'static str {
    match bucket {
        "projects" => "learning projects",
        "02-learning" => "learning cockpit",
        "03-completed" => "archive vault",
        "04-abandoned" => "recovery bay",
        _ => "learning cockpit",
    }
}

fn bucket_header_title(bucket: &str) -> &'static str {
    match bucket {
        "projects" => " System Online ",
        "02-learning" => " System Online ",
        "03-completed" => " Knowledge Archived ",
        "04-abandoned" => " Recovery Review ",
        _ => " System Online ",
    }
}

fn bucket_current_title(bucket: &str) -> &'static str {
    match bucket {
        "projects" => " Current Project ",
        "02-learning" => " Current Vector ",
        "03-completed" => " Closure Snapshot ",
        "04-abandoned" => " Stop Point ",
        _ => " Current Vector ",
    }
}

fn bucket_progress_title(bucket: &str) -> &'static str {
    match bucket {
        "projects" => " Topic Progress ",
        "02-learning" => " Stage Progress ",
        "03-completed" => " Completion Trace ",
        "04-abandoned" => " Progress Before Stop ",
        _ => " Stage Progress ",
    }
}

fn bucket_missing_empty(bucket: &str) -> &'static str {
    match bucket {
        "projects" => "All required artifacts are present.",
        "02-learning" => "All required artifacts are present.",
        "03-completed" => "Archive is structurally complete.",
        "04-abandoned" => "No obvious recovery gaps.",
        _ => "All required artifacts are present.",
    }
}

fn bucket_transition_title(bucket: &str) -> &'static str {
    match bucket {
        "projects" => " Recent Transitions ",
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
        "projects" => LEARNING,
        "02-learning" => LEARNING,
        "03-completed" => COMPLETED,
        "04-abandoned" => ABANDONED,
        _ => TERTIARY,
    }
}
