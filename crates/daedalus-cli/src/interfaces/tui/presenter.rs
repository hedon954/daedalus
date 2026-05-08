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
use crate::interfaces::tui::app::{TuiApp, TuiOverview, TuiView};
use crate::interfaces::tui::screens::{draw_overview, draw_selector};

/// 启动只读 TUI 总览。
///
/// 当前版本只负责展示学习任务状态，不提供写操作，避免绕过 application use case。
pub fn run_readonly_overview(overview: TuiOverview) -> Result<()> {
    run_tui(TuiApp {
        view: TuiView::TaskOverview,
        tasks: Vec::new(),
        selected_task: 0,
        overview: Some(overview),
        can_return_to_selector: false,
    })
}

/// 启动只读 TUI。
///
/// 支持 workspace 任务选择页和单任务详情页，所有状态更新仍必须通过 `daedalus` CLI。
pub fn run_tui(mut app: TuiApp) -> Result<()> {
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
            .draw(|frame| match app.view {
                TuiView::TaskSelector => {
                    draw_selector(frame, frame.area(), &app.tasks, app.selected_task)
                }
                TuiView::TaskOverview => {
                    if let Some(overview) = &app.overview {
                        draw_overview(frame, frame.area(), overview, app.can_return_to_selector);
                    }
                }
            })
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
        })? {
            match (app.view, key.code) {
                (_, KeyCode::Char('q') | KeyCode::Esc) => break Ok(()),
                (TuiView::TaskSelector, KeyCode::Up | KeyCode::Char('k')) => {
                    app.select_previous();
                }
                (TuiView::TaskSelector, KeyCode::Down | KeyCode::Char('j')) => {
                    app.select_next();
                }
                (TuiView::TaskSelector, KeyCode::Enter) => {
                    app.open_selected_task()?;
                }
                (TuiView::TaskOverview, KeyCode::Backspace | KeyCode::Char('b')) => {
                    app.return_to_selector();
                }
                _ => {}
            }
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
