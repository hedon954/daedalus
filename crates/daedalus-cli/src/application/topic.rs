use std::fs;
use std::path::{Path, PathBuf};

use crate::application::ide::sync_rust_analyzer_linked_projects;
use crate::application::render::render_state;
use crate::application::validate_workspace::validate_topic_workspace;
use crate::domain::transition::Transition;
use crate::domain::{
    DaedalusError, Result, TaskLifecycle, TopicLifecycle, TopicSnapshot, WorkspaceBucket,
};
use crate::infrastructure::{clock, state_toml, template_fs, workspace_fs};

/// 新建 topic 参数。
#[derive(Debug, Clone)]
pub struct NewTopicOptions {
    pub repo_root: PathBuf,
    pub project_dir: PathBuf,
    pub slug: String,
    pub title: String,
    pub actor: String,
}

/// topic 操作输出。
#[derive(Debug, Clone)]
pub struct TopicOutput {
    pub project_dir: PathBuf,
    pub topic_dir: PathBuf,
    pub slug: String,
    pub action: String,
    pub state_md: PathBuf,
}

/// 激活 topic 参数。
#[derive(Debug, Clone)]
pub struct ActivateTopicOptions {
    pub repo_root: PathBuf,
    pub project_dir: PathBuf,
    pub slug: String,
    pub actor: String,
}

/// 关闭 topic 参数。
#[derive(Debug, Clone)]
pub struct CloseTopicOptions {
    pub repo_root: PathBuf,
    pub project_dir: PathBuf,
    pub slug: String,
    pub lifecycle: TopicLifecycle,
    pub reason: String,
    pub actor: String,
}

/// 新建 topic。
pub fn new_topic(options: NewTopicOptions) -> Result<TopicOutput> {
    let project_dir = options.project_dir;
    let slug = sanitize_slug(&options.slug);
    let title = if options.title.trim().is_empty() {
        slug.clone()
    } else {
        options.title.trim().to_owned()
    };
    let topic_dir = project_dir.join("topics").join(&slug);
    if topic_dir.exists() {
        return Err(DaedalusError::TopicNotFound(format!(
            "topic already exists: {slug}"
        )));
    }

    let state_path = state_toml::state_path(&project_dir);
    let mut project_doc = state_toml::load_state_doc(&state_path)?;
    if state_toml::topic_exists(&project_doc, &slug) {
        return Err(DaedalusError::TopicNotFound(format!(
            "topic already registered: {slug}"
        )));
    }

    workspace_fs::ensure_dir(&topic_dir)?;
    let template_dir = options
        .repo_root
        .join("system")
        .join("templates")
        .join("repo-topic");
    let created_at = clock::now_local_timestamp();
    template_fs::copy_template_dir(
        &template_dir,
        &topic_dir,
        &[
            ("{{TASK_NAME}}", &state_toml::task_name(&project_doc)),
            ("{{CREATED_AT}}", &created_at),
            ("{{TOPIC_SLUG}}", &slug),
            ("{{TOPIC_TITLE}}", &title),
        ],
    )?;

    state_toml::append_project_topic(
        &mut project_doc,
        TopicSnapshot {
            slug: slug.clone(),
            title,
            lifecycle: TopicLifecycle::Planned.as_str().to_owned(),
            path: format!("topics/{slug}"),
            inherits: Vec::new(),
        },
    );
    state_toml::append_transition(
        &mut project_doc,
        Transition {
            stage: "project".to_owned(),
            action: "topic-new".to_owned(),
            timestamp: created_at,
            actor: options.actor,
            reason: format!("Create topic `{slug}`."),
            approval_source: None,
        },
    );
    state_toml::save_state_doc(&state_path, &project_doc)?;
    let _ = render_state(&project_dir)?;
    let state_md = render_state(&topic_dir)?.path;
    workspace_fs::rebuild_project_index(&options.repo_root)?;

    Ok(TopicOutput {
        project_dir,
        topic_dir,
        slug,
        action: "topic-new".to_owned(),
        state_md,
    })
}

/// 激活 topic。
pub fn activate_topic(options: ActivateTopicOptions) -> Result<TopicOutput> {
    let project_dir = options.project_dir;
    let topic_dir = workspace_fs::topic_dir_by_slug(&project_dir, &options.slug)?;
    let state_path = state_toml::state_path(&project_dir);
    let mut project_doc = state_toml::load_state_doc(&state_path)?;
    if !state_toml::topic_exists(&project_doc, &options.slug) {
        return Err(DaedalusError::TopicNotFound(options.slug));
    }
    state_toml::block_other_active_topics(&mut project_doc, &options.slug);
    state_toml::set_project_topic_lifecycle(
        &mut project_doc,
        &options.slug,
        TopicLifecycle::Active,
    )?;
    state_toml::set_task_lifecycle(&mut project_doc, TaskLifecycle::Active);
    state_toml::set_workspace_bucket(&mut project_doc, WorkspaceBucket::Projects);
    state_toml::set_active_topic(&mut project_doc, &options.slug);
    state_toml::append_transition(
        &mut project_doc,
        Transition {
            stage: "project".to_owned(),
            action: "topic-activate".to_owned(),
            timestamp: clock::now_local_timestamp(),
            actor: options.actor,
            reason: format!("Activate topic `{}`.", options.slug),
            approval_source: None,
        },
    );
    state_toml::save_state_doc(&state_path, &project_doc)?;

    let topic_state_path = state_toml::state_path(&topic_dir);
    let mut topic_doc = state_toml::load_state_doc(&topic_state_path)?;
    state_toml::set_topic_lifecycle(&mut topic_doc, TopicLifecycle::Active);
    state_toml::save_state_doc(&topic_state_path, &topic_doc)?;

    let _ = render_state(&project_dir)?;
    let state_md = render_state(&topic_dir)?.path;
    workspace_fs::sync_current_workspace(&options.repo_root, Some(&project_dir), Some(&topic_dir))?;
    sync_rust_analyzer_linked_projects(&options.repo_root)?;
    Ok(TopicOutput {
        project_dir,
        topic_dir,
        slug: options.slug,
        action: "topic-activate".to_owned(),
        state_md,
    })
}

/// 关闭 topic。
pub fn close_topic(options: CloseTopicOptions) -> Result<TopicOutput> {
    let reason = options.reason.trim();
    if reason.is_empty() {
        return Err(DaedalusError::TaskLifecycleReasonRequired);
    }
    let project_dir = options.project_dir;
    let topic_dir = workspace_fs::topic_dir_by_slug(&project_dir, &options.slug)?;
    let mut output_topic_dir = topic_dir.clone();
    match options.lifecycle {
        TopicLifecycle::Completed => validate_topic_completion(&topic_dir, &options.slug)?,
        TopicLifecycle::AwaitingReflection => {
            validate_topic_awaiting_reflection(&topic_dir, &options.slug)?
        }
        _ => {}
    }
    let project_state_path = state_toml::state_path(&project_dir);
    let mut project_doc = state_toml::load_state_doc(&project_state_path)?;
    if options.lifecycle == TopicLifecycle::AwaitingReflection
        && state_toml::awaiting_reflection_topic(&project_doc)
            .as_deref()
            .is_some_and(|slug| slug != options.slug)
    {
        return Err(DaedalusError::InvalidTopicLifecycleTransition(
            "another topic is already awaiting reflection".to_owned(),
        ));
    }
    state_toml::set_project_topic_lifecycle(&mut project_doc, &options.slug, options.lifecycle)?;
    if state_toml::active_topic(&project_doc).as_deref() == Some(&options.slug) {
        state_toml::set_active_topic(&mut project_doc, "");
    }
    state_toml::append_transition(
        &mut project_doc,
        Transition {
            stage: "project".to_owned(),
            action: match options.lifecycle {
                TopicLifecycle::Completed => "topic-complete",
                TopicLifecycle::AwaitingReflection => "topic-await-reflection",
                TopicLifecycle::Abandoned => "topic-abandon",
                _ => "topic-close",
            }
            .to_owned(),
            timestamp: clock::now_local_timestamp(),
            actor: options.actor.clone(),
            reason: reason.to_owned(),
            approval_source: None,
        },
    );
    state_toml::save_state_doc(&project_state_path, &project_doc)?;

    let topic_state_path = state_toml::state_path(&topic_dir);
    let mut topic_doc = state_toml::load_state_doc(&topic_state_path)?;
    state_toml::set_topic_lifecycle(&mut topic_doc, options.lifecycle);
    state_toml::append_transition(
        &mut topic_doc,
        Transition {
            stage: "topic".to_owned(),
            action: match options.lifecycle {
                TopicLifecycle::Completed => "topic-complete",
                TopicLifecycle::AwaitingReflection => "topic-await-reflection",
                TopicLifecycle::Abandoned => "topic-abandon",
                _ => "topic-close",
            }
            .to_owned(),
            timestamp: clock::now_local_timestamp(),
            actor: options.actor,
            reason: reason.to_owned(),
            approval_source: None,
        },
    );
    state_toml::save_state_doc(&topic_state_path, &topic_doc)?;
    let mut state_md = render_state(&topic_dir)?.path;

    if options.lifecycle == TopicLifecycle::Abandoned {
        output_topic_dir = archive_topic_dir(&project_dir, &options.slug)?;
        if output_topic_dir.exists() {
            return Err(DaedalusError::TaskMoveDestinationExists(output_topic_dir));
        }
        if let Some(parent) = output_topic_dir.parent() {
            workspace_fs::ensure_dir(parent)?;
        }
        fs::rename(&topic_dir, &output_topic_dir).map_err(|source| DaedalusError::Io {
            path: output_topic_dir.clone(),
            source,
        })?;
        let relative = output_topic_dir
            .strip_prefix(&project_dir)
            .unwrap_or(&output_topic_dir)
            .to_string_lossy()
            .to_string();
        state_toml::set_project_topic_path(&mut project_doc, &options.slug, &relative)?;
        state_toml::save_state_doc(&project_state_path, &project_doc)?;
        state_md = output_topic_dir.join(".daedalus").join("state.md");
    }

    if state_toml::active_topic_count(&project_doc) == 0 {
        state_toml::set_task_lifecycle(&mut project_doc, TaskLifecycle::Idle);
        state_toml::set_workspace_bucket(&mut project_doc, WorkspaceBucket::Projects);
        let next_action = if state_toml::awaiting_reflection_topic_count(&project_doc) > 0 {
            "当前没有 active topic，但存在 awaiting-reflection topic；可以用大块时间完成 closeout reflection，或用 `daedalus topic new` / `daedalus topic activate` 启动新专题。"
        } else {
            "当前没有 active topic；可以复盘 project，或用 `daedalus topic new` 启动新专题。"
        };
        state_toml::set_next_action(&mut project_doc, next_action);
        state_toml::save_state_doc(&project_state_path, &project_doc)?;
    }

    let _ = render_state(&project_dir)?;
    let current_topic = if state_toml::active_topic(&project_doc).is_some() {
        state_toml::active_topic(&project_doc)
            .and_then(|slug| state_toml::topic_path(&project_doc, &slug))
            .map(|path| project_dir.join(path))
    } else {
        None
    };
    workspace_fs::sync_current_workspace(
        &options.repo_root,
        Some(&project_dir),
        current_topic.as_deref(),
    )?;
    match options.lifecycle {
        TopicLifecycle::AwaitingReflection => workspace_fs::sync_closeout_workspace(
            &options.repo_root,
            Some(&project_dir),
            Some(&topic_dir),
        )?,
        TopicLifecycle::Completed | TopicLifecycle::Abandoned => {
            let pending = workspace_fs::pending_closeout_topic_dir(&options.repo_root)?;
            let topic_dir_for_compare = if topic_dir.is_absolute() {
                topic_dir.clone()
            } else {
                options.repo_root.join(&topic_dir)
            };
            if pending
                .as_deref()
                .is_some_and(|pending| pending == topic_dir_for_compare)
            {
                workspace_fs::sync_closeout_workspace(&options.repo_root, None, None)?;
            }
        }
        _ => {}
    }
    sync_rust_analyzer_linked_projects(&options.repo_root)?;
    let action = match options.lifecycle {
        TopicLifecycle::Completed => "topic-complete",
        TopicLifecycle::AwaitingReflection => "topic-await-reflection",
        TopicLifecycle::Abandoned => "topic-abandon",
        _ => "topic-close",
    };
    Ok(TopicOutput {
        project_dir,
        topic_dir: output_topic_dir,
        slug: options.slug,
        action: action.to_owned(),
        state_md,
    })
}

/// 校验 topic。
pub fn validate_topic(project_dir: PathBuf, slug: String) -> Result<Vec<String>> {
    let topic_dir = workspace_fs::topic_dir_by_slug(&project_dir, &slug)?;
    validate_topic_workspace(&topic_dir, &slug)
}

fn validate_topic_completion(topic_dir: &std::path::Path, slug: &str) -> Result<()> {
    let topic_doc = state_toml::load_state_doc(&state_toml::state_path(topic_dir))?;
    let unfinished: Vec<_> = state_toml::stages(&topic_doc)
        .into_iter()
        .filter(|stage| stage.status != "done")
        .map(|stage| format!("{} ({})", stage.id, stage.status))
        .collect();
    if !unfinished.is_empty() {
        return Err(DaedalusError::InvalidTopicLifecycleTransition(format!(
            "topic `{slug}` has unfinished stages: {}",
            unfinished.join(", ")
        )));
    }
    let issues = validate_topic_workspace(topic_dir, slug)?;
    if !issues.is_empty() {
        return Err(DaedalusError::WorkspaceValidationFailed(issues));
    }
    Ok(())
}

fn validate_topic_awaiting_reflection(topic_dir: &std::path::Path, slug: &str) -> Result<()> {
    let topic_doc = state_toml::load_state_doc(&state_toml::state_path(topic_dir))?;
    let unfinished: Vec<_> = state_toml::stages(&topic_doc)
        .into_iter()
        .filter(|stage| stage.id != "10-archivist" && stage.status != "done")
        .map(|stage| format!("{} ({})", stage.id, stage.status))
        .collect();
    if !unfinished.is_empty() {
        return Err(DaedalusError::InvalidTopicLifecycleTransition(format!(
            "topic `{slug}` cannot await reflection before core stages are done: {}",
            unfinished.join(", ")
        )));
    }
    let issues = validate_topic_workspace(topic_dir, slug)?;
    if !issues.is_empty() {
        return Err(DaedalusError::WorkspaceValidationFailed(issues));
    }
    Ok(())
}

fn sanitize_slug(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let sanitized = sanitized.trim_matches('-');
    if sanitized.is_empty() {
        "topic".to_owned()
    } else {
        sanitized.to_owned()
    }
}

fn archive_topic_dir(project_dir: &Path, slug: &str) -> Result<PathBuf> {
    Ok(project_dir.join(".archive").join("topics").join(slug))
}
