use std::borrow::Cow;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap},
};

use crate::cli::app::{CliState, UiLine, UiMode};

const BG: Color = Color::Rgb(12, 13, 14);
const SURFACE: Color = Color::Rgb(18, 19, 21);
const SURFACE_HOT: Color = Color::Rgb(24, 25, 28);
const LINE: Color = Color::Rgb(60, 64, 70);
const MUTED: Color = Color::Rgb(126, 132, 142);
const SUBTLE: Color = Color::Rgb(166, 171, 181);
const TEXT: Color = Color::Rgb(226, 229, 234);
const BLUE: Color = Color::Rgb(96, 165, 250);
const GREEN: Color = Color::Rgb(74, 222, 128);
const YELLOW: Color = Color::Rgb(251, 191, 36);
const ORANGE: Color = Color::Rgb(251, 146, 60);
const RED: Color = Color::Rgb(248, 113, 113);
const MAGENTA: Color = Color::Rgb(192, 132, 252);
const CYAN: Color = Color::Rgb(45, 212, 191);

pub fn draw(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    let root = Block::default().style(Style::default().bg(BG));
    frame.render_widget(root, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(input_height(state)),
            Constraint::Length(2),
        ])
        .split(area);

    draw_header(frame, chunks[0], state);
    draw_transcript(frame, chunks[1], state);
    draw_prompt_or_approval(frame, chunks[2], state);
    draw_footer(frame, chunks[3], state);
}

fn input_height(state: &CliState) -> u16 {
    match state.mode {
        UiMode::PendingApproval => 7,
        UiMode::RunningAgent => 3,
        UiMode::EditingPrompt => 4,
    }
}

fn draw_header(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    let status_color = match state.mode {
        UiMode::EditingPrompt => GREEN,
        UiMode::RunningAgent => BLUE,
        UiMode::PendingApproval => YELLOW,
    };

    let prompt = state
        .active_prompt
        .as_deref()
        .map(|prompt| truncate(prompt, area.width.saturating_sub(28).max(18) as usize))
        .unwrap_or_else(|| state.input_hint().to_string());

    let lines = vec![
        Line::from(vec![
            Span::styled(
                "Codex Mini",
                Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  local agent", Style::default().fg(MUTED)),
            Span::raw("  "),
            pill(state.status_text(), status_color),
            Span::styled("  workspace tools", Style::default().fg(MUTED)),
        ]),
        Line::from(vec![
            Span::styled("focus ", Style::default().fg(MUTED)),
            Span::styled(prompt, Style::default().fg(SUBTLE)),
        ]),
    ];

    let widget = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(LINE))
                .padding(Padding::horizontal(1)),
        )
        .style(Style::default().fg(TEXT).bg(BG));

    frame.render_widget(widget, area);
}

fn draw_transcript(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    let line_count = state.lines.len();
    let lines = transcript_lines(state);
    let scroll = transcript_scroll_offset(state, &lines, area);
    let title = Line::from(vec![
        Span::styled(
            " transcript",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("  {line_count} events"), Style::default().fg(MUTED)),
        Span::styled(
            format!("  row {}", scroll.saturating_add(1)),
            Style::default().fg(MUTED),
        ),
    ]);

    let widget = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .title(title)
                .borders(Borders::LEFT)
                .border_style(Style::default().fg(CYAN))
                .padding(Padding::horizontal(1)),
        )
        .style(Style::default().fg(TEXT).bg(BG))
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    frame.render_widget(widget, area);
}

fn transcript_lines(state: &CliState) -> Vec<Line<'static>> {
    if state.lines.is_empty() {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Ready.",
                    Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " Ask a question, calculate with tools, or request a safe local command.",
                    Style::default().fg(MUTED),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Try  ", Style::default().fg(MUTED)),
                Span::styled(
                    "请调用 run_command 执行 pwd，然后解释结果。",
                    Style::default().fg(SUBTLE),
                ),
            ]),
        ]
    } else {
        state.lines.iter().flat_map(render_line).collect()
    }
}

fn transcript_scroll_offset(state: &CliState, lines: &[Line<'_>], area: Rect) -> u16 {
    let max_scroll = max_visual_scroll(lines, area);
    if follows_tail(state) {
        max_scroll
    } else {
        state.log_scroll.min(max_scroll)
    }
}

fn follows_tail(state: &CliState) -> bool {
    let last_logical_line = state.lines.len().saturating_sub(1) as u16;
    state.log_scroll == last_logical_line
}

fn max_visual_scroll(lines: &[Line<'_>], area: Rect) -> u16 {
    let content_width = area.width.saturating_sub(3).max(1) as usize;
    let content_height = area.height.max(1) as usize;
    let visual_rows = lines
        .iter()
        .map(|line| wrapped_row_count(line, content_width))
        .sum::<usize>();
    visual_rows.saturating_sub(content_height) as u16
}

fn wrapped_row_count(line: &Line<'_>, content_width: usize) -> usize {
    let width = visual_width(line);
    width.max(1).div_ceil(content_width)
}

fn visual_width(line: &Line<'_>) -> usize {
    line.spans
        .iter()
        .map(|span| terminal_width(span.content.as_ref()))
        .sum()
}

fn terminal_width(text: &str) -> usize {
    text.chars()
        .map(|ch| if ch.is_ascii() { 1 } else { 2 })
        .sum()
}

fn draw_prompt_or_approval(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    match state.mode {
        UiMode::PendingApproval => draw_approval(frame, area, state),
        _ => draw_prompt(frame, area, state),
    }
}

fn draw_prompt(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    let text = match state.mode {
        UiMode::RunningAgent => Text::from(Line::from(vec![
            Span::styled(
                "  running",
                Style::default().fg(BLUE).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  streaming model and tool events...",
                Style::default().fg(MUTED),
            ),
        ])),
        _ => Text::from(vec![Line::from(vec![
            Span::styled(
                "> ",
                Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
            ),
            Span::styled(state.input.as_str(), Style::default().fg(TEXT)),
            Span::styled(
                " ",
                Style::default().fg(TEXT).add_modifier(Modifier::REVERSED),
            ),
        ])]),
    };

    let widget = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(match state.mode {
                    UiMode::RunningAgent => BLUE,
                    _ => GREEN,
                }))
                .padding(Padding::horizontal(1)),
        )
        .style(Style::default().fg(TEXT).bg(SURFACE))
        .wrap(Wrap { trim: false });

    frame.render_widget(widget, area);
}

fn draw_approval(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    let text = match &state.pending_approval {
        Some(pending) => Text::from(vec![
            Line::from(vec![
                Span::styled(
                    "  Approval required",
                    Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " before command execution continues",
                    Style::default().fg(MUTED),
                ),
            ]),
            Line::from(vec![
                Span::styled("  reason  ", Style::default().fg(MUTED)),
                Span::styled(pending.reason.as_str(), Style::default().fg(TEXT)),
            ]),
            Line::from(vec![
                Span::styled("  scope   ", Style::default().fg(MUTED)),
                Span::styled(pending.scope_summary.as_str(), Style::default().fg(TEXT)),
            ]),
            Line::from(""),
            Line::from(vec![
                key("a"),
                Span::raw(" approve once   "),
                key("s"),
                Span::raw(" approve session   "),
                key("r"),
                Span::raw(" reject"),
            ]),
        ]),
        None => Text::from(Line::from(vec![Span::styled(
            "Approval state is missing. Press q to quit.",
            Style::default().fg(RED),
        )])),
    };

    frame.render_widget(Clear, area);
    let widget = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(YELLOW))
                .padding(Padding::horizontal(1)),
        )
        .style(Style::default().bg(SURFACE_HOT))
        .wrap(Wrap { trim: false });

    frame.render_widget(widget, area);
}

fn draw_footer(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    let keys = match state.mode {
        UiMode::PendingApproval => vec![
            key("a"),
            Span::raw(" once  "),
            key("s"),
            Span::raw(" session  "),
            key("r"),
            Span::raw(" reject  "),
            key("q"),
            Span::raw(" quit"),
        ],
        UiMode::RunningAgent => vec![
            key("up/down"),
            Span::raw(" scroll  "),
            key("pgup/pgdn"),
            Span::raw(" page  "),
            key("home/end"),
            Span::raw(" jump  "),
            key("q"),
            Span::raw(" quit"),
        ],
        UiMode::EditingPrompt => vec![
            key("enter"),
            Span::raw(" submit  "),
            key("backspace"),
            Span::raw(" delete  "),
            key("up/down"),
            Span::raw(" scroll  "),
            key("q"),
            Span::raw(" quit"),
        ],
    };

    let widget = Paragraph::new(Line::from(keys))
        .alignment(Alignment::Center)
        .style(Style::default().fg(MUTED).bg(BG));

    frame.render_widget(widget, area);
}

fn render_line(line: &UiLine) -> Vec<Line<'static>> {
    match line {
        UiLine::System(text) => compact_event("system", BLUE, text),
        UiLine::User(text) => conversation_block("you", GREEN, text, false),
        UiLine::Thinking(text) => markdown_block("thinking", MAGENTA, text),
        UiLine::Model(text) => markdown_block("assistant", TEXT, text),
        UiLine::Tool(text) => compact_event("tool", CYAN, text),
        UiLine::Command(text) => compact_event("command", ORANGE, text),
        UiLine::Approval(text) => compact_event("approval", YELLOW, text),
        UiLine::Error(text) => compact_event("error", RED, text),
    }
}

fn conversation_block(
    label: &'static str,
    color: Color,
    body: &str,
    show_rule: bool,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if show_rule {
        lines.push(Line::from(vec![Span::styled(
            "─".repeat(12),
            Style::default().fg(LINE),
        )]));
    }
    lines.push(Line::from(vec![Span::styled(
        format!(" {label}"),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )]));
    for text_line in body.lines() {
        lines.push(Line::from(vec![
            Span::styled("   ", Style::default().fg(MUTED)),
            Span::styled(text_line.to_string(), Style::default().fg(TEXT)),
        ]));
    }
    lines.push(Line::from(""));
    lines
}

fn markdown_block(label: &'static str, color: Color, body: &str) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(vec![Span::styled(
        format!(" {label}"),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )])];

    for line in markdown_content_lines(body) {
        let mut spans = vec![Span::styled("   ", Style::default().fg(MUTED))];
        spans.extend(line.spans);
        lines.push(Line {
            style: line.style,
            alignment: line.alignment,
            spans,
        });
    }

    lines.push(Line::from(""));
    lines
}

fn markdown_content_lines(input: &str) -> Vec<Line<'static>> {
    let lines = input.lines().collect::<Vec<_>>();
    let mut output = Vec::<Line<'static>>::new();
    let mut markdown_buffer = Vec::<&str>::new();
    let mut index = 0;

    while index < lines.len() {
        if index + 1 < lines.len()
            && parse_table_row(lines[index]).is_some()
            && is_table_separator(lines[index + 1])
        {
            flush_markdown_buffer(&mut markdown_buffer, &mut output);

            let mut rows = vec![parse_table_row(lines[index]).expect("checked table header")];
            index += 2;

            while index < lines.len() {
                let Some(row) = parse_table_row(lines[index]) else {
                    break;
                };
                rows.push(row);
                index += 1;
            }

            output.extend(format_table_rows(rows).into_iter().enumerate().map(
                |(row_index, row)| {
                    let style = if row_index == 0 {
                        Style::default().fg(TEXT).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(SUBTLE)
                    };
                    Line::from(Span::styled(row, style))
                },
            ));
            continue;
        }

        markdown_buffer.push(lines[index]);
        index += 1;
    }

    flush_markdown_buffer(&mut markdown_buffer, &mut output);
    output
}

fn flush_markdown_buffer<'a>(buffer: &mut Vec<&'a str>, output: &mut Vec<Line<'static>>) {
    if buffer.is_empty() {
        return;
    }

    let markdown = buffer.join("\n");
    output.extend(
        tui_markdown::from_str(&markdown)
            .lines
            .into_iter()
            .map(owned_line),
    );
    buffer.clear();
}

fn owned_line(line: Line<'_>) -> Line<'static> {
    Line {
        style: line.style,
        alignment: line.alignment,
        spans: line.spans.into_iter().map(owned_span).collect(),
    }
}

fn owned_span(span: Span<'_>) -> Span<'static> {
    Span {
        style: span.style,
        content: Cow::Owned(span.content.into_owned()),
    }
}

fn parse_table_row(line: &str) -> Option<Vec<String>> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return None;
    }

    let cells = trimmed
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect::<Vec<_>>();

    if cells.len() < 2 || cells.iter().all(String::is_empty) {
        return None;
    }

    Some(cells)
}

fn is_table_separator(line: &str) -> bool {
    parse_table_row(line).is_some_and(|cells| {
        cells.iter().all(|cell| {
            let marker = cell.trim();
            marker.len() >= 3 && marker.chars().all(|ch| matches!(ch, '-' | ':' | ' '))
        })
    })
}

fn format_table_rows(rows: Vec<Vec<String>>) -> Vec<String> {
    let column_count = rows.iter().map(Vec::len).max().unwrap_or(0);
    let widths = (0..column_count)
        .map(|column| {
            rows.iter()
                .filter_map(|row| row.get(column))
                .map(|cell| terminal_width(cell))
                .max()
                .unwrap_or(0)
        })
        .collect::<Vec<_>>();

    rows.into_iter()
        .map(|row| {
            (0..column_count)
                .map(|column| {
                    let cell = row.get(column).map(String::as_str).unwrap_or("");
                    let padding = widths[column].saturating_sub(terminal_width(cell));
                    format!("{cell}{}", " ".repeat(padding))
                })
                .collect::<Vec<_>>()
                .join("  ")
                .trim_end()
                .to_string()
        })
        .collect()
}

fn compact_event(label: &'static str, color: Color, body: &str) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for (index, text_line) in body.lines().enumerate() {
        let marker = if index == 0 { label } else { "" };
        lines.push(Line::from(vec![
            Span::styled("   ", Style::default().fg(MUTED)),
            Span::styled(
                format!("{marker:<9}"),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(text_line.to_string(), Style::default().fg(SUBTLE)),
        ]));
    }
    lines.push(Line::from(""));
    lines
}

fn key(text: &'static str) -> Span<'static> {
    Span::styled(
        text,
        Style::default()
            .fg(BG)
            .bg(BLUE)
            .add_modifier(Modifier::BOLD),
    )
}

fn pill(text: &'static str, color: Color) -> Span<'static> {
    Span::styled(
        format!(" {text} "),
        Style::default()
            .fg(BG)
            .bg(color)
            .add_modifier(Modifier::BOLD),
    )
}

fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    let mut output = text
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect::<String>();
    output.push_str("...");
    output
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use super::{transcript_lines, transcript_scroll_offset};
    use crate::cli::app::{CliState, UiLine};

    #[test]
    fn tail_scroll_uses_visual_rows_not_logical_event_count() {
        let mut state = CliState::new();
        state.push_line(UiLine::Model(
            "this is a very long assistant response that must wrap across several terminal rows"
                .to_string(),
        ));
        state.push_line(UiLine::System("turn completed".to_string()));

        let lines = transcript_lines(&state);
        let scroll = transcript_scroll_offset(&state, &lines, Rect::new(0, 0, 24, 5));

        assert!(scroll > state.lines.len() as u16);
    }

    #[test]
    fn manual_scroll_is_clamped_to_visual_scroll_bounds() {
        let mut state = CliState::new();
        state.push_line(UiLine::Model("short".to_string()));
        state.log_scroll = 100;

        let lines = transcript_lines(&state);
        let scroll = transcript_scroll_offset(&state, &lines, Rect::new(0, 0, 80, 20));

        assert_eq!(scroll, 0);
    }

    #[test]
    fn assistant_output_renders_markdown_instead_of_raw_markers() {
        let mut state = CliState::new();
        state.push_line(UiLine::Model(
            "Result:\n\n- **final answer**: `10018`".to_string(),
        ));

        let rendered = transcript_lines(&state)
            .into_iter()
            .flat_map(|line| line.spans.into_iter())
            .map(|span| span.content.into_owned())
            .collect::<Vec<_>>()
            .join("");

        assert!(rendered.contains("final answer"));
        assert!(rendered.contains("10018"));
        assert!(!rendered.contains("**"));
        assert!(!rendered.contains('`'));
    }

    #[test]
    fn assistant_output_preserves_markdown_table_rows() {
        let mut state = CliState::new();
        state.push_line(UiLine::Model(
            "计算结果如下：\n\n| 步骤 | 表达式 | 结果 |\n|---|---|---|\n| 1 | 6000 + 2000 | 8000 |\n| 2 | 2141 - 123 | 2018 |"
                .to_string(),
        ));

        let rows = transcript_lines(&state)
            .into_iter()
            .map(|line| {
                line.spans
                    .into_iter()
                    .map(|span| span.content.into_owned())
                    .collect::<String>()
            })
            .collect::<Vec<_>>();

        assert!(
            rows.iter()
                .any(|row| row.contains("步骤") && row.contains("表达式"))
        );
        assert!(rows.iter().any(|row| row.contains("6000 + 2000")));
        assert!(rows.iter().any(|row| row.contains("2141 - 123")));
        assert!(!rows.iter().any(|row| row.contains("---")));
    }
}
