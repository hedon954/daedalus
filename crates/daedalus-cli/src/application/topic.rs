use std::path::PathBuf;

use crate::application::render::render_state;
use crate::application::validate_workspace::validate_topic_workspace;
use crate::domain::transition::Transition;
use crate::domain::{DaedalusError, Result, TopicLifecycle, TopicSnapshot};
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
    pub project_dir: PathBuf,
    pub slug: String,
    pub actor: String,
}

/// 关闭 topic 参数。
#[derive(Debug, Clone)]
pub struct CloseTopicOptions {
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
    if options.lifecycle == TopicLifecycle::Completed {
        validate_topic_completion(&topic_dir, &options.slug)?;
    }
    let project_state_path = state_toml::state_path(&project_dir);
    let mut project_doc = state_toml::load_state_doc(&project_state_path)?;
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
    let _ = render_state(&project_dir)?;
    let state_md = render_state(&topic_dir)?.path;
    Ok(TopicOutput {
        project_dir,
        topic_dir,
        slug: options.slug,
        action: "topic-close".to_owned(),
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
