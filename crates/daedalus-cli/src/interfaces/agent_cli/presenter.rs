use serde_json::json;

use crate::application::close_task::CloseTaskOutput;
use crate::application::ide::RustAnalyzerSyncOutput;
use crate::application::init_task::InitTaskOutput;
use crate::application::knowledge::{
    KnowledgeListOutput, KnowledgeOutput, KnowledgeValidationOutput,
};
use crate::application::migrate::MigrateRepoLearningOutput;
use crate::application::render::RenderedState;
use crate::application::review::{
    ReviewListOutput, ReviewOutput, ReviewSessionOutput, ReviewValidationOutput,
};
use crate::application::topic::TopicOutput;
use crate::application::transition_stage::TransitionStageOutput;
use crate::application::validate_workspace::ValidationOutput;
use crate::domain::{DaedalusError, TopicSnapshot};
use crate::interfaces::agent_cli::args::OutputFormat;

/// 输出初始化任务成功结果。
pub fn print_init(output: &InitTaskOutput, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            println!("ok: initialized repo learning project");
            println!("project_dir: {}", output.task_dir.display());
            println!("topic_dir: {}", output.topic_dir.display());
            println!("state_md: {}", output.state_md.display());
            println!("topic_state_md: {}", output.topic_state_md.display());
            println!("next: fill active topic .daedalus/task-card.md and run daedalus validate");
        }
        OutputFormat::Json => println!(
            "{}",
            json!({
                "ok": true,
                "action": "init",
                "project_dir": output.task_dir,
                "topic_dir": output.topic_dir,
                "state_md": output.state_md,
                "topic_state_md": output.topic_state_md,
                "next": "fill active topic .daedalus/task-card.md and run daedalus validate"
            })
        ),
    }
}

/// 输出阶段状态流转成功结果。
pub fn print_transition(output: &TransitionStageOutput, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            println!("ok: state transition completed");
            println!("action: {}", output.action);
            println!("stage: {}", output.stage_id);
            println!("topic_dir: {}", output.task_dir.display());
            println!("state_md: {}", output.state_md.display());
        }
        OutputFormat::Json => println!(
            "{}",
            json!({
                "ok": true,
                "action": output.action,
                "stage": output.stage_id,
                "topic_dir": output.task_dir,
                "state_md": output.state_md
            })
        ),
    }
}

/// 输出任务生命周期关闭结果。
pub fn print_close_task(output: &CloseTaskOutput, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            let label = match output.action.as_str() {
                "task-complete" => "completed",
                "abandon" => "abandoned",
                _ => output.action.as_str(),
            };
            println!("ok: task {label}");
            println!("lifecycle: {}", output.lifecycle);
            println!("moved: {}", output.moved);
            println!("from_task_dir: {}", output.from_task_dir.display());
            println!("to_task_dir: {}", output.to_task_dir.display());
            println!("state_md: {}", output.state_md.display());
            println!("decision_log: {}", output.decision_log.display());
            println!("next: {}", output.next);
        }
        OutputFormat::Json => println!(
            "{}",
            json!({
                "ok": true,
                "action": output.action,
                "lifecycle": output.lifecycle,
                "moved": output.moved,
                "from_task_dir": output.from_task_dir,
                "to_task_dir": output.to_task_dir,
                "state_md": output.state_md,
                "decision_log": output.decision_log,
                "next": output.next
            })
        ),
    }
}

/// 输出 `state.md` 渲染结果。
pub fn print_render(output: &RenderedState, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            println!("ok: rendered state.md");
            println!("path: {}", output.path.display());
        }
        OutputFormat::Json => println!(
            "{}",
            json!({
                "ok": true,
                "action": "render",
                "path": output.path
            })
        ),
    }
}

/// 输出 workspace 校验结果。
pub fn print_validation(output: &ValidationOutput, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            if output.is_ok() {
                println!("ok: workspace valid");
                println!("task_dir: {}", output.task_dir.display());
                println!(
                    "note: validate checks structure and required files only; notes still need user-owned evidence"
                );
            } else {
                println!("error: workspace validation failed");
                println!("task_dir: {}", output.task_dir.display());
                for issue in &output.issues {
                    println!("issue: {issue}");
                }
                println!(
                    "next: fix listed issues or regenerate derived state with daedalus state render"
                );
            }
        }
        OutputFormat::Json => println!(
            "{}",
            json!({
                "ok": output.is_ok(),
                "error": if output.is_ok() { serde_json::Value::Null } else { json!("workspace_validation_failed") },
                "task_dir": output.task_dir,
                "issues": output.issues,
                "note": "validate checks structure and required files only; notes still need user-owned evidence",
                "next": if output.is_ok() { "continue" } else { "fix listed issues or regenerate derived state with daedalus state render" }
            })
        ),
    }
}

/// 输出 rust-analyzer linkedProjects 同步结果。
pub fn print_rust_analyzer_sync(output: &RustAnalyzerSyncOutput, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            println!("ok: rust-analyzer linkedProjects synced");
            println!("settings: {}", output.settings_path.display());
            for project in &output.linked_projects {
                println!("linked_project: {project}");
            }
        }
        OutputFormat::Json => println!(
            "{}",
            json!({
                "ok": true,
                "action": "ide-sync-rust-analyzer",
                "settings": output.settings_path,
                "linked_projects": output.linked_projects
            })
        ),
    }
}

/// 输出 topic 操作结果。
pub fn print_topic(output: &TopicOutput, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            println!("ok: topic operation completed");
            println!("action: {}", output.action);
            println!("topic: {}", output.slug);
            println!("project_dir: {}", output.project_dir.display());
            println!("topic_dir: {}", output.topic_dir.display());
            println!("state_md: {}", output.state_md.display());
        }
        OutputFormat::Json => println!(
            "{}",
            json!({
                "ok": true,
                "action": output.action,
                "topic": output.slug,
                "project_dir": output.project_dir,
                "topic_dir": output.topic_dir,
                "state_md": output.state_md
            })
        ),
    }
}

/// 输出 topic 列表。
pub fn print_topic_list(
    project_dir: &std::path::Path,
    topics: &[TopicSnapshot],
    format: OutputFormat,
) {
    match format {
        OutputFormat::Text => {
            println!("ok: topics");
            println!("project_dir: {}", project_dir.display());
            for topic in topics {
                println!(
                    "topic: {} | {} | {} | {}",
                    topic.slug, topic.title, topic.lifecycle, topic.path
                );
            }
        }
        OutputFormat::Json => println!(
            "{}",
            json!({
                "ok": true,
                "project_dir": project_dir,
                "topics": topics.iter().map(|topic| {
                    json!({
                        "slug": &topic.slug,
                        "title": &topic.title,
                        "lifecycle": &topic.lifecycle,
                        "path": &topic.path,
                        "inherits": &topic.inherits
                    })
                }).collect::<Vec<_>>()
            })
        ),
    }
}

/// 输出 topic 校验结果。
pub fn print_topic_validation(project_dir: &std::path::Path, slug: &str, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            println!("ok: topic valid");
            println!("project_dir: {}", project_dir.display());
            println!("topic: {slug}");
        }
        OutputFormat::Json => println!(
            "{}",
            json!({
                "ok": true,
                "action": "topic-validate",
                "project_dir": project_dir,
                "topic": slug
            })
        ),
    }
}

/// 输出 review 操作结果。
pub fn print_review(output: &ReviewOutput, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            println!("ok: review operation completed");
            println!("action: {}", output.action);
            println!("review: {}", output.review_id);
            println!("project_dir: {}", output.project_dir.display());
            println!("review_dir: {}", output.review_dir.display());
            if let Some(state_md) = &output.state_md {
                println!("state_md: {}", state_md.display());
            }
        }
        OutputFormat::Json => println!(
            "{}",
            json!({
                "ok": true,
                "action": output.action,
                "review": output.review_id,
                "project_dir": output.project_dir,
                "review_dir": output.review_dir,
                "state_md": output.state_md
            })
        ),
    }
}

/// 输出 review 列表。
pub fn print_review_list(output: &ReviewListOutput, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            println!("ok: reviews");
            println!("project_dir: {}", output.project_dir.display());
            for review in &output.reviews {
                println!(
                    "review: {} | {}:{} | {} | {} | {}",
                    review.id,
                    review.target_type,
                    review.target,
                    review.mode,
                    review.lifecycle,
                    review.path
                );
            }
        }
        OutputFormat::Json => println!(
            "{}",
            json!({
                "ok": true,
                "project_dir": output.project_dir,
                "reviews": output.reviews.iter().map(|review| {
                    json!({
                        "id": &review.id,
                        "target_type": &review.target_type,
                        "target": &review.target,
                        "mode": &review.mode,
                        "lifecycle": &review.lifecycle,
                        "path": &review.path,
                        "next_action": &review.next_action
                    })
                }).collect::<Vec<_>>()
            })
        ),
    }
}

/// 输出 review session 操作结果。
pub fn print_review_session(output: &ReviewSessionOutput, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            println!("ok: review session operation completed");
            println!("action: {}", output.action);
            println!("review: {}", output.review_id);
            println!("session: {}", output.session_id);
            println!("project_dir: {}", output.project_dir.display());
            println!("review_dir: {}", output.review_dir.display());
            println!("session_path: {}", output.session_path.display());
        }
        OutputFormat::Json => println!(
            "{}",
            json!({
                "ok": true,
                "action": output.action,
                "review": output.review_id,
                "session": output.session_id,
                "project_dir": output.project_dir,
                "review_dir": output.review_dir,
                "session_path": output.session_path
            })
        ),
    }
}

/// 输出 review 校验结果。
pub fn print_review_validation(output: &ReviewValidationOutput, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            println!("ok: review valid");
            println!("project_dir: {}", output.project_dir.display());
            println!("review: {}", output.review_id);
            println!("review_dir: {}", output.review_dir.display());
        }
        OutputFormat::Json => println!(
            "{}",
            json!({
                "ok": true,
                "action": "review-validate",
                "project_dir": output.project_dir,
                "review": output.review_id,
                "review_dir": output.review_dir
            })
        ),
    }
}

/// 输出 knowledge 操作结果。
pub fn print_knowledge(output: &KnowledgeOutput, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            println!("ok: knowledge operation completed");
            println!("action: {}", output.action);
            println!("path: {}", output.path.display());
            println!("next: {}", output.next);
        }
        OutputFormat::Json => println!(
            "{}",
            json!({
                "ok": true,
                "action": output.action,
                "path": output.path,
                "next": output.next
            })
        ),
    }
}

/// 输出 knowledge 列表。
pub fn print_knowledge_list(output: &KnowledgeListOutput, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            println!("ok: knowledge items");
            println!("root: {}", output.root.display());
            for item in &output.items {
                println!(
                    "knowledge: {} | {} | {} | {}",
                    item.level, item.name, item.status, item.path
                );
            }
        }
        OutputFormat::Json => println!(
            "{}",
            json!({
                "ok": true,
                "root": output.root,
                "items": output.items.iter().map(|item| {
                    json!({
                        "level": &item.level,
                        "name": &item.name,
                        "path": &item.path,
                        "status": &item.status
                    })
                }).collect::<Vec<_>>()
            })
        ),
    }
}

/// 输出 knowledge 校验结果。
pub fn print_knowledge_validation(output: &KnowledgeValidationOutput, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            println!("ok: knowledge valid");
            println!("action: {}", output.action);
            println!("root: {}", output.root.display());
        }
        OutputFormat::Json => println!(
            "{}",
            json!({
                "ok": true,
                "action": output.action,
                "root": output.root
            })
        ),
    }
}

/// 输出迁移结果。
pub fn print_migration(output: &MigrateRepoLearningOutput, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            if output.execute {
                println!("ok: migration completed");
            } else {
                println!("ok: migration dry-run");
            }
            println!("project_dir: {}", output.project_dir.display());
            println!("topic_dir: {}", output.topic_dir.display());
            if let Some(backup_dir) = &output.backup_dir {
                println!("backup_dir: {}", backup_dir.display());
            }
            if let Some(state_md) = &output.state_md {
                println!("state_md: {}", state_md.display());
            }
            if let Some(topic_state_md) = &output.topic_state_md {
                println!("topic_state_md: {}", topic_state_md.display());
            }
            for action in &output.planned_actions {
                println!("action: {action}");
            }
        }
        OutputFormat::Json => println!(
            "{}",
            json!({
                "ok": true,
                "action": "migrate-repo-learning-multi-topic",
                "execute": output.execute,
                "project_dir": &output.project_dir,
                "topic_dir": &output.topic_dir,
                "backup_dir": &output.backup_dir,
                "state_md": &output.state_md,
                "topic_state_md": &output.topic_state_md,
                "planned_actions": &output.planned_actions
            })
        ),
    }
}

/// 输出稳定、可行动的错误信息。
///
/// Agent 会依赖这些错误文本或 JSON 字段来决定下一步，因此措辞应保持稳定。
pub fn print_error(error: &DaedalusError, format: OutputFormat) {
    match format {
        OutputFormat::Text => print_text_error(error),
        OutputFormat::Json => print_json_error(error),
    }
}

fn print_text_error(error: &DaedalusError) {
    match error {
        DaedalusError::MissingRequiredArtifact { artifact, stage } => {
            eprintln!("error: missing required artifact");
            eprintln!("artifact: {}", artifact.display());
            eprintln!("stage: {stage}");
            eprintln!(
                "next: complete the artifact, ask the user for confirmation, or rerun with --force --reason <reason> --approval-source <source> only if explicitly approved"
            );
        }
        DaedalusError::ForceRequiresApproval => {
            eprintln!("error: force requires approval");
            eprintln!(
                "next: ask the user for explicit approval, document equivalent evidence, or complete the missing artifact first"
            );
        }
        DaedalusError::TaskAlreadyActive(path) => {
            eprintln!("error: task already active");
            eprintln!("task_dir: {}", path.display());
            eprintln!(
                "next: complete or move the active task before initializing a new one; use --allow-existing-active --reason <reason> only with explicit approval"
            );
        }
        DaedalusError::TaskMoveDestinationExists(path) => {
            eprintln!("error: task move destination already exists");
            eprintln!("to_task_dir: {}", path.display());
            eprintln!(
                "next: inspect the existing destination before retrying; never overwrite completed or abandoned learning history"
            );
        }
        DaedalusError::TaskLifecycleReasonRequired => {
            eprintln!("error: task lifecycle reason required");
            eprintln!(
                "next: rerun with --reason <specific reason> so future Agents understand why the task moved"
            );
        }
        DaedalusError::InvalidTaskLifecycleTransition(message) => {
            eprintln!("error: invalid task lifecycle transition");
            eprintln!("detail: {message}");
            eprintln!(
                "next: inspect workspaces/.daedalus/current.toml plus the active project/topic .daedalus/state.toml, then run daedalus validate"
            );
        }
        DaedalusError::TaskLifecycleLocationMismatch(message) => {
            eprintln!("error: task lifecycle location mismatch");
            eprintln!("detail: {message}");
            eprintln!(
                "next: keep task.lifecycle, task.workspace_bucket, and the workspace directory bucket consistent"
            );
        }
        DaedalusError::NotDaedalusProject(path) => {
            eprintln!("error: not a daedalus project directory");
            eprintln!("cwd: {}", path.display());
            eprintln!(
                "next: run this command from the daedalus project root or one of its subdirectories"
            );
        }
        DaedalusError::WorkspaceValidationFailed(issues) => {
            eprintln!("error: workspace validation failed");
            for issue in issues {
                eprintln!("issue: {issue}");
            }
            eprintln!("next: fix listed issues and rerun daedalus validate");
        }
        _ => {
            eprintln!("error: {error}");
            eprintln!(
                "next: inspect the workspace and rerun the command with explicit paths if needed"
            );
        }
    }
}

fn print_json_error(error: &DaedalusError) {
    let value = match error {
        DaedalusError::MissingRequiredArtifact { artifact, stage } => json!({
            "ok": false,
            "error": "missing_required_artifact",
            "artifact": artifact,
            "stage": stage,
            "next": "complete the artifact, ask the user for confirmation, or rerun with --force --reason <reason> --approval-source <source> only if explicitly approved",
            "force_requires_user_approval": true
        }),
        DaedalusError::ForceRequiresApproval => json!({
            "ok": false,
            "error": "force_requires_approval",
            "next": "ask the user for explicit approval, document equivalent evidence, or complete the missing artifact first",
            "force_requires_user_approval": true
        }),
        DaedalusError::TaskAlreadyActive(path) => json!({
            "ok": false,
            "error": "task_already_active",
            "task_dir": path,
            "next": "complete or move the active task before initializing a new one; use --allow-existing-active --reason <reason> only with explicit approval"
        }),
        DaedalusError::TaskMoveDestinationExists(path) => json!({
            "ok": false,
            "error": "task_move_destination_exists",
            "to_task_dir": path,
            "next": "inspect the existing destination before retrying; never overwrite completed or abandoned learning history"
        }),
        DaedalusError::TaskLifecycleReasonRequired => json!({
            "ok": false,
            "error": "task_lifecycle_reason_required",
            "next": "rerun with --reason <specific reason> so future Agents understand why the task moved"
        }),
        DaedalusError::InvalidTaskLifecycleTransition(message) => json!({
            "ok": false,
            "error": "invalid_task_lifecycle_transition",
            "detail": message,
            "next": "inspect workspaces/.daedalus/current.toml plus the active project/topic .daedalus/state.toml, then run daedalus validate"
        }),
        DaedalusError::TaskLifecycleLocationMismatch(message) => json!({
            "ok": false,
            "error": "task_lifecycle_location_mismatch",
            "detail": message,
            "next": "keep task.lifecycle, task.workspace_bucket, and the workspace directory bucket consistent"
        }),
        DaedalusError::NotDaedalusProject(path) => json!({
            "ok": false,
            "error": "not_daedalus_project",
            "cwd": path,
            "next": "run this command from the daedalus project root or one of its subdirectories"
        }),
        _ => json!({
            "ok": false,
            "error": error.to_string(),
            "next": "inspect the workspace and rerun the command with explicit paths if needed"
        }),
    };
    eprintln!("{value}");
}
