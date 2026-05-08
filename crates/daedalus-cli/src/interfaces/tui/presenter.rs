use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::domain::{DaedalusError, Result};
use crate::interfaces::tui::app::TuiOverview;
use crate::interfaces::tui::screens::draw_overview;

/// 启动只读 TUI 总览。
///
/// 当前版本只负责展示学习任务状态，不提供写操作，避免绕过 application use case。
pub fn run_readonly_overview(overview: TuiOverview) -> Result<()> {
    enable_raw_mode().map_err(|source| DaedalusError::Io {
        path: "terminal".into(),
        source,
    })?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|source| DaedalusError::Io {
        path: "terminal".into(),
        source,
    })?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|source| DaedalusError::Io {
        path: "terminal".into(),
        source,
    })?;

    let result = loop {
        terminal
            .draw(|frame| draw_overview(frame, frame.area(), &overview))
            .map_err(|source| DaedalusError::Io {
                path: "terminal".into(),
                source,
            })?;

        if event::poll(Duration::from_millis(200)).map_err(|source| DaedalusError::Io {
            path: "terminal".into(),
            source,
        })? && let Event::Key(key) = event::read().map_err(|source| DaedalusError::Io {
            path: "terminal".into(),
            source,
        })? && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
        {
            break Ok(());
        }
    };

    disable_raw_mode().map_err(|source| DaedalusError::Io {
        path: "terminal".into(),
        source,
    })?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(|source| DaedalusError::Io {
        path: "terminal".into(),
        source,
    })?;
    terminal.show_cursor().map_err(|source| DaedalusError::Io {
        path: "terminal".into(),
        source,
    })?;

    result
}
