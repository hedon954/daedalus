use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

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

pub async fn run_cli(
    agent: Arc<ReActAgent>,
    approval_tx: mpsc::Sender<ToolApprovalResult>,
) -> anyhow::Result<()> {
    let mut state = CliState::new();
    let (ui_tx, ui_rx) = mpsc::channel(128);
    let (key_tx, key_rx) = mpsc::channel(128);

    let _guard = TerminalGuard::enter()?;
    let _input_guard = InputThreadGuard::spawn(key_tx);

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("failed to create terminal")?;

    let result = run_event_loop(
        &mut terminal,
        &mut state,
        agent,
        approval_tx,
        ui_tx,
        ui_rx,
        key_rx,
    )
    .await;

    terminal.show_cursor().context("failed to show cursor")?;

    result
}

struct InputThreadGuard {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl InputThreadGuard {
    fn spawn(key_tx: mpsc::Sender<KeyEvent>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_for_thread = Arc::clone(&stop);

        let handle = thread::spawn(move || {
            while !stop_for_thread.load(Ordering::Relaxed) {
                let Ok(ready) = event::poll(Duration::from_millis(50)) else {
                    continue;
                };
                if !ready {
                    continue;
                }

                let Ok(Event::Key(key)) = event::read() else {
                    continue;
                };

                if key_tx.blocking_send(key).is_err() {
                    break;
                }
            }
        });

        Self {
            stop,
            handle: Some(handle),
        }
    }
}

impl Drop for InputThreadGuard {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut CliState,
    agent: Arc<ReActAgent>,
    approval_tx: mpsc::Sender<ToolApprovalResult>,
    ui_tx: mpsc::Sender<UiEvent>,
    mut ui_rx: mpsc::Receiver<UiEvent>,
    mut key_rx: mpsc::Receiver<KeyEvent>,
) -> anyhow::Result<()> {
    loop {
        drain_ui_events(state, &mut ui_rx);

        terminal
            .draw(|frame| view::draw(frame, frame.area(), state))
            .context("failed to draw cli")?;

        if state.should_quit {
            break;
        }

        tokio::select! {
            Some(event) = ui_rx.recv() => {
                handle_ui_event(state, event);
            }
            Some(key) = key_rx.recv() => {
                let command = handle_key(state, key);
                handle_command(
                    state,
                    command,
                    agent.clone(),
                    approval_tx.clone(),
                    ui_tx.clone(),
                );
            }
            _ = tokio::time::sleep(Duration::from_millis(250)) => {}
        }
    }

    Ok(())
}

fn drain_ui_events(state: &mut CliState, ui_rx: &mut mpsc::Receiver<UiEvent>) {
    while let Ok(event) = ui_rx.try_recv() {
        handle_ui_event(state, event);
    }
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
        KeyCode::PageUp => {
            state.scroll_log_page_up();
            return UiCommand::None;
        }
        KeyCode::PageDown => {
            state.scroll_log_page_down();
            return UiCommand::None;
        }
        KeyCode::Home => {
            state.scroll_log_home();
            return UiCommand::None;
        }
        KeyCode::End => {
            state.scroll_to_bottom();
            return UiCommand::None;
        }
        _ => {}
    }

    match state.mode {
        UiMode::EditingPrompt => handle_prompt_key(state, key),
        UiMode::RunningAgent => handle_running_key(state, key),
        UiMode::PendingApproval => handle_approval_key(state, key),
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
            state.set_running(prompt.clone());

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
            state.push_line(UiLine::Approval("approved once".to_string()));
            send_approval(state, approval_tx, approval_id, ApprovalPersistence::Once);
        }
        UiCommand::ApproveSession(approval_id) => {
            state.push_line(UiLine::Approval("approved for this session".to_string()));
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
            state.push_line(UiLine::Approval("rejected".to_string()));
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
            state.set_ready();
        }
        UiEvent::AgentFinished => {
            state.push_line(UiLine::System("turn completed".to_string()));
            state.set_ready();
        }
    }
}

fn apply_stream_event(state: &mut CliState, event: StreamEvent) {
    match event {
        StreamEvent::TextDelta(text) => state.append_model_delta(text),
        StreamEvent::ThinkingDelta(text) => state.append_thinking_delta(text),
        StreamEvent::ToolCallFinished(call) => {
            state.push_line(UiLine::Tool(format!(
                "model selected {} {}",
                call.name, call.arguments
            )));
        }
        StreamEvent::ToolRunStarted {
            name, arguments, ..
        } => {
            state.push_line(UiLine::Tool(format!("started {name} {arguments}")));
        }
        StreamEvent::ToolRunFinished { name, output, .. } => {
            state.push_line(UiLine::Tool(format!("finished {name}: {output}")));
        }
        StreamEvent::ToolRunFailed { name, error, .. } => {
            state.push_line(UiLine::Error(format!("{name} failed: {error}")));
        }
        StreamEvent::CommandNeedsApproval {
            approval_id,
            name,
            reason,
            scope,
            ..
        } => {
            state.pending_approval = Some(PendingApprovalView {
                approval_id,
                reason,
                scope_summary: format!("{scope:?}"),
            });
            state.push_line(UiLine::Approval(format!("{name} needs approval")));
            state.mode = UiMode::PendingApproval;
        }
        StreamEvent::CommandExecutionStarted { name, attempt, .. } => {
            state.push_line(UiLine::Command(format!(
                "{name} execution started: {attempt:?}"
            )));
        }
        StreamEvent::CommandExecutionFinished {
            name,
            attempt,
            output,
            ..
        } => {
            state.push_line(UiLine::Command(format!(
                "{name} execution finished: {attempt:?}; output: {output}"
            )));
        }
        StreamEvent::CommandExecutionFailed {
            name,
            attempt,
            error,
            ..
        } => {
            state.push_line(UiLine::Error(format!(
                "{name} execution failed: {attempt:?}; error: {error}"
            )));
        }
        StreamEvent::CommandRetryEvaluated { name, decision, .. } => {
            state.push_line(UiLine::Command(format!(
                "{name} retry decision: {decision:?}"
            )));
        }
        StreamEvent::Started => {
            state.push_line(UiLine::System("turn started".to_string()));
        }
        StreamEvent::Error(err) => {
            state.push_line(UiLine::Error(err));
        }
        StreamEvent::Completed => {
            state.push_line(UiLine::System("turn completed".to_string()));
        }
        other => {
            state.push_line(UiLine::Command(format!("{other:?}")));
        }
    }
}
