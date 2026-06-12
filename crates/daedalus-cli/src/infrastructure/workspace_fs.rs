use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::{DaedalusError, Result, WorkspaceBucket};
use crate::infrastructure::state_toml;
use serde_json::{Value, json};
use toml_edit::{DocumentMut, Item, value};

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
        && path.join("workspaces").is_dir()
}

/// 返回 daedalus workspace 根目录。
pub fn workspaces_root(repo_root: &Path) -> PathBuf {
    repo_root.join("workspaces")
}

/// 返回稳定学习 project 根目录。
pub fn projects_root(repo_root: &Path) -> PathBuf {
    workspaces_root(repo_root).join("projects")
}

/// 返回全局 backlog 根目录。
pub fn backlog_root(repo_root: &Path) -> PathBuf {
    workspaces_root(repo_root).join("backlog")
}

/// 返回 workspace 级 daedalus 状态目录。
pub fn workspace_state_root(repo_root: &Path) -> PathBuf {
    workspaces_root(repo_root).join(".daedalus")
}

/// 当前学习现场指针文件。
pub fn current_toml_path(repo_root: &Path) -> PathBuf {
    workspace_state_root(repo_root).join("current.toml")
}

/// 项目索引文件。
pub fn project_index_path(repo_root: &Path) -> PathBuf {
    workspace_state_root(repo_root).join("project-index.toml")
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

/// 扫描 `workspaces/projects` 下的 project 目录。
pub fn project_dirs(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let root = projects_root(repo_root);
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut projects = Vec::new();
    let entries = fs::read_dir(&root).map_err(|source| DaedalusError::Io {
        path: root.clone(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| DaedalusError::Io {
            path: root.clone(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() && path.join(".daedalus").join("state.toml").exists() {
            projects.push(path);
        }
    }
    Ok(projects)
}

/// 扫描稳定工作区中的 active projects。
pub fn active_projects(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let mut active = Vec::new();
    for project_dir in project_dirs(repo_root)? {
        let doc = state_toml::load_state_doc(&state_toml::state_path(&project_dir))?;
        if state_toml::task_lifecycle(&doc).is_ok_and(|lifecycle| lifecycle.as_str() == "active") {
            active.push(project_dir);
        }
    }
    Ok(active)
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
    if let Some(current) = current_project_dir(&repo_root)? {
        return Ok(current);
    }
    let projects = active_projects(&repo_root)?;
    match projects.as_slice() {
        [project] => Ok(project.clone()),
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

/// 确保新的稳定 workspace 目录和 ignore 规则存在。
pub fn ensure_stable_workspace_layout(repo_root: &Path) -> Result<()> {
    ensure_dir(&projects_root(repo_root))?;
    ensure_dir(&backlog_root(repo_root))?;
    ensure_dir(&workspace_state_root(repo_root))?;
    write_archive_ignore_files(repo_root)?;
    Ok(())
}

/// 同步当前学习现场指针和可点击软链接。
pub fn sync_current_workspace(
    repo_root: &Path,
    project_dir: Option<&Path>,
    topic_dir: Option<&Path>,
) -> Result<()> {
    ensure_stable_workspace_layout(repo_root)?;
    let path = current_toml_path(repo_root);
    let project_dir = project_dir.map(|path| normalize_repo_path(repo_root, path));
    let topic_dir = topic_dir.map(|path| normalize_repo_path(repo_root, path));
    let pending_closeout_project = current_path(repo_root, "pending_closeout_project")?;
    let pending_closeout_topic = current_path(repo_root, "pending_closeout_topic")?;
    let mut doc = DocumentMut::new();
    doc["schema_version"] = value(1);
    doc["current_project"] = value(
        project_dir
            .as_deref()
            .and_then(|path| path.strip_prefix(workspaces_root(repo_root)).ok())
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default(),
    );
    doc["current_topic"] = value(
        topic_dir
            .as_deref()
            .and_then(|path| path.strip_prefix(workspaces_root(repo_root)).ok())
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default(),
    );
    doc["pending_closeout_project"] = value(
        pending_closeout_project
            .as_deref()
            .and_then(|path| path.strip_prefix(workspaces_root(repo_root)).ok())
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default(),
    );
    doc["pending_closeout_topic"] = value(
        pending_closeout_topic
            .as_deref()
            .and_then(|path| path.strip_prefix(workspaces_root(repo_root)).ok())
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default(),
    );
    fs::write(&path, doc.to_string()).map_err(|source| DaedalusError::Io {
        path: path.clone(),
        source,
    })?;

    sync_symlink(
        &workspaces_root(repo_root).join("current-project"),
        project_dir.as_deref(),
    )?;
    sync_symlink(
        &workspaces_root(repo_root).join("current-topic"),
        topic_dir.as_deref(),
    )?;
    sync_symlink(
        &workspaces_root(repo_root).join("closeout-project"),
        pending_closeout_project.as_deref(),
    )?;
    sync_symlink(
        &workspaces_root(repo_root).join("closeout-topic"),
        pending_closeout_topic.as_deref(),
    )?;
    rebuild_project_index(repo_root)
}

/// 同步等待 closeout 的学习现场指针和可点击软链接。
pub fn sync_closeout_workspace(
    repo_root: &Path,
    project_dir: Option<&Path>,
    topic_dir: Option<&Path>,
) -> Result<()> {
    ensure_stable_workspace_layout(repo_root)?;
    let path = current_toml_path(repo_root);
    let current_project = current_project_dir(repo_root)?;
    let current_topic = current_topic_dir(repo_root)?;
    let project_dir = project_dir.map(|path| normalize_repo_path(repo_root, path));
    let topic_dir = topic_dir.map(|path| normalize_repo_path(repo_root, path));
    let mut doc = DocumentMut::new();
    doc["schema_version"] = value(1);
    doc["current_project"] = value(relative_workspace_value(
        repo_root,
        current_project.as_deref(),
    ));
    doc["current_topic"] = value(relative_workspace_value(
        repo_root,
        current_topic.as_deref(),
    ));
    doc["pending_closeout_project"] =
        value(relative_workspace_value(repo_root, project_dir.as_deref()));
    doc["pending_closeout_topic"] =
        value(relative_workspace_value(repo_root, topic_dir.as_deref()));
    fs::write(&path, doc.to_string()).map_err(|source| DaedalusError::Io {
        path: path.clone(),
        source,
    })?;

    sync_symlink(
        &workspaces_root(repo_root).join("current-project"),
        current_project.as_deref(),
    )?;
    sync_symlink(
        &workspaces_root(repo_root).join("current-topic"),
        current_topic.as_deref(),
    )?;
    sync_symlink(
        &workspaces_root(repo_root).join("closeout-project"),
        project_dir.as_deref(),
    )?;
    sync_symlink(
        &workspaces_root(repo_root).join("closeout-topic"),
        topic_dir.as_deref(),
    )?;
    rebuild_project_index(repo_root)
}

/// 从 current.toml 读取当前 project。
pub fn current_project_dir(repo_root: &Path) -> Result<Option<PathBuf>> {
    current_path(repo_root, "current_project")
}

/// 从 current.toml 读取当前 topic。
pub fn current_topic_dir(repo_root: &Path) -> Result<Option<PathBuf>> {
    current_path(repo_root, "current_topic")
}

/// 从 current.toml 读取等待 closeout 的 project。
pub fn pending_closeout_project_dir(repo_root: &Path) -> Result<Option<PathBuf>> {
    current_path(repo_root, "pending_closeout_project")
}

/// 从 current.toml 读取等待 closeout 的 topic。
pub fn pending_closeout_topic_dir(repo_root: &Path) -> Result<Option<PathBuf>> {
    current_path(repo_root, "pending_closeout_topic")
}

fn current_path(repo_root: &Path, key: &str) -> Result<Option<PathBuf>> {
    let path = current_toml_path(repo_root);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path).map_err(|source| DaedalusError::Io {
        path: path.clone(),
        source,
    })?;
    let doc = content
        .parse::<DocumentMut>()
        .map_err(|source| DaedalusError::Toml {
            path: path.clone(),
            source,
        })?;
    let Some(value) = doc
        .get(key)
        .and_then(Item::as_str)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };
    let resolved = workspaces_root(repo_root).join(value);
    if resolved.exists() {
        Ok(Some(resolved))
    } else {
        Ok(None)
    }
}

pub fn rebuild_project_index(repo_root: &Path) -> Result<()> {
    let path = project_index_path(repo_root);
    let mut doc = DocumentMut::new();
    doc["schema_version"] = value(1);
    doc["projects"] = Item::ArrayOfTables(Default::default());
    if let Some(array) = doc["projects"].as_array_of_tables_mut() {
        for project_dir in project_dirs(repo_root)? {
            let state = state_toml::load_state_doc(&state_toml::state_path(&project_dir))?;
            let mut table = toml_edit::Table::new();
            let slug = project_dir
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("unknown");
            table["slug"] = value(slug);
            table["name"] = value(state_toml::task_name(&state));
            table["lifecycle"] = value(
                state_toml::task_lifecycle(&state)
                    .map(|lifecycle| lifecycle.as_str().to_owned())
                    .unwrap_or_else(|_| "unknown".to_owned()),
            );
            table["path"] = value(
                project_dir
                    .strip_prefix(workspaces_root(repo_root))
                    .unwrap_or(&project_dir)
                    .to_string_lossy()
                    .to_string(),
            );
            table["active_topic"] = value(state_toml::active_topic(&state).unwrap_or_default());
            array.push(table);
        }
    }
    fs::write(&path, doc.to_string()).map_err(|source| DaedalusError::Io { path, source })
}

fn write_archive_ignore_files(repo_root: &Path) -> Result<()> {
    const FILES: [&str; 9] = [
        ".ignore",
        ".cursorignore",
        ".clineignore",
        ".rooignore",
        ".aiderignore",
        ".continueignore",
        ".geminiignore",
        ".codeiumignore",
        ".augmentignore",
    ];
    const CONTENT: &str = "# daedalus: archived projects/topics should stay out of default AI/search context\nworkspaces/projects/**/.archive/\n";
    for file in FILES {
        let path = repo_root.join(file);
        if !path.exists() {
            fs::write(&path, CONTENT).map_err(|source| DaedalusError::Io { path, source })?;
        }
    }
    write_claude_archive_deny_settings(repo_root)?;
    Ok(())
}

fn write_claude_archive_deny_settings(repo_root: &Path) -> Result<()> {
    let claude_dir = repo_root.join(".claude");
    ensure_dir(&claude_dir)?;
    let path = claude_dir.join("settings.json");
    let mut settings = if path.exists() {
        let content = fs::read_to_string(&path).map_err(|source| DaedalusError::Io {
            path: path.clone(),
            source,
        })?;
        serde_json::from_str::<Value>(&content).map_err(|source| DaedalusError::Json {
            path: path.clone(),
            source,
        })?
    } else {
        json!({})
    };

    let settings_object = settings.as_object_mut().ok_or_else(|| {
        DaedalusError::InvalidIdeOperation(".claude/settings.json must be a JSON object".to_owned())
    })?;
    let permissions = settings_object
        .entry("permissions")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| {
            DaedalusError::InvalidIdeOperation(
                ".claude/settings.json permissions must be a JSON object".to_owned(),
            )
        })?;
    let deny = permissions
        .entry("deny")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .ok_or_else(|| {
            DaedalusError::InvalidIdeOperation(
                ".claude/settings.json permissions.deny must be a JSON array".to_owned(),
            )
        })?;
    for rule in [
        "Read(./workspaces/projects/**/.archive/**)",
        "Edit(./workspaces/projects/**/.archive/**)",
        "Write(./workspaces/projects/**/.archive/**)",
    ] {
        if !deny.iter().any(|item| item.as_str() == Some(rule)) {
            deny.push(json!(rule));
        }
    }

    let content =
        serde_json::to_string_pretty(&settings).map_err(|source| DaedalusError::Json {
            path: path.clone(),
            source,
        })?;
    fs::write(&path, format!("{content}\n")).map_err(|source| DaedalusError::Io { path, source })
}

fn sync_symlink(link: &Path, target: Option<&Path>) -> Result<()> {
    if link.exists() || fs::symlink_metadata(link).is_ok() {
        if link.is_dir()
            && !fs::symlink_metadata(link).is_ok_and(|meta| meta.file_type().is_symlink())
        {
            return Err(DaedalusError::TaskLifecycleLocationMismatch(format!(
                "cannot replace non-symlink directory {}",
                link.display()
            )));
        }
        fs::remove_file(link).map_err(|source| DaedalusError::Io {
            path: link.to_path_buf(),
            source,
        })?;
    }
    let Some(target) = target else {
        return Ok(());
    };
    let symlink_target = link
        .parent()
        .and_then(|parent| target.strip_prefix(parent).ok())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| target.to_path_buf());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&symlink_target, link).map_err(|source| DaedalusError::Io {
            path: link.to_path_buf(),
            source,
        })?;
    }
    #[cfg(not(unix))]
    {
        fs::write(link, symlink_target.to_string_lossy().as_ref()).map_err(|source| {
            DaedalusError::Io {
                path: link.to_path_buf(),
                source,
            }
        })?;
    }
    Ok(())
}

fn normalize_repo_path(repo_root: &Path, path: &Path) -> PathBuf {
    let normalized = if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    };
    normalized.canonicalize().unwrap_or(normalized)
}

fn relative_workspace_value(repo_root: &Path, path: Option<&Path>) -> String {
    path.and_then(|path| path.strip_prefix(workspaces_root(repo_root)).ok())
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_default()
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
