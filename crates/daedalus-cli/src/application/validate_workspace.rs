use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::{DaedalusError, Result, TaskLifecycle, WorkspaceBucket};
use crate::infrastructure::{state_toml, workspace_fs};

/// workspace 校验结果。
#[derive(Debug, Clone)]
pub struct ValidationOutput {
    /// 被校验的学习任务目录。
    pub task_dir: PathBuf,
    /// 校验发现的问题列表；为空表示通过。
    pub issues: Vec<String>,
}

impl ValidationOutput {
    /// 判断校验是否通过。
    pub fn is_ok(&self) -> bool {
        self.issues.is_empty()
    }
}

/// 校验学习任务 workspace 的文件和状态一致性。
///
/// 该函数只读取文件系统，不会自动修复问题。调用方应根据 `issues`
/// 决定是否提示用户、重新渲染状态，或阻止阶段完成。
pub fn validate_workspace(task_dir: &Path, repo_root: Option<&Path>) -> Result<ValidationOutput> {
    let mut issues = Vec::new();
    let required = [
        "CLAUDE.md",
        ".daedalus/task-card.md",
        ".daedalus/state.toml",
        ".daedalus/state.md",
        ".daedalus/todo.md",
        ".daedalus/long-context.md",
        ".daedalus/artifact-index.md",
        ".daedalus/decision-log.md",
        "demo/.gitkeep",
        "notes/.gitkeep",
        "source/.gitkeep",
    ];

    for path in required {
        if !task_dir.join(path).exists() {
            issues.push(format!("missing required file: {path}"));
        }
    }

    let state_path = state_toml::state_path(task_dir);
    let doc = match state_toml::load_state_doc(&state_path) {
        Ok(doc) => doc,
        Err(error) => {
            issues.push(format!("state.toml cannot be parsed: {error}"));
            return Ok(ValidationOutput {
                task_dir: task_dir.to_path_buf(),
                issues,
            });
        }
    };

    if state_toml::active_stage_count(&doc) > 1 {
        issues.push("multiple active stages".to_owned());
    }

    if let Some(current_phase) = state_toml::current_phase(&doc) {
        if !state_toml::stage_exists(&doc, &current_phase) {
            issues.push(format!(
                "current_phase points to unknown stage: {current_phase}"
            ));
        }
    } else {
        issues.push("current_phase is missing".to_owned());
    }

    match (
        state_toml::task_lifecycle(&doc),
        state_toml::workspace_bucket(&doc),
        workspace_fs::bucket_from_task_dir(task_dir),
    ) {
        (Ok(lifecycle), Ok(state_bucket), Ok(actual_bucket)) => {
            let expected_bucket = expected_bucket_for_lifecycle(lifecycle);
            if state_bucket != expected_bucket {
                issues.push(format!(
                    "task lifecycle `{}` expects workspace_bucket `{}`, got `{}`",
                    lifecycle.as_str(),
                    expected_bucket.as_str(),
                    state_bucket.as_str()
                ));
            }
            if actual_bucket != expected_bucket {
                issues.push(format!(
                    "task lifecycle `{}` expects directory bucket `{}`, got `{}`",
                    lifecycle.as_str(),
                    expected_bucket.as_str(),
                    actual_bucket.as_str()
                ));
            }
        }
        (lifecycle, state_bucket, actual_bucket) => {
            if let Err(error) = lifecycle {
                issues.push(format!("invalid task lifecycle: {error}"));
            }
            if let Err(error) = state_bucket {
                issues.push(format!("invalid workspace_bucket: {error}"));
            }
            if let Err(error) = actual_bucket {
                issues.push(format!("invalid task directory bucket: {error}"));
            }
        }
    }

    let state_md = state_toml::state_md_path(task_dir);
    if stale(&state_path, &state_md)? {
        issues.push("state.md is stale or missing".to_owned());
    }

    for stage in state_toml::stages(&doc) {
        if stage.status == "done" {
            for artifact in stage.required_artifacts {
                if !task_dir.join(&artifact).exists() {
                    issues.push(format!(
                        "completed stage `{}` missing artifact: {artifact}",
                        stage.id
                    ));
                }
            }
        }
    }

    if let Some(repo_root) = repo_root {
        let tasks = workspace_fs::active_tasks(&workspace_fs::learning_root(repo_root))?;
        if tasks.len() > 1 {
            issues.push(
                "WIP violation: more than one learning task exists in workspaces/02-learning"
                    .to_owned(),
            );
        }
    }

    Ok(ValidationOutput {
        task_dir: task_dir.to_path_buf(),
        issues,
    })
}

fn expected_bucket_for_lifecycle(lifecycle: TaskLifecycle) -> WorkspaceBucket {
    match lifecycle {
        TaskLifecycle::Active => WorkspaceBucket::Learning,
        TaskLifecycle::Completed => WorkspaceBucket::Completed,
        TaskLifecycle::Abandoned => WorkspaceBucket::Abandoned,
    }
}

fn stale(source: &Path, generated: &Path) -> Result<bool> {
    if !generated.exists() {
        return Ok(true);
    }
    let source_time = fs::metadata(source)
        .map_err(|source_err| DaedalusError::Io {
            path: source.to_path_buf(),
            source: source_err,
        })?
        .modified()
        .map_err(|source_err| DaedalusError::Io {
            path: source.to_path_buf(),
            source: source_err,
        })?;
    let generated_time = fs::metadata(generated)
        .map_err(|source_err| DaedalusError::Io {
            path: generated.to_path_buf(),
            source: source_err,
        })?
        .modified()
        .map_err(|source_err| DaedalusError::Io {
            path: generated.to_path_buf(),
            source: source_err,
        })?;
    Ok(generated_time < source_time)
}
