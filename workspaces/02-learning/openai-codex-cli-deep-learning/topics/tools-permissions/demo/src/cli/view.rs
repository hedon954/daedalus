use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap},
};

use crate::cli::app::{CliState, UiLine, UiMode};

const BG: Color = Color::Rgb(18, 18, 18);
const PANEL: Color = Color::Rgb(28, 28, 28);
const MUTED: Color = Color::Rgb(145, 145, 145);
const TEXT: Color = Color::Rgb(218, 218, 218);
const BLUE: Color = Color::Rgb(95, 175, 255);
const GREEN: Color = Color::Rgb(95, 215, 135);
const YELLOW: Color = Color::Rgb(255, 190, 95);
const RED: Color = Color::Rgb(255, 105, 105);
const MAGENTA: Color = Color::Rgb(190, 135, 255);
const CYAN: Color = Color::Rgb(95, 215, 215);

pub fn draw(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    let root = Block::default().style(Style::default().bg(BG));
    frame.render_widget(root, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(10),
            Constraint::Length(input_height(state)),
            Constraint::Length(3),
        ])
        .split(area);

    draw_header(frame, chunks[0], state);
    draw_transcript(frame, chunks[1], state);
    draw_prompt_or_approval(frame, chunks[2], state);
    draw_footer(frame, chunks[3], state);
}

fn input_height(state: &CliState) -> u16 {
    match state.mode {
        UiMode::PendingApproval => 8,
        _ => 5,
    }
}

fn draw_header(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    let status_color = match state.mode {
        UiMode::EditingPrompt => GREEN,
        UiMode::RunningAgent => BLUE,
        UiMode::PendingApproval => YELLOW,
    };

    let mut lines = vec![Line::from(vec![
        Span::styled(
            "Codex Mini",
            Style::default().fg(BLUE).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("agent cli", Style::default().fg(MUTED)),
        Span::raw("  "),
        Span::styled(
            format!("[{}]", state.status_text()),
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        ),
    ])];

    let prompt = state
        .active_prompt
        .as_deref()
        .map(|prompt| truncate(prompt, 88))
        .unwrap_or_else(|| state.input_hint().to_string());
    lines.push(Line::from(vec![
        Span::styled("focus ", Style::default().fg(MUTED)),
        Span::styled(prompt, Style::default().fg(TEXT)),
    ]));

    let widget = Paragraph::new(Text::from(lines))
        .block(panel("System Online").border_style(Style::default().fg(BLUE)))
        .style(Style::default().fg(TEXT).bg(PANEL));

    frame.render_widget(widget, area);
}

fn draw_transcript(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    let line_count = state.lines.len();
    let title = format!(
        "Transcript  {line_count} events  line {}",
        state.log_scroll + 1
    );
    let lines = if state.lines.is_empty() {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "Ask a question, request a calculation, or ask me to run a safe local command.",
                Style::default().fg(MUTED),
            )]),
            Line::from(vec![Span::styled(
                "Example: 请调用 run_command 执行 pwd，然后解释结果。",
                Style::default().fg(MUTED),
            )]),
        ]
    } else {
        state.lines.iter().flat_map(render_line).collect()
    };

    let widget = Paragraph::new(Text::from(lines))
        .block(panel(title).border_style(Style::default().fg(CYAN)))
        .style(Style::default().bg(PANEL))
        .wrap(Wrap { trim: false })
        .scroll((state.log_scroll, 0));

    frame.render_widget(widget, area);
}

fn draw_prompt_or_approval(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    match state.mode {
        UiMode::PendingApproval => draw_approval(frame, area, state),
        _ => draw_prompt(frame, area, state),
    }
}

fn draw_prompt(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    let text = match state.mode {
        UiMode::RunningAgent => Text::from(vec![
            Line::from(vec![Span::styled(
                "Agent is running. New prompt input is paused until this turn finishes.",
                Style::default().fg(MUTED),
            )]),
            Line::from(vec![Span::styled(
                "Use Up/Down or PageUp/PageDown to inspect the transcript.",
                Style::default().fg(MUTED),
            )]),
        ]),
        _ => Text::from(vec![Line::from(vec![
            Span::styled("> ", Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
            Span::styled(state.input.as_str(), Style::default().fg(TEXT)),
            Span::styled(
                " ",
                Style::default().fg(TEXT).add_modifier(Modifier::REVERSED),
            ),
        ])]),
    };

    let widget = Paragraph::new(text)
        .block(panel("Prompt").border_style(Style::default().fg(BLUE)))
        .style(Style::default().bg(PANEL))
        .wrap(Wrap { trim: false });

    frame.render_widget(widget, area);
}

fn draw_approval(frame: &mut Frame<'_>, area: Rect, state: &CliState) {
    let text = match &state.pending_approval {
        Some(pending) => Text::from(vec![
            Line::from(vec![
                Span::styled(
                    "Approval required",
                    Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " before command execution continues",
                    Style::default().fg(MUTED),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("reason  ", Style::default().fg(MUTED)),
                Span::styled(pending.reason.as_str(), Style::default().fg(TEXT)),
            ]),
            Line::from(vec![
                Span::styled("scope   ", Style::default().fg(MUTED)),
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

    let widget = Paragraph::new(text)
        .block(panel("Security Gate").border_style(Style::default().fg(YELLOW)))
        .style(Style::default().bg(PANEL))
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
        .block(panel("Controls").border_style(Style::default().fg(MUTED)))
        .style(Style::default().fg(TEXT).bg(PANEL));

    frame.render_widget(widget, area);
}

fn render_line(line: &UiLine) -> Vec<Line<'static>> {
    let (label, color, body) = match line {
        UiLine::System(text) => ("system", BLUE, text.as_str()),
        UiLine::User(text) => ("you", GREEN, text.as_str()),
        UiLine::Thinking(text) => ("thinking", MAGENTA, text.as_str()),
        UiLine::Model(text) => ("assistant", TEXT, text.as_str()),
        UiLine::Tool(text) => ("tool", CYAN, text.as_str()),
        UiLine::Command(text) => ("command", YELLOW, text.as_str()),
        UiLine::Approval(text) => ("approval", YELLOW, text.as_str()),
        UiLine::Error(text) => ("error", RED, text.as_str()),
    };

    vec![
        Line::from(vec![
            Span::styled(
                format!("{label:<9}"),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(body.to_string(), Style::default().fg(TEXT)),
        ]),
        Line::from(""),
    ]
}

fn panel<T>(title: T) -> Block<'static>
where
    T: Into<String>,
{
    Block::default()
        .title(title.into())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(PANEL))
}

fn key(text: &'static str) -> Span<'static> {
    Span::styled(
        text,
        Style::default()
            .fg(BLUE)
            .add_modifier(Modifier::BOLD | Modifier::REVERSED),
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
