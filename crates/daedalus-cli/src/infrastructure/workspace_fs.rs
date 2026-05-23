use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::{DaedalusError, Result, WorkspaceBucket};
use crate::infrastructure::state_toml;

/// 从给定目录向上查找 daedalus 项目根目录。
///
/// 只有位于 daedalus 根目录或其任意子目录下时才返回成功；否则拒绝服务，
/// 避免 CLI 在无关目录中创建 `workspaces`。
pub fn repo_root_from(start: &Path) -> Result<PathBuf> {
    let mut current = start.canonicalize().map_err(|source| DaedalusError::Io {
        path: start.to_path_buf(),
        source,
    })?;

    loop {
        if is_daedalus_root(&current) {
            return Ok(current);
        }

        if !current.pop() {
            return Err(DaedalusError::NotDaedalusProject(start.to_path_buf()));
        }
    }
}

fn is_daedalus_root(path: &Path) -> bool {
    path.join("CLAUDE.md").is_file()
        && path.join("system").join("templates").join("repo").is_dir()
        && path.join("workspaces").join("02-learning").is_dir()
}

/// 返回 repo learning 任务所在的 workspace 根目录。
pub fn learning_root(repo_root: &Path) -> PathBuf {
    repo_root.join("workspaces").join("02-learning")
}

/// 返回已完成学习任务所在目录。
pub fn completed_root(repo_root: &Path) -> PathBuf {
    repo_root.join("workspaces").join("03-completed")
}

/// 返回已放弃学习任务所在目录。
pub fn abandoned_root(repo_root: &Path) -> PathBuf {
    repo_root.join("workspaces").join("04-abandoned")
}

/// 根据任务目录位置推导 workspace bucket。
pub fn bucket_from_task_dir(task_dir: &Path) -> Result<WorkspaceBucket> {
    let bucket = task_dir
        .parent()
        .and_then(Path::file_name)
        .and_then(|value| value.to_str())
        .and_then(WorkspaceBucket::parse)
        .ok_or_else(|| {
            DaedalusError::TaskLifecycleLocationMismatch(format!(
                "cannot infer workspace bucket from {}",
                task_dir.display()
            ))
        })?;
    Ok(bucket)
}

/// 扫描 `workspaces/02-learning` 下已有的学习任务目录。
pub fn active_tasks(learning_root: &Path) -> Result<Vec<PathBuf>> {
    if !learning_root.exists() {
        return Ok(Vec::new());
    }

    let mut tasks = Vec::new();
    let entries = fs::read_dir(learning_root).map_err(|source| DaedalusError::Io {
        path: learning_root.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| DaedalusError::Io {
            path: learning_root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() && path.join(".daedalus").join("state.toml").exists() {
            tasks.push(path);
        }
    }

    Ok(tasks)
}

/// 解析默认学习任务目录。
///
/// 如果调用方显式传入 `task_dir`，直接使用该目录；否则优先使用当前目录下的
/// `.daedalus`，再回退到 daedalus 项目中唯一的 learning task。
pub fn default_task_dir(explicit: Option<PathBuf>) -> Result<PathBuf> {
    default_project_dir(explicit)
}

/// 解析默认 repo learning project 目录。
pub fn default_project_dir(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return project_dir_from_any(&path);
    }

    let cwd = std::env::current_dir().map_err(|source| DaedalusError::Io {
        path: PathBuf::from("."),
        source,
    })?;

    if let Some(topic_dir) = enclosing_topic_dir(&cwd) {
        return project_dir_from_topic_dir(&topic_dir);
    }
    if let Some(project_dir) = enclosing_project_dir(&cwd) {
        return Ok(project_dir);
    }

    let repo_root = repo_root_from(&cwd)?;
    let tasks = active_tasks(&learning_root(&repo_root))?;
    match tasks.as_slice() {
        [task] => Ok(task.clone()),
        [] => Err(DaedalusError::NoActiveWorkspace),
        _ => Err(DaedalusError::MultipleActiveStages),
    }
}

/// 解析默认 topic 目录。
pub fn default_topic_dir(
    explicit_topic_dir: Option<PathBuf>,
    explicit_project_dir: Option<PathBuf>,
    explicit_topic: Option<String>,
) -> Result<PathBuf> {
    if let Some(path) = explicit_topic_dir {
        ensure_state_kind(&path, "topic")?;
        return Ok(path);
    }

    if let Some(slug) = explicit_topic {
        let project_dir = default_project_dir(explicit_project_dir)?;
        return topic_dir_by_slug(&project_dir, &slug);
    }

    let cwd = std::env::current_dir().map_err(|source| DaedalusError::Io {
        path: PathBuf::from("."),
        source,
    })?;
    if let Some(topic_dir) = enclosing_topic_dir(&cwd) {
        return Ok(topic_dir);
    }

    let project_dir = default_project_dir(explicit_project_dir)?;
    let doc = state_toml::load_state_doc(&state_toml::state_path(&project_dir))?;
    let slug = state_toml::active_topic(&doc).ok_or(DaedalusError::NoActiveTopic)?;
    topic_dir_by_slug(&project_dir, &slug)
}

/// 查找当前目录是否处于 topic 目录或其子目录。
pub fn enclosing_topic_dir(cwd: &Path) -> Option<PathBuf> {
    enclosing_state_dir(cwd, "topic")
}

/// 查找当前目录是否处于 project 目录或其子目录。
pub fn enclosing_project_dir(cwd: &Path) -> Option<PathBuf> {
    enclosing_state_dir(cwd, "project")
}

/// 根据 topic 目录推导 project 目录。
pub fn project_dir_from_topic_dir(topic_dir: &Path) -> Result<PathBuf> {
    topic_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            DaedalusError::TaskLifecycleLocationMismatch(format!(
                "cannot infer project dir from topic dir {}",
                topic_dir.display()
            ))
        })
}

/// 根据 project 和 topic slug 定位 topic 目录。
pub fn topic_dir_by_slug(project_dir: &Path, slug: &str) -> Result<PathBuf> {
    let doc = state_toml::load_state_doc(&state_toml::state_path(project_dir))?;
    if state_toml::state_kind(&doc) != "project" {
        return Err(DaedalusError::InvalidStateDocumentKind {
            expected: "project".to_owned(),
            actual: state_toml::state_kind(&doc).to_owned(),
        });
    }
    let path = state_toml::topic_path(&doc, slug)
        .ok_or_else(|| DaedalusError::TopicNotFound(slug.to_owned()))?;
    let topic_dir = project_dir.join(path);
    ensure_state_kind(&topic_dir, "topic")?;
    Ok(topic_dir)
}

fn project_dir_from_any(path: &Path) -> Result<PathBuf> {
    if is_state_kind(path, "project")? {
        return Ok(path.to_path_buf());
    }
    if is_state_kind(path, "topic")? {
        return project_dir_from_topic_dir(path);
    }
    Ok(path.to_path_buf())
}

fn enclosing_state_dir(cwd: &Path, kind: &str) -> Option<PathBuf> {
    let mut current = cwd.to_path_buf();
    loop {
        if is_state_kind(&current, kind).ok()? {
            return Some(current);
        }
        if current.file_name().and_then(|name| name.to_str()) == Some(".daedalus")
            && current.join("state.toml").exists()
            && let Some(parent) = current.parent()
            && is_state_kind(parent, kind).ok()?
        {
            return Some(parent.to_path_buf());
        }
        if !current.pop() {
            return None;
        }
    }
}

fn ensure_state_kind(dir: &Path, expected: &str) -> Result<()> {
    let doc = state_toml::load_state_doc(&state_toml::state_path(dir))?;
    let actual = state_toml::state_kind(&doc);
    if actual != expected {
        return Err(DaedalusError::InvalidStateDocumentKind {
            expected: expected.to_owned(),
            actual: actual.to_owned(),
        });
    }
    Ok(())
}

fn is_state_kind(dir: &Path, expected: &str) -> Result<bool> {
    let path = state_toml::state_path(dir);
    if !path.exists() {
        return Ok(false);
    }
    let doc = state_toml::load_state_doc(&path)?;
    Ok(state_toml::state_kind(&doc) == expected)
}

/// 确保目录存在。
pub fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| DaedalusError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// 将学习任务目录移动到目标 bucket，拒绝覆盖已有目录。
pub fn move_task(task_dir: &Path, destination_root: &Path) -> Result<PathBuf> {
    ensure_dir(destination_root)?;
    let task_name = task_dir
        .file_name()
        .map(ToOwned::to_owned)
        .ok_or(DaedalusError::NoActiveWorkspace)?;
    let destination = destination_root.join(task_name);
    if destination.exists() {
        return Err(DaedalusError::TaskMoveDestinationExists(destination));
    }
    fs::rename(task_dir, &destination).map_err(|source| DaedalusError::Io {
        path: destination.clone(),
        source,
    })?;
    Ok(destination)
}
