use std::process::ExitCode;

use clap::Parser;
use daedalus_cli::application::init_task::{InitTaskOptions, init_repo_learning};
use daedalus_cli::application::render::render_state;
use daedalus_cli::application::transition_stage::{
    StageAction, TransitionStageOptions, transition_stage,
};
use daedalus_cli::application::validate_workspace::validate_workspace;
use daedalus_cli::domain::{ApprovalSource, DaedalusError};
use daedalus_cli::infrastructure::workspace_fs;
use daedalus_cli::interfaces::agent_cli::args::{Cli, Command, InitKind, StateSubcommand};
use daedalus_cli::interfaces::agent_cli::presenter::{
    print_error, print_init, print_render, print_transition, print_validation,
};

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            print_error(&error, cli.format);
            ExitCode::from(1)
        }
    }
}

fn run(cli: &Cli) -> Result<(), DaedalusError> {
    let cwd = std::env::current_dir().map_err(|source| DaedalusError::Io {
        path: ".".into(),
        source,
    })?;
    let repo_root = workspace_fs::repo_root_from(&cwd)?;

    match &cli.command {
        Command::Init(command) => match &command.kind {
            InitKind::RepoLearning(args) => {
                let output = init_repo_learning(InitTaskOptions {
                    repo_root,
                    name: args.name.clone(),
                    allow_existing_active: args.allow_existing_active,
                    reason: args.reason.clone(),
                })?;
                print_init(&output, cli.format);
            }
        },
        Command::State(command) => match &command.command {
            StateSubcommand::Enter(args) => {
                let task_dir = workspace_fs::default_task_dir(args.task_dir.clone())?;
                let output = transition_stage(TransitionStageOptions {
                    task_dir,
                    stage_id: args.stage_id.clone(),
                    action: StageAction::Enter,
                    reason: args.reason.clone(),
                    actor: "agent".to_owned(),
                })?;
                print_transition(&output, cli.format);
            }
            StateSubcommand::Complete(args) => {
                let approval_source =
                    match args.approval_source.as_deref() {
                        Some(value) => Some(ApprovalSource::parse(value).ok_or_else(|| {
                            DaedalusError::InvalidApprovalSource(value.to_owned())
                        })?),
                        None => None,
                    };
                let task_dir = workspace_fs::default_task_dir(args.task_dir.clone())?;
                let output = transition_stage(TransitionStageOptions {
                    task_dir,
                    stage_id: args.stage_id.clone(),
                    action: StageAction::Complete {
                        force: args.force,
                        approval_source,
                    },
                    reason: args.reason.clone(),
                    actor: "agent".to_owned(),
                })?;
                print_transition(&output, cli.format);
            }
            StateSubcommand::Block(args) => {
                let task_dir = workspace_fs::default_task_dir(args.task_dir.clone())?;
                let output = transition_stage(TransitionStageOptions {
                    task_dir,
                    stage_id: args.stage_id.clone(),
                    action: StageAction::Block,
                    reason: Some(args.reason.clone()),
                    actor: "agent".to_owned(),
                })?;
                print_transition(&output, cli.format);
            }
            StateSubcommand::Resume(args) => {
                let task_dir = workspace_fs::default_task_dir(args.task_dir.clone())?;
                let output = transition_stage(TransitionStageOptions {
                    task_dir,
                    stage_id: args.stage_id.clone(),
                    action: StageAction::Resume,
                    reason: args.reason.clone(),
                    actor: "agent".to_owned(),
                })?;
                print_transition(&output, cli.format);
            }
            StateSubcommand::Render(args) => {
                let task_dir = workspace_fs::default_task_dir(args.task_dir.clone())?;
                let output = render_state(&task_dir)?;
                print_render(&output, cli.format);
            }
        },
        Command::Validate(args) => {
            let task_dir = workspace_fs::default_task_dir(args.task_dir.clone())?;
            let output = validate_workspace(&task_dir, Some(&repo_root))?;
            let ok = output.is_ok();
            if !ok {
                return Err(DaedalusError::WorkspaceValidationFailed(output.issues));
            }
            print_validation(&output, cli.format);
        }
    }

    Ok(())
}
