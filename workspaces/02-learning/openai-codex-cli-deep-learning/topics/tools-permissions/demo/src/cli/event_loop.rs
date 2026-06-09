use std::{io, sync::Arc, time::Duration};

use anyhow::Context;
use futures_util::StreamExt;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEvent},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};
use tokio::sync::mpsc;

use crate::{
    agent::{react::ReActAgent, stream_event::StreamEvent},
    cli::{
        app::{CliState, PendingApprovalView, UiLine, UiMode},
        view,
    },
    model::{approval::ApprovalPersistence, event::UserApprovalDecision},
    tool::shell::approval::ToolApprovalResult,
};

enum UiEvent {
    Agent(StreamEvent),
    AgentFailed(String),
    AgentFinished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum UiCommand {
    None,
    Quit,
    SubmitPrompt(String),
    ApproveOnce(String),
    ApproveSession(String),
    Reject(String),
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> anyhow::Result<Self> {
        enable_raw_mode().context("failed to enable raw mode")?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).context("failed to enter alternate screen")?;

        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

pub fn run_cli(
    agent: Arc<ReActAgent>,
    approval_tx: mpsc::Sender<ToolApprovalResult>,
) -> anyhow::Result<()> {
    let mut state = CliState::new();
    let (ui_tx, ui_rx) = mpsc::channel(128);

    let _guard = TerminalGuard::enter()?;

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("failed to create terminal")?;

    let result = run_event_loop(&mut terminal, &mut state, agent, approval_tx, ui_tx, ui_rx);

    terminal.show_cursor().context("failed to show cursor")?;

    result
}

fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut CliState,
    agent: Arc<ReActAgent>,
    approval_tx: mpsc::Sender<ToolApprovalResult>,
    ui_tx: mpsc::Sender<UiEvent>,
    mut ui_rx: mpsc::Receiver<UiEvent>,
) -> anyhow::Result<()> {
    loop {
        while let Ok(event) = ui_rx.try_recv() {
            handle_ui_event(state, event);
        }

        terminal
            .draw(|frame| view::draw(frame, frame.area(), state))
            .context("failed to draw cli")?;

        if state.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(50)).context("failed to poll terminal event")?
            && let Event::Key(key) = event::read().context("failed to read terminal event")?
        {
            let command = handle_key(state, key);
            handle_command(
                state,
                command,
                agent.clone(),
                approval_tx.clone(),
                ui_tx.clone(),
            );
        }
    }

    Ok(())
}

fn handle_key(state: &mut CliState, key: KeyEvent) -> UiCommand {
    match key.code {
        KeyCode::Up => {
            state.scroll_log_up();
            return UiCommand::None;
        }
        KeyCode::Down => {
            state.scroll_log_down();
            return UiCommand::None;
        }
        _ => {}
    }

    match state.mode {
        super::app::UiMode::EditingPrompt => handle_prompt_key(state, key),
        super::app::UiMode::RunningAgent => handle_running_key(state, key),
        super::app::UiMode::PendingApproval => handle_approval_key(state, key),
    }
}

fn handle_prompt_key(state: &mut CliState, key: KeyEvent) -> UiCommand {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => UiCommand::Quit,
        KeyCode::Char(c) => {
            state.push_char(c);
            UiCommand::None
        }
        KeyCode::Backspace => {
            state.backspace();
            UiCommand::None
        }
        KeyCode::Enter => match state.take_prompt() {
            Some(prompt) => UiCommand::SubmitPrompt(prompt),
            None => UiCommand::None,
        },
        _ => UiCommand::None,
    }
}

fn handle_running_key(_state: &mut CliState, key: KeyEvent) -> UiCommand {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => UiCommand::Quit,
        _ => UiCommand::None,
    }
}

fn handle_approval_key(state: &mut CliState, key: KeyEvent) -> UiCommand {
    let Some(pending) = &state.pending_approval else {
        return UiCommand::None;
    };

    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => UiCommand::Quit,
        KeyCode::Char('a') => UiCommand::ApproveOnce(pending.approval_id.clone()),
        KeyCode::Char('s') => UiCommand::ApproveSession(pending.approval_id.clone()),
        KeyCode::Char('r') => UiCommand::Reject(pending.approval_id.clone()),
        _ => UiCommand::None,
    }
}

fn handle_command(
    state: &mut CliState,
    command: UiCommand,
    agent: Arc<ReActAgent>,
    approval_tx: mpsc::Sender<ToolApprovalResult>,
    ui_tx: mpsc::Sender<UiEvent>,
) {
    match command {
        UiCommand::None => {}
        UiCommand::Quit => state.should_quit = true,
        UiCommand::SubmitPrompt(prompt) => {
            state.push_line(UiLine::User(prompt.clone()));
            state.mode = UiMode::RunningAgent;

            tokio::spawn(async move {
                match agent.run(prompt) {
                    Ok(mut stream) => {
                        while let Some(event) = stream.next().await {
                            match event {
                                Ok(StreamEvent::Completed) => {
                                    let _ = ui_tx.send(UiEvent::AgentFinished).await;
                                    break;
                                }
                                Ok(event) => {
                                    let _ = ui_tx.send(UiEvent::Agent(event)).await;
                                }
                                Err(err) => {
                                    let _ = ui_tx.send(UiEvent::AgentFailed(err.to_string())).await;
                                    break;
                                }
                            }
                        }
                    }
                    Err(err) => {
                        let _ = ui_tx.send(UiEvent::AgentFailed(err.to_string())).await;
                    }
                }
            });
        }
        UiCommand::ApproveOnce(approval_id) => {
            send_approval(state, approval_tx, approval_id, ApprovalPersistence::Once);
        }
        UiCommand::ApproveSession(approval_id) => {
            send_approval(
                state,
                approval_tx,
                approval_id,
                ApprovalPersistence::Session,
            );
        }
        UiCommand::Reject(approval_id) => {
            let _ = approval_tx.try_send(ToolApprovalResult {
                approval_id,
                decision: UserApprovalDecision::Rejected,
            });
            state.pending_approval = None;
            state.mode = UiMode::RunningAgent;
        }
    }
}

fn send_approval(
    state: &mut CliState,
    approval_tx: mpsc::Sender<ToolApprovalResult>,
    approval_id: String,
    persistence: ApprovalPersistence,
) {
    let _ = approval_tx.try_send(ToolApprovalResult {
        approval_id,
        decision: UserApprovalDecision::Approved { persistence },
    });

    state.pending_approval = None;
    state.mode = UiMode::RunningAgent;
}

fn handle_ui_event(state: &mut CliState, event: UiEvent) {
    match event {
        UiEvent::Agent(event) => apply_stream_event(state, event),
        UiEvent::AgentFailed(err) => {
            state.push_line(UiLine::Error(err));
            state.mode = UiMode::EditingPrompt;
        }
        UiEvent::AgentFinished => {
            state.push_line(UiLine::Model("[completed]".to_string()));
            state.mode = UiMode::EditingPrompt;
        }
    }
}

fn apply_stream_event(state: &mut CliState, event: StreamEvent) {
    match event {
        StreamEvent::TextDelta(text) => state.append_model_delta(text),
        StreamEvent::ThinkingDelta(text) => state.append_thinking_delta(text),
        StreamEvent::ToolCallFinished(call) => {
            state.push_line(UiLine::Tool(format!("{} {}", call.name, call.arguments)));
        }
        StreamEvent::ToolRunStarted {
            name, arguments, ..
        } => {
            state.push_line(UiLine::Tool(format!("started {name}: {arguments}")));
        }
        StreamEvent::ToolRunFinished { name, output, .. } => {
            state.push_line(UiLine::Tool(format!("finished {name}: {output}")));
        }
        StreamEvent::ToolRunFailed { name, error, .. } => {
            state.push_line(UiLine::Error(format!("{name} failed: {error}")));
        }
        StreamEvent::CommandNeedsApproval {
            approval_id,
            reason,
            scope,
            ..
        } => {
            state.pending_approval = Some(PendingApprovalView {
                approval_id,
                reason,
                scope_summary: format!("{scope:?}"),
            });
            state.mode = UiMode::PendingApproval;
        }
        other => {
            state.push_line(UiLine::Command(format!("{other:?}")));
        }
    }
}
