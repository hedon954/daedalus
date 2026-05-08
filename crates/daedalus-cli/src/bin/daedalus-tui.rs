use std::process::ExitCode;

use daedalus_cli::infrastructure::workspace_fs;
use daedalus_cli::interfaces::tui::app::TuiApp;
use daedalus_cli::interfaces::tui::presenter::run_tui;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            eprintln!("next: run daedalus validate or pass through an active task directory");
            ExitCode::from(1)
        }
    }
}

fn run() -> daedalus_cli::domain::Result<()> {
    let cwd =
        std::env::current_dir().map_err(|source| daedalus_cli::domain::DaedalusError::Io {
            path: ".".into(),
            source,
        })?;
    let repo_root = workspace_fs::repo_root_from(&cwd)?;
    let app =
        TuiApp::from_launch_context(&repo_root, &cwd, std::env::args_os().nth(1).map(Into::into))?;
    run_tui(app)
}
