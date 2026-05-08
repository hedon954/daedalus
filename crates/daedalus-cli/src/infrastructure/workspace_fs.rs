use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::{DaedalusError, Result};

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
    if let Some(path) = explicit {
        return Ok(path);
    }

    let cwd = std::env::current_dir().map_err(|source| DaedalusError::Io {
        path: PathBuf::from("."),
        source,
    })?;

    if cwd.join(".daedalus").exists() {
        return Ok(cwd);
    }

    let repo_root = repo_root_from(&cwd)?;
    let tasks = active_tasks(&learning_root(&repo_root))?;
    match tasks.as_slice() {
        [task] => Ok(task.clone()),
        [] => Err(DaedalusError::NoActiveWorkspace),
        _ => Err(DaedalusError::MultipleActiveStages),
    }
}

/// 确保目录存在。
pub fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| DaedalusError::Io {
        path: path.to_path_buf(),
        source,
    })
}
