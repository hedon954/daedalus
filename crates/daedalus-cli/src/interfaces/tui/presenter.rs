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
use crate::interfaces::tui::app::{DetailSource, OverviewFocus, TuiApp, TuiOverview, TuiView};
use crate::interfaces::tui::screens::{draw_detail, draw_overview, draw_selector};

/// 启动只读 TUI 总览。
///
/// 当前版本只负责展示学习任务状态，不提供写操作，避免绕过 application use case。
pub fn run_readonly_overview(overview: TuiOverview) -> Result<()> {
    run_tui(TuiApp {
        view: TuiView::TaskOverview,
        tasks: Vec::new(),
        selected_task: 0,
        overview_focus: OverviewFocus::NextAction,
        overview: Some(overview),
        detail: None,
        detail_scroll: 0,
        can_return_to_selector: false,
        repo_root: None,
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
                        draw_overview(
                            frame,
                            frame.area(),
                            overview,
                            app.overview_focus,
                            app.can_return_to_selector,
                        );
                    }
                }
                TuiView::Detail => {
                    if let Some(detail) = &app.detail {
                        draw_detail(
                            frame,
                            frame.area(),
                            detail,
                            app.detail_scroll,
                            app.can_return_to_selector,
                        );
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
            let detail_viewport = terminal
                .size()
                .map(|area| {
                    (
                        area.height.saturating_sub(8) as usize,
                        area.width.saturating_sub(6) as usize,
                    )
                })
                .map_err(|source| DaedalusError::Io {
                    path: "terminal".into(),
                    source,
                })?;
            match (app.view, key.code) {
                (_, KeyCode::Char('q') | KeyCode::Esc) => break Ok(()),
                (_, KeyCode::Char('r')) => {
                    app.refresh()?;
                }
                (TuiView::TaskSelector, KeyCode::Up | KeyCode::Char('k')) => {
                    app.select_previous();
                }
                (TuiView::TaskSelector, KeyCode::Down | KeyCode::Char('j')) => {
                    app.select_next();
                }
                (TuiView::TaskSelector, KeyCode::Enter) => {
                    app.open_selected_task()?;
                }
                (TuiView::TaskOverview, KeyCode::Up | KeyCode::Char('k')) => {
                    app.focus_previous();
                }
                (TuiView::TaskOverview, KeyCode::Down | KeyCode::Char('j')) => {
                    app.focus_next();
                }
                (TuiView::TaskOverview, KeyCode::Enter) => {
                    app.open_focused_detail()?;
                }
                (TuiView::TaskOverview, KeyCode::Char('g')) => {
                    app.open_detail(DetailSource::Guide)?;
                }
                (TuiView::TaskOverview, KeyCode::Char('t')) => {
                    app.open_detail(DetailSource::Todo)?;
                }
                (TuiView::TaskOverview, KeyCode::Char('o')) => {
                    app.open_detail(DetailSource::OutcomeMap)?;
                }
                (TuiView::TaskOverview, KeyCode::Backspace | KeyCode::Char('b')) => {
                    app.return_to_selector();
                }
                (TuiView::Detail, KeyCode::Backspace | KeyCode::Char('b')) => {
                    app.close_detail();
                }
                (TuiView::Detail, KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('u')) => {
                    app.scroll_detail_up(1);
                }
                (TuiView::Detail, KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('d')) => {
                    app.scroll_detail_down(1, detail_viewport.0, detail_viewport.1);
                }
                (TuiView::Detail, KeyCode::PageUp) => {
                    app.scroll_detail_up(detail_viewport.0);
                }
                (TuiView::Detail, KeyCode::PageDown) => {
                    app.scroll_detail_down(detail_viewport.0, detail_viewport.0, detail_viewport.1);
                }
                (TuiView::Detail, KeyCode::Home) => {
                    app.scroll_detail_top();
                }
                (TuiView::Detail, KeyCode::End) => {
                    app.scroll_detail_bottom(detail_viewport.0, detail_viewport.1);
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
