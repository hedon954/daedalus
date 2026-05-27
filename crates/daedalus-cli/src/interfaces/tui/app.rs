use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::Result;
use crate::infrastructure::{state_toml, workspace_fs};

const WORKSPACE_BUCKETS: [&str; 3] = ["02-learning", "03-completed", "04-abandoned"];
const RECENT_TRANSITION_LIMIT: usize = 10;

/// TUI 当前页面。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiView {
    /// workspace 任务选择页。
    TaskSelector,
    /// 单个学习任务详情页。
    TaskOverview,
}

/// TUI 只读应用状态。
#[derive(Debug, Clone)]
pub struct TuiApp {
    /// 当前页面。
    pub view: TuiView,
    /// 选择页任务列表。
    pub tasks: Vec<TuiTaskSummary>,
    /// 当前选择的任务索引。
    pub selected_task: usize,
    /// 当前展示的任务详情。
    pub overview: Option<TuiOverview>,
    /// 是否允许从详情返回选择页。
    pub can_return_to_selector: bool,
}

impl TuiApp {
    /// 根据当前目录和可选显式任务目录创建 TUI 状态。
    pub fn from_launch_context(
        repo_root: &Path,
        cwd: &Path,
        explicit: Option<PathBuf>,
    ) -> Result<Self> {
        if let Some(task_dir) = explicit {
            let overview = load_overview(&task_dir)?;
            return Ok(Self::single_task(overview));
        }

        if let Some(task_dir) = enclosing_task_dir(cwd, repo_root) {
            let overview = load_overview(&task_dir)?;
            return Ok(Self::single_task(overview));
        }

        let tasks = scan_task_summaries(repo_root)?;
        Ok(Self {
            view: TuiView::TaskSelector,
            tasks,
            selected_task: 0,
            overview: None,
            can_return_to_selector: true,
        })
    }

    fn single_task(overview: TuiOverview) -> Self {
        Self {
            view: TuiView::TaskOverview,
            tasks: Vec::new(),
            selected_task: 0,
            overview: Some(overview),
            can_return_to_selector: false,
        }
    }

    /// 将选择游标向上移动一项。
    pub fn select_previous(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        self.selected_task = self.selected_task.saturating_sub(1);
    }

    /// 将选择游标向下移动一项。
    pub fn select_next(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        self.selected_task = (self.selected_task + 1).min(self.tasks.len() - 1);
    }

    /// 进入当前选择的任务详情页。
    pub fn open_selected_task(&mut self) -> Result<()> {
        if let Some(task) = self.tasks.get(self.selected_task) {
            self.overview = Some(load_overview(&task.path)?);
            self.view = TuiView::TaskOverview;
        }
        Ok(())
    }

    /// 从任务详情页返回选择页。
    pub fn return_to_selector(&mut self) {
        if self.can_return_to_selector {
            self.view = TuiView::TaskSelector;
            self.overview = None;
        }
    }
}

/// 选择页展示的学习任务摘要。
#[derive(Debug, Clone)]
pub struct TuiTaskSummary {
    /// 学习项目名称。
    pub task_name: String,
    /// 学习项目目录。
    pub path: PathBuf,
    /// 实际所在 workspace bucket。
    pub bucket: String,
    /// 生命周期。
    pub lifecycle: String,
    /// Active topic。
    pub active_topic: String,
    /// 当前 topic 阶段。
    pub current_phase: String,
    /// 当前阶段状态。
    pub current_status: String,
    /// 已完成阶段数量。
    pub done_stage_count: usize,
    /// 阶段总数。
    pub total_stage_count: usize,
}

/// TUI 只读总览所需的数据。
#[derive(Debug, Clone)]
pub struct TuiOverview {
    /// 学习项目名称。
    pub task_name: String,
    /// 学习项目目录。
    pub task_dir: PathBuf,
    /// Active topic slug。
    pub active_topic: String,
    /// Active topic title。
    pub active_topic_title: String,
    /// Active topic 目录。
    pub topic_dir: Option<PathBuf>,
    /// 任务生命周期状态。
    pub lifecycle: String,
    /// workspace bucket。
    pub workspace_bucket: String,
    /// 当前阶段 ID。
    pub current_phase: String,
    /// 当前阶段状态。
    pub current_status: String,
    /// 面向 Agent 的下一步动作。
    pub next_action: String,
    /// 已完成阶段数量。
    pub done_stage_count: usize,
    /// 阶段总数。
    pub total_stage_count: usize,
    /// 当前缺失的关键产物。
    pub missing_artifacts: Vec<String>,
    /// `todo.md` 中的待办摘要。
    pub todo_summary: Vec<String>,
    /// 最近几条状态流转摘要。
    pub recent_transitions: Vec<String>,
    /// Review 摘要。
    pub review_summary: Vec<String>,
    /// Knowledge 摘要。
    pub knowledge_summary: Vec<String>,
    /// 任务关闭信息摘要。
    pub closure_summary: Vec<String>,
}

/// 从学习任务目录加载 TUI 只读总览数据。
pub fn load_overview(task_dir: &Path) -> Result<TuiOverview> {
    let doc = state_toml::load_state_doc(&state_toml::state_path(task_dir))?;
    let (topic_dir, topic_doc) = active_topic_doc(task_dir, &doc)?;
    let active_topic = topic_doc
        .as_ref()
        .map(state_toml::topic_slug)
        .or_else(|| state_toml::active_topic(&doc))
        .unwrap_or_else(|| "none".to_owned());
    let active_topic_title = topic_doc
        .as_ref()
        .map(state_toml::topic_title)
        .unwrap_or_else(|| "none".to_owned());
    let progress_doc = topic_doc.as_ref().unwrap_or(&doc);
    let current_phase =
        state_toml::current_phase(progress_doc).unwrap_or_else(|| "unknown".to_owned());
    let stages = state_toml::stages(progress_doc);
    let current_status = stages
        .iter()
        .find(|stage| stage.id == current_phase)
        .map(|stage| stage.status.clone())
        .unwrap_or_else(|| "unknown".to_owned());
    let done_stage_count = stages.iter().filter(|stage| stage.status == "done").count();
    let total_stage_count = stages.len();
    let lifecycle = task_field(&doc, "lifecycle").unwrap_or_else(|| "unknown".to_owned());
    let workspace_bucket = actual_bucket(task_dir);

    let mut missing_artifacts = Vec::new();
    let artifact_root = topic_dir.as_deref().unwrap_or(task_dir);
    for stage in stages {
        for artifact in stage.required_artifacts {
            if !artifact_root.join(&artifact).exists() {
                missing_artifacts.push(format!("{artifact} ({})", stage.id));
            }
        }
    }

    let todo_summary = fs::read_to_string(artifact_root.join(".daedalus").join("todo.md"))
        .ok()
        .map(|content| {
            content
                .lines()
                .filter(|line| line.trim_start().starts_with("- [ ]"))
                .take(6)
                .map(|line| line.trim().trim_start_matches("- [ ]").trim().to_owned())
                .collect()
        })
        .unwrap_or_default();

    let transitions = state_toml::transitions(progress_doc);
    let start = transitions.len().saturating_sub(RECENT_TRANSITION_LIMIT);
    let recent_transitions = transitions[start..]
        .iter()
        .map(|transition| {
            format!(
                "{}  {}  {}",
                transition.timestamp, transition.action, transition.stage
            )
        })
        .collect();

    let mut closure_summary = Vec::new();
    if let Some(closed_at) = state_toml::closed_at(&doc) {
        closure_summary.push(format!("closed_at: {closed_at}"));
    }
    if let Some(close_reason) = state_toml::close_reason(&doc) {
        closure_summary.push(format!("reason: {close_reason}"));
    }

    let next_action = build_next_action(
        artifact_root,
        &current_phase,
        &current_status,
        state_toml::next_action(progress_doc),
    );

    Ok(TuiOverview {
        task_name: state_toml::task_name(&doc),
        task_dir: task_dir.to_path_buf(),
        active_topic,
        active_topic_title,
        topic_dir,
        lifecycle,
        workspace_bucket,
        current_phase,
        current_status,
        next_action,
        done_stage_count,
        total_stage_count,
        missing_artifacts,
        todo_summary,
        recent_transitions,
        review_summary: review_summary(task_dir),
        knowledge_summary: knowledge_summary(task_dir),
        closure_summary,
    })
}

fn build_next_action(
    artifact_root: &Path,
    current_phase: &str,
    current_status: &str,
    fallback: String,
) -> String {
    let guide_dir = artifact_root.join("guides").join(current_phase);
    if let Some(guide) = find_action_guide(&guide_dir)
        && let Ok(content) = fs::read_to_string(&guide)
    {
        let mut lines = Vec::new();
        let relative_guide = relative_display(artifact_root, &guide);
        lines.push(format!("Current: {current_phase} / {current_status}"));
        if let Some(slice) = markdown_value(&content, "Current slice") {
            lines.push(format!("Slice: {slice}"));
        }
        if let Some(gap) = markdown_value(&content, "Current gap") {
            lines.push(format!("Gap: {gap}"));
        }
        let actions = action_card_items(&content);
        if !actions.is_empty() {
            lines.push(format!("Next: {}", actions.join(" -> ")));
        }
        if let Some(after) = markdown_value(&content, "After this") {
            lines.push(format!("Unlocks: {after}"));
        }
        lines.push(format!("Guide: {relative_guide}"));
        return lines.join("\n");
    }

    if let Some(todo_summary) = todo_now_summary(&artifact_root.join(".daedalus").join("todo.md")) {
        return todo_summary;
    }

    fallback
}

fn find_action_guide(guide_dir: &Path) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    let entries = fs::read_dir(guide_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if path.is_file() && name.starts_with("slice-") && name.ends_with(".md") {
            candidates.push(path);
        }
    }
    candidates.sort();
    candidates.pop()
}

fn markdown_value(content: &str, key: &str) -> Option<String> {
    let prefix = format!("- {key}:");
    content.lines().find_map(|line| {
        line.trim()
            .strip_prefix(&prefix)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    })
}

fn action_card_items(content: &str) -> Vec<String> {
    let mut in_action_card = false;
    let mut values = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "## Action Card" {
            in_action_card = true;
            continue;
        }
        if in_action_card && trimmed.starts_with("## ") {
            break;
        }
        if in_action_card
            && trimmed.starts_with("### ")
            && let Some((_, title)) = trimmed.trim_start_matches("### ").split_once(". ")
        {
            values.push(title.to_owned());
        }
        if values.len() >= 3 {
            break;
        }
    }
    values
}

fn todo_now_summary(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let mut in_now = false;
    let mut lines = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "## Now" {
            in_now = true;
            continue;
        }
        if in_now && trimmed.starts_with("## ") {
            break;
        }
        if in_now
            && trimmed.starts_with("- ")
            && (trimmed.contains("当前问题")
                || trimmed.contains("当前待解决")
                || trimmed.contains("完成后解锁"))
        {
            lines.push(trimmed.trim_start_matches("- ").to_owned());
        }
        if lines.len() >= 4 {
            break;
        }
    }
    (!lines.is_empty()).then(|| lines.join("\n"))
}

fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

/// 查找当前目录是否处于某个学习任务目录或其子目录下。
pub fn enclosing_task_dir(cwd: &Path, repo_root: &Path) -> Option<PathBuf> {
    if let Some(topic_dir) = workspace_fs::enclosing_topic_dir(cwd) {
        return workspace_fs::project_dir_from_topic_dir(&topic_dir).ok();
    }
    workspace_fs::enclosing_project_dir(cwd).filter(|path| path.starts_with(repo_root))
}

/// 扫描当前已具备明确 lifecycle 语义的 workspace bucket。
pub fn scan_task_summaries(repo_root: &Path) -> Result<Vec<TuiTaskSummary>> {
    let mut tasks = Vec::new();
    for bucket in WORKSPACE_BUCKETS {
        let root = repo_root.join("workspaces").join(bucket);
        if !root.exists() {
            continue;
        }
        let entries = fs::read_dir(&root).map_err(|source| crate::domain::DaedalusError::Io {
            path: root.clone(),
            source,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| crate::domain::DaedalusError::Io {
                path: root.clone(),
                source,
            })?;
            let path = entry.path();
            if path.is_dir() && path.join(".daedalus").join("state.toml").exists() {
                tasks.push(summary_from_task_dir(&path, bucket)?);
            }
        }
    }
    tasks.sort_by(|left, right| {
        bucket_order(&left.bucket)
            .cmp(&bucket_order(&right.bucket))
            .then_with(|| left.task_name.cmp(&right.task_name))
    });
    Ok(tasks)
}

fn summary_from_task_dir(task_dir: &Path, bucket: &str) -> Result<TuiTaskSummary> {
    let doc = state_toml::load_state_doc(&state_toml::state_path(task_dir))?;
    let (_, topic_doc) = active_topic_doc(task_dir, &doc)?;
    let progress_doc = topic_doc.as_ref().unwrap_or(&doc);
    let active_topic = topic_doc
        .as_ref()
        .map(state_toml::topic_slug)
        .or_else(|| state_toml::active_topic(&doc))
        .unwrap_or_else(|| "none".to_owned());
    let current_phase =
        state_toml::current_phase(progress_doc).unwrap_or_else(|| "unknown".to_owned());
    let stages = state_toml::stages(progress_doc);
    let current_status = stages
        .iter()
        .find(|stage| stage.id == current_phase)
        .map(|stage| stage.status.clone())
        .unwrap_or_else(|| "unknown".to_owned());
    Ok(TuiTaskSummary {
        task_name: state_toml::task_name(&doc),
        path: task_dir.to_path_buf(),
        bucket: bucket.to_owned(),
        lifecycle: task_field(&doc, "lifecycle").unwrap_or_else(|| "unknown".to_owned()),
        active_topic,
        current_phase,
        current_status,
        done_stage_count: stages.iter().filter(|stage| stage.status == "done").count(),
        total_stage_count: stages.len(),
    })
}

fn active_topic_doc(
    project_dir: &Path,
    project_doc: &toml_edit::DocumentMut,
) -> Result<(Option<PathBuf>, Option<toml_edit::DocumentMut>)> {
    let Some(slug) = state_toml::active_topic(project_doc).filter(|slug| !slug.is_empty()) else {
        return Ok((None, None));
    };
    let Some(path) = state_toml::topic_path(project_doc, &slug) else {
        return Ok((None, None));
    };
    let topic_dir = project_dir.join(path);
    let topic_doc = state_toml::load_state_doc(&state_toml::state_path(&topic_dir))?;
    Ok((Some(topic_dir), Some(topic_doc)))
}

fn task_field(doc: &toml_edit::DocumentMut, field: &str) -> Option<String> {
    doc["task"][field].as_str().map(ToOwned::to_owned)
}

fn actual_bucket(task_dir: &Path) -> String {
    workspace_fs::bucket_from_task_dir(task_dir)
        .map(|bucket| bucket.as_str().to_owned())
        .unwrap_or_else(|_| {
            task_dir
                .parent()
                .and_then(Path::file_name)
                .and_then(|value| value.to_str())
                .unwrap_or("unknown")
                .to_owned()
        })
}

fn review_summary(task_dir: &Path) -> Vec<String> {
    let mut values = Vec::new();
    collect_review_labels(
        &task_dir.join(".daedalus").join("reviews"),
        "project",
        &mut values,
    );
    if let Ok(doc) = state_toml::load_state_doc(&state_toml::state_path(task_dir)) {
        for topic in state_toml::topics(&doc) {
            collect_review_labels(
                &task_dir.join(&topic.path).join(".daedalus").join("reviews"),
                &topic.slug,
                &mut values,
            );
        }
    }
    values.into_iter().take(6).collect()
}

fn collect_review_labels(root: &Path, owner: &str, values: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("state.toml").exists() {
            let id = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("unknown");
            values.push(format!("{owner}: {id}"));
        }
    }
}

fn knowledge_summary(task_dir: &Path) -> Vec<String> {
    let mut values = Vec::new();
    let shared = task_dir.join("shared").join("knowledge-system");
    if shared.exists() {
        values.push("shared knowledge-system present".to_owned());
    }
    if let Ok(doc) = state_toml::load_state_doc(&state_toml::state_path(task_dir)) {
        for topic in state_toml::topics(&doc) {
            let extraction = task_dir
                .join(&topic.path)
                .join("notes")
                .join("knowledge-system")
                .join("extraction.md");
            if extraction.exists() {
                values.push(format!("{}: extraction candidates", topic.slug));
            }
        }
    }
    values.into_iter().take(6).collect()
}

fn bucket_order(bucket: &str) -> usize {
    WORKSPACE_BUCKETS
        .iter()
        .position(|candidate| *candidate == bucket)
        .unwrap_or(WORKSPACE_BUCKETS.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_action_prefers_slice_action_guide() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let root = temp.path();
        let guide_dir = root.join("guides/08-demo-coder");
        fs::create_dir_all(&guide_dir).expect("guide dir");
        fs::write(
            guide_dir.join("slice-6-hardening.md"),
            r#"# Slice 6 Hardening Guide

## Learning Navigation

- Current slice: Slice 6 Agent Orchestrator
- Current gap: live path 已跑通，但 deterministic tests 和安全阀不足
- After this: 可以把 approval / sandbox / retry 接入 ReAct loop

## Action Card

### 1. Add `max_turns`

### 2. Emit `ToolCallFinished`

### 3. Add Fake LLM Tests

## Completion Criteria
"#,
        )
        .expect("guide");

        let next_action =
            build_next_action(root, "08-demo-coder", "active", "short fallback".to_owned());

        assert!(next_action.contains("Current: 08-demo-coder / active"));
        assert!(next_action.contains("Slice: Slice 6 Agent Orchestrator"));
        assert!(next_action.contains("Gap: live path 已跑通"));
        assert!(next_action.contains("Next: Add `max_turns` -> Emit `ToolCallFinished`"));
        assert!(next_action.contains("Guide: guides/08-demo-coder/slice-6-hardening.md"));
        assert!(!next_action.contains("short fallback"));
    }
}
