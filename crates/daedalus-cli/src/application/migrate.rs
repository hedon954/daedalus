use std::fs;
use std::path::{Path, PathBuf};

use toml_edit::{DocumentMut, Item, Table, value};

use crate::application::render::render_state;
use crate::domain::{DaedalusError, Result};
use crate::infrastructure::{clock, state_toml, template_fs, workspace_fs};

/// repo learning workspace 迁移参数。
#[derive(Debug, Clone)]
pub struct MigrateRepoLearningOptions {
    pub repo_root: PathBuf,
    pub task_dir: PathBuf,
    pub topic_slug: String,
    pub topic_title: String,
    pub execute: bool,
}

/// 迁移输出。
#[derive(Debug, Clone)]
pub struct MigrateRepoLearningOutput {
    pub project_dir: PathBuf,
    pub topic_dir: PathBuf,
    pub backup_dir: Option<PathBuf>,
    pub execute: bool,
    pub state_md: Option<PathBuf>,
    pub topic_state_md: Option<PathBuf>,
    pub planned_actions: Vec<String>,
}

/// 将旧 single-topic workspace 一次性迁移为 project/topic 新结构。
pub fn migrate_repo_learning(
    options: MigrateRepoLearningOptions,
) -> Result<MigrateRepoLearningOutput> {
    let project_dir = options.task_dir;
    let topic_slug = sanitize_slug(&options.topic_slug);
    let topic_title = if options.topic_title.trim().is_empty() {
        topic_slug.clone()
    } else {
        options.topic_title.trim().to_owned()
    };
    let topic_dir = project_dir.join("topics").join(&topic_slug);

    let mut planned_actions = vec![
        format!("create topic dir {}", topic_dir.display()),
        "move notes/ guides/ demo/ into topic".to_owned(),
        "move topic-level .daedalus markdown files into topic".to_owned(),
        "rewrite project .daedalus/state.toml".to_owned(),
        "render project and topic state.md".to_owned(),
    ];

    validate_old_workspace(&project_dir)?;
    if !options.execute {
        planned_actions.push("dry-run only; pass --execute to apply".to_owned());
        return Ok(MigrateRepoLearningOutput {
            project_dir,
            topic_dir,
            backup_dir: None,
            execute: false,
            state_md: None,
            topic_state_md: None,
            planned_actions,
        });
    }

    let old_state_path = state_toml::state_path(&project_dir);
    let old_doc = state_toml::load_state_doc(&old_state_path)?;
    let task_name = state_toml::task_name(&old_doc);
    let migrated_at = clock::now_local_timestamp();
    let task_created_at = old_doc
        .get("task")
        .and_then(|task| task.get("created_at"))
        .and_then(Item::as_str)
        .unwrap_or(&migrated_at)
        .to_owned();
    let backup_dir = project_dir
        .join(".daedalus-migration-backup")
        .join(migrated_at.replace([':', ' '], "-"));
    workspace_fs::ensure_dir(&backup_dir)?;
    backup_if_exists(&project_dir, &backup_dir, ".daedalus")?;
    backup_if_exists(&project_dir, &backup_dir, "notes")?;
    backup_if_exists(&project_dir, &backup_dir, "guides")?;
    backup_if_exists(&project_dir, &backup_dir, "demo")?;
    let preserved_source_files = preserve_existing_files(
        &project_dir,
        &[
            "source/README.md",
            "source/pull_source.sh",
            "source/.gitignore",
        ],
    )?;

    workspace_fs::ensure_dir(&topic_dir)?;
    move_dir_if_exists(&project_dir, &topic_dir, "notes")?;
    move_dir_if_exists(&project_dir, &topic_dir, "guides")?;
    move_dir_if_exists(&project_dir, &topic_dir, "demo")?;
    workspace_fs::ensure_dir(&topic_dir.join(".daedalus"))?;
    for file in [
        "task-card.md",
        "outcome-map.md",
        "todo.md",
        "artifact-index.md",
        "decision-log.md",
        "validation-log.md",
        "long-context.md",
    ] {
        move_file_if_exists(
            &project_dir.join(".daedalus").join(file),
            &topic_dir.join(".daedalus").join(file),
        )?;
    }

    let topic_doc = build_topic_doc(old_doc, &topic_slug, &topic_title);
    state_toml::save_state_doc(&state_toml::state_path(&topic_dir), &topic_doc)?;

    let project_template = options
        .repo_root
        .join("system")
        .join("templates")
        .join("repo");
    template_fs::copy_template_dir(
        &project_template,
        &project_dir,
        &[
            ("{{TASK_NAME}}", &task_name),
            ("{{CREATED_AT}}", &task_created_at),
            ("{{TOPIC_SLUG}}", &topic_slug),
            ("{{TOPIC_TITLE}}", &topic_title),
        ],
    )?;
    restore_preserved_files(&project_dir, preserved_source_files)?;

    let mut project_doc = state_toml::load_state_doc(&state_toml::state_path(&project_dir))?;
    let topic_next_action = state_toml::next_action(&topic_doc);
    state_toml::set_next_action(
        &mut project_doc,
        &format!("继续 active topic `{topic_slug}`：{topic_next_action}"),
    );
    state_toml::append_transition(
        &mut project_doc,
        crate::domain::transition::Transition {
            stage: "project".to_owned(),
            action: "migrate".to_owned(),
            timestamp: migrated_at,
            actor: "daedalus-cli".to_owned(),
            reason: format!(
                "从 single-topic workspace 迁移为 multi-topic project，初始 topic 为 `{topic_slug}`。"
            ),
            approval_source: None,
        },
    );
    state_toml::save_state_doc(&state_toml::state_path(&project_dir), &project_doc)?;

    let state_md = render_state(&project_dir)?.path;
    let topic_state_md = render_state(&topic_dir)?.path;
    Ok(MigrateRepoLearningOutput {
        project_dir,
        topic_dir,
        backup_dir: Some(backup_dir),
        execute: true,
        state_md: Some(state_md),
        topic_state_md: Some(topic_state_md),
        planned_actions,
    })
}

fn validate_old_workspace(project_dir: &Path) -> Result<()> {
    let doc = state_toml::load_state_doc(&state_toml::state_path(project_dir))?;
    if state_toml::state_kind(&doc) == "project" || project_dir.join("topics").exists() {
        return Err(DaedalusError::InvalidStateDocumentKind {
            expected: "legacy-task".to_owned(),
            actual: "project-or-topic-layout".to_owned(),
        });
    }
    Ok(())
}

fn build_topic_doc(mut old_doc: DocumentMut, slug: &str, title: &str) -> DocumentMut {
    let current_phase = state_toml::current_phase(&old_doc).unwrap_or_else(|| {
        state_toml::stages(&old_doc)
            .first()
            .map(|stage| stage.id.clone())
            .unwrap_or_else(|| "01-goal-aligner".to_owned())
    });
    let next_action = state_toml::next_action(&old_doc);
    old_doc.as_table_mut().remove("task");
    let mut topic = Table::new();
    topic["slug"] = value(slug);
    topic["title"] = value(title);
    topic["kind"] = value("repo-learning-topic");
    topic["parent_project"] = value("../..");
    topic["lifecycle"] = value("active");
    topic["current_phase"] = value(current_phase);
    topic["next_action"] = value(next_action);
    old_doc["topic"] = Item::Table(topic);
    old_doc
}

fn backup_if_exists(project_dir: &Path, backup_dir: &Path, relative: &str) -> Result<()> {
    let source = project_dir.join(relative);
    if source.exists() {
        copy_dir(&source, &backup_dir.join(relative))?;
    }
    Ok(())
}

fn preserve_existing_files(
    project_dir: &Path,
    relatives: &[&str],
) -> Result<Vec<(PathBuf, String)>> {
    let mut preserved = Vec::new();
    for relative in relatives {
        let path = project_dir.join(relative);
        if path.exists() {
            let content = fs::read_to_string(&path).map_err(|source| DaedalusError::Io {
                path: path.clone(),
                source,
            })?;
            preserved.push((PathBuf::from(relative), content));
        }
    }
    Ok(preserved)
}

fn restore_preserved_files(project_dir: &Path, preserved: Vec<(PathBuf, String)>) -> Result<()> {
    for (relative, content) in preserved {
        let path = project_dir.join(relative);
        if let Some(parent) = path.parent() {
            workspace_fs::ensure_dir(parent)?;
        }
        fs::write(&path, content).map_err(|source| DaedalusError::Io { path, source })?;
    }
    Ok(())
}

fn copy_dir(source: &Path, target: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(source) {
        let entry = entry.map_err(|source_err| DaedalusError::Io {
            path: source.to_path_buf(),
            source: std::io::Error::other(source_err),
        })?;
        let relative = entry.path().strip_prefix(source).unwrap_or(entry.path());
        let target_path = target.join(relative);
        if entry.file_type().is_dir() {
            workspace_fs::ensure_dir(&target_path)?;
        } else {
            if let Some(parent) = target_path.parent() {
                workspace_fs::ensure_dir(parent)?;
            }
            fs::copy(entry.path(), &target_path).map_err(|source_err| DaedalusError::Io {
                path: target_path,
                source: source_err,
            })?;
        }
    }
    Ok(())
}

fn move_dir_if_exists(project_dir: &Path, topic_dir: &Path, name: &str) -> Result<()> {
    let source = project_dir.join(name);
    if source.exists() {
        let target = topic_dir.join(name);
        if target.exists() {
            fs::remove_dir_all(&target).map_err(|source_err| DaedalusError::Io {
                path: target.clone(),
                source: source_err,
            })?;
        }
        fs::rename(&source, &target).map_err(|source_err| DaedalusError::Io {
            path: target,
            source: source_err,
        })?;
    }
    Ok(())
}

fn move_file_if_exists(source: &Path, target: &Path) -> Result<()> {
    if !source.exists() {
        return Ok(());
    }
    if let Some(parent) = target.parent() {
        workspace_fs::ensure_dir(parent)?;
    }
    if target.exists() {
        fs::remove_file(target).map_err(|source_err| DaedalusError::Io {
            path: target.to_path_buf(),
            source: source_err,
        })?;
    }
    fs::rename(source, target).map_err(|source_err| DaedalusError::Io {
        path: target.to_path_buf(),
        source: source_err,
    })
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
