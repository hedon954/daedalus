use std::process::ExitCode;

use daedalus_cli::infrastructure::workspace_fs;
use daedalus_cli::interfaces::tui::app::load_overview;
use daedalus_cli::interfaces::tui::presenter::run_readonly_overview;

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
    let task_dir = workspace_fs::default_task_dir(std::env::args_os().nth(1).map(Into::into))?;
    let overview = load_overview(&task_dir)?;
    run_readonly_overview(overview)
}
