use std::fs;
use std::path::{Path, PathBuf};

use crate::application::knowledge::validate_project_knowledge;
use crate::application::project_navigation::validate_project_navigation;
use crate::application::review::validate_all_reviews;
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
pub fn validate_workspace(
    task_dir: &Path,
    repo_root: Option<&Path>,
    all_topics: bool,
    reviews: bool,
    knowledge: bool,
) -> Result<ValidationOutput> {
    let mut issues = Vec::new();
    let required = [
        "CLAUDE.md",
        ".daedalus/project-map.md",
        ".daedalus/topic-board.md",
        ".daedalus/state.toml",
        ".daedalus/state.md",
        ".daedalus/long-context.md",
        ".daedalus/artifact-index.md",
        ".daedalus/decision-log.md",
        ".daedalus/validation-log.md",
        "shared/README.md",
        "shared/evidence-registry.md",
        "shared/source-index.md",
        "topics/.gitkeep",
        "source/.gitignore",
        "source/pull_source.sh",
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

    if state_toml::state_kind(&doc) != "project" {
        issues.push(format!(
            "project state expected, got {}",
            state_toml::state_kind(&doc)
        ));
    }

    match state_toml::task_lifecycle(&doc) {
        Ok(TaskLifecycle::Completed) => {
            issues.push("legacy task lifecycle `completed`; use `idle`".to_owned())
        }
        Err(error) => issues.push(format!("invalid task lifecycle: {error}")),
        _ => {}
    }
    match state_toml::workspace_bucket(&doc) {
        Ok(WorkspaceBucket::Projects) => {}
        Ok(bucket) => issues.push(format!(
            "workspace_bucket should be `projects`, got `{}`",
            bucket.as_str()
        )),
        Err(error) => issues.push(format!("invalid workspace_bucket: {error}")),
    }
    if !task_dir
        .components()
        .any(|component| component.as_os_str() == "projects")
    {
        issues.push("project should live under workspaces/projects".to_owned());
    }

    let state_md = state_toml::state_md_path(task_dir);
    if stale(&state_path, &state_md)? {
        issues.push("state.md is stale or missing".to_owned());
    }
    issues.extend(validate_project_navigation(task_dir, &doc)?);

    let active_count = state_toml::active_topic_count(&doc);
    if active_count > 1 {
        issues.push("multiple active topics".to_owned());
    }
    let awaiting_count = state_toml::awaiting_reflection_topic_count(&doc);
    if awaiting_count > 1 {
        issues.push("multiple awaiting-reflection topics".to_owned());
    }

    let topics = state_toml::topics(&doc);
    let active_topic = state_toml::active_topic(&doc);
    if let Some(active_topic) = &active_topic {
        if !topics.iter().any(|topic| &topic.slug == active_topic) {
            issues.push(format!(
                "active_topic points to unknown topic: {active_topic}"
            ));
        }
    } else if active_count > 0 {
        issues.push("active_topic is missing".to_owned());
    }

    let topic_targets: Vec<_> = if all_topics {
        topics
            .into_iter()
            .filter(|topic| topic.lifecycle != "abandoned")
            .collect()
    } else {
        topics
            .into_iter()
            .filter(|topic| Some(topic.slug.as_str()) == active_topic.as_deref())
            .collect()
    };
    for topic in topic_targets {
        let topic_dir = task_dir.join(&topic.path);
        issues.extend(validate_topic_workspace(&topic_dir, &topic.slug)?);
    }

    if reviews {
        issues.extend(validate_all_reviews(task_dir)?);
    }

    if knowledge {
        if let Some(repo_root) = repo_root {
            issues.extend(validate_project_knowledge(task_dir, repo_root)?);
        } else {
            issues.push("knowledge validation requires repo_root".to_owned());
        }
    }

    if let Some(repo_root) = repo_root {
        let active_topic_path = active_topic
            .as_deref()
            .and_then(|slug| state_toml::topic_path(&doc, slug))
            .map(|path| task_dir.join(path));
        let awaiting_reflection_topic_path = state_toml::awaiting_reflection_topic(&doc)
            .and_then(|slug| state_toml::topic_path(&doc, &slug))
            .map(|path| task_dir.join(path));
        issues.extend(validate_current_projection(
            repo_root,
            task_dir,
            active_topic_path.as_deref(),
            awaiting_reflection_topic_path.as_deref(),
        )?);
        let active_projects = workspace_fs::active_projects(repo_root)?;
        if active_projects.len() > 1 {
            issues.push("WIP violation: more than one active project exists".to_owned());
        }
    }

    Ok(ValidationOutput {
        task_dir: task_dir.to_path_buf(),
        issues,
    })
}

fn validate_current_projection(
    repo_root: &Path,
    task_dir: &Path,
    active_topic: Option<&Path>,
    awaiting_reflection_topic: Option<&Path>,
) -> Result<Vec<String>> {
    let mut issues = Vec::new();
    validate_current_project_projection(&mut issues, repo_root, task_dir, active_topic.is_some())?;
    validate_projected_path(
        &mut issues,
        repo_root,
        task_dir,
        "current-topic",
        active_topic,
        workspace_fs::current_topic_dir(repo_root)?,
    )?;
    validate_projected_path(
        &mut issues,
        repo_root,
        task_dir,
        "closeout-topic",
        awaiting_reflection_topic,
        workspace_fs::pending_closeout_topic_dir(repo_root)?,
    )?;
    Ok(issues)
}

fn validate_current_project_projection(
    issues: &mut Vec<String>,
    repo_root: &Path,
    task_dir: &Path,
    has_active_topic: bool,
) -> Result<()> {
    let expected = has_active_topic.then_some(task_dir);
    let stored = workspace_fs::current_project_dir(repo_root)?;
    let link_target = projection_link_target(repo_root, "current-project")?;

    match (expected, stored.as_deref()) {
        (Some(_), None) => issues
            .push("current-project is missing from workspaces/.daedalus/current.toml".to_owned()),
        (Some(expected), Some(actual)) if !same_path(expected, actual)? => issues.push(
            "current-project in workspaces/.daedalus/current.toml points to the wrong project"
                .to_owned(),
        ),
        (None, Some(_)) => {}
        _ => {}
    }

    match (expected, link_target.as_deref()) {
        (Some(_), None) => issues.push("current-project symlink is missing".to_owned()),
        (Some(expected), Some(actual)) if !same_path(expected, actual)? => {
            issues.push("current-project symlink points to the wrong project".to_owned())
        }
        (None, Some(actual)) if path_is_inside(actual, task_dir) => issues.push(
            "current-project symlink points into this project but project has no active topic"
                .to_owned(),
        ),
        _ => {}
    }
    Ok(())
}

fn validate_projected_path(
    issues: &mut Vec<String>,
    repo_root: &Path,
    task_dir: &Path,
    projection_name: &str,
    expected: Option<&Path>,
    stored: Option<PathBuf>,
) -> Result<()> {
    let link_target = projection_link_target(repo_root, projection_name)?;
    match (expected, stored.as_deref()) {
        (Some(_), None) => issues.push(format!(
            "{projection_name} is missing from workspaces/.daedalus/current.toml"
        )),
        (Some(expected), Some(actual)) if !same_path(expected, actual)? => issues.push(format!(
            "{projection_name} in workspaces/.daedalus/current.toml points to the wrong topic"
        )),
        (None, Some(actual)) if path_is_inside(actual, task_dir) => issues.push(format!(
            "{projection_name} points into this project but project has no matching lifecycle topic"
        )),
        _ => {}
    }
    match (expected, link_target.as_deref()) {
        (Some(_), None) => issues.push(format!("{projection_name} symlink is missing")),
        (Some(expected), Some(actual)) if !same_path(expected, actual)? => {
            issues.push(format!("{projection_name} symlink points to the wrong topic"))
        }
        (None, Some(actual)) if path_is_inside(actual, task_dir) => issues.push(format!(
            "{projection_name} symlink points into this project but project has no matching lifecycle topic"
        )),
        _ => {}
    }
    Ok(())
}

fn projection_link_target(repo_root: &Path, name: &str) -> Result<Option<PathBuf>> {
    let link = workspace_fs::workspaces_root(repo_root).join(name);
    let Ok(metadata) = fs::symlink_metadata(&link) else {
        return Ok(None);
    };
    if metadata.file_type().is_symlink() {
        let target = fs::read_link(&link).map_err(|source| DaedalusError::Io {
            path: link.clone(),
            source,
        })?;
        let resolved = if target.is_absolute() {
            target
        } else {
            link.parent().unwrap_or(repo_root).join(target)
        };
        return Ok(Some(resolved.canonicalize().unwrap_or(resolved)));
    }
    if metadata.is_file() {
        let target = fs::read_to_string(&link).map_err(|source| DaedalusError::Io {
            path: link.clone(),
            source,
        })?;
        let target = PathBuf::from(target.trim());
        let resolved = if target.is_absolute() {
            target
        } else {
            link.parent().unwrap_or(repo_root).join(target)
        };
        return Ok(Some(resolved.canonicalize().unwrap_or(resolved)));
    }
    Ok(Some(link))
}

fn path_is_inside(path: &Path, parent: &Path) -> bool {
    match (path.canonicalize(), parent.canonicalize()) {
        (Ok(path), Ok(parent)) => path.starts_with(parent),
        _ => path.starts_with(parent),
    }
}

fn same_path(left: &Path, right: &Path) -> Result<bool> {
    let left = left.canonicalize().map_err(|source| DaedalusError::Io {
        path: left.to_path_buf(),
        source,
    })?;
    let right = right.canonicalize().map_err(|source| DaedalusError::Io {
        path: right.to_path_buf(),
        source,
    })?;
    Ok(left == right)
}

/// 校验 topic workspace。
pub fn validate_topic_workspace(topic_dir: &Path, slug: &str) -> Result<Vec<String>> {
    let mut issues = Vec::new();
    let required = [
        ".daedalus/task-card.md",
        ".daedalus/outcome-map.md",
        ".daedalus/state.toml",
        ".daedalus/state.md",
        ".daedalus/todo.md",
        ".daedalus/long-context.md",
        ".daedalus/artifact-index.md",
        ".daedalus/decision-log.md",
        ".daedalus/validation-log.md",
    ];
    for path in required {
        if !topic_dir.join(path).exists() {
            issues.push(format!("topic `{slug}` missing required file: {path}"));
        }
    }
    for path in ["demo", "guides", "notes"] {
        if !topic_dir.join(path).is_dir() {
            issues.push(format!("topic `{slug}` missing required directory: {path}"));
        }
    }
    issues.extend(validate_reflection_loop(topic_dir, slug));

    let state_path = state_toml::state_path(topic_dir);
    let doc = match state_toml::load_state_doc(&state_path) {
        Ok(doc) => doc,
        Err(error) => {
            issues.push(format!(
                "topic `{slug}` state.toml cannot be parsed: {error}"
            ));
            return Ok(issues);
        }
    };
    if state_toml::state_kind(&doc) != "topic" {
        issues.push(format!(
            "topic `{slug}` state expected, got {}",
            state_toml::state_kind(&doc)
        ));
    }
    if state_toml::topic_slug(&doc) != slug {
        issues.push(format!(
            "topic slug mismatch: project references `{slug}`, topic state says `{}`",
            state_toml::topic_slug(&doc)
        ));
    }
    if state_toml::active_stage_count(&doc) > 1 {
        issues.push(format!("topic `{slug}` has multiple active stages"));
    }
    if let Some(current_phase) = state_toml::current_phase(&doc) {
        if !state_toml::stage_exists(&doc, &current_phase) {
            issues.push(format!(
                "topic `{slug}` current_phase points to unknown stage: {current_phase}"
            ));
        }
    } else {
        issues.push(format!("topic `{slug}` current_phase is missing"));
    }

    let state_md = state_toml::state_md_path(topic_dir);
    if stale(&state_path, &state_md)? {
        issues.push(format!("topic `{slug}` state.md is stale or missing"));
    }

    for stage in state_toml::stages(&doc) {
        if stage.status == "done" {
            for artifact in stage.required_artifacts {
                if !topic_dir.join(&artifact).exists() {
                    issues.push(format!(
                        "topic `{slug}` completed stage `{}` missing artifact: {artifact}",
                        stage.id
                    ));
                }
            }
        }
    }
    Ok(issues)
}

fn validate_reflection_loop(topic_dir: &Path, slug: &str) -> Vec<String> {
    let mut issues = Vec::new();
    for path in [
        "reflection/README.md",
        "reflection/closeout.md",
        "reflection/candidate-map.md",
    ] {
        if !topic_dir.join(path).exists() {
            issues.push(format!(
                "topic `{slug}` missing reflection loop artifact: {path}"
            ));
        }
    }
    issues
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
