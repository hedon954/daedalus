//! IDE 投影同步。
//!
//! daedalus 的学习状态以 filesystem artifacts 为准，IDE 配置只是便于开发的
//! 派生视图。本模块负责把当前 repo 中可用的 Rust workspace / demo manifest
//! 同步到 VSCode rust-analyzer 配置。

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde_json::{Value, json};
use walkdir::WalkDir;

use crate::domain::{DaedalusError, Result};

/// rust-analyzer linkedProjects 同步结果。
#[derive(Debug, Clone)]
pub struct RustAnalyzerSyncOutput {
    /// 被写入的 VSCode settings 文件。
    pub settings_path: PathBuf,
    /// 最终写入的 linkedProjects。
    pub linked_projects: Vec<String>,
}

/// 同步 `.vscode/settings.json` 中的 `rust-analyzer.linkedProjects`。
pub fn sync_rust_analyzer_linked_projects(repo_root: &Path) -> Result<RustAnalyzerSyncOutput> {
    let settings_dir = repo_root.join(".vscode");
    fs::create_dir_all(&settings_dir).map_err(|source| DaedalusError::Io {
        path: settings_dir.clone(),
        source,
    })?;

    let settings_path = settings_dir.join("settings.json");
    let mut settings = load_settings_json(&settings_path)?;
    let linked_projects = discover_linked_projects(repo_root);

    let object = settings.as_object_mut().ok_or_else(|| {
        DaedalusError::InvalidIdeOperation(format!(
            "{} must contain a JSON object",
            settings_path.display()
        ))
    })?;
    object.insert(
        "rust-analyzer.linkedProjects".to_owned(),
        json!(linked_projects),
    );

    let content = format!(
        "{}\n",
        serde_json::to_string_pretty(&settings).map_err(|source| DaedalusError::Json {
            path: settings_path.clone(),
            source,
        })?
    );
    fs::write(&settings_path, content).map_err(|source| DaedalusError::Io {
        path: settings_path.clone(),
        source,
    })?;

    Ok(RustAnalyzerSyncOutput {
        settings_path,
        linked_projects,
    })
}

fn load_settings_json(path: &Path) -> Result<Value> {
    if !path.exists() {
        return Ok(json!({}));
    }

    let content = fs::read_to_string(path).map_err(|source| DaedalusError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&content).map_err(|source| DaedalusError::Json {
        path: path.to_path_buf(),
        source,
    })
}

fn discover_linked_projects(repo_root: &Path) -> Vec<String> {
    let mut projects = BTreeSet::new();

    let crates_manifest = repo_root.join("crates/Cargo.toml");
    if crates_manifest.is_file() {
        projects.insert("crates/Cargo.toml".to_owned());
    }

    let workspace_root = repo_root.join("workspaces");
    if workspace_root.exists() {
        for entry in WalkDir::new(&workspace_root)
            .follow_links(false)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            if entry.file_type().is_file()
                && path.file_name().and_then(|value| value.to_str()) == Some("Cargo.toml")
                && is_topic_demo_manifest(repo_root, path)
                && let Some(relative) = relative_unix_path(repo_root, path)
            {
                projects.insert(relative);
            }
        }
    }

    projects.into_iter().collect()
}

fn is_topic_demo_manifest(repo_root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(repo_root) else {
        return false;
    };
    let components: Vec<String> = relative
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str().map(ToOwned::to_owned),
            _ => None,
        })
        .collect();

    matches!(
        components.as_slice(),
        [workspaces, bucket, _project, topics, _topic, demo, cargo]
            if workspaces == "workspaces"
                && bucket == "projects"
                && topics == "topics"
                && demo == "demo"
                && cargo == "Cargo.toml"
    )
}

fn relative_unix_path(repo_root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(repo_root).ok().map(|relative| {
        relative
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/")
    })
}
