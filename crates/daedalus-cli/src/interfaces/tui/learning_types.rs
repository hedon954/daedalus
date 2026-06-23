use std::fs;
use std::path::{Path, PathBuf};

use crate::infrastructure::state_toml;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GuideLookupStrategy {
    RepoActionGuide,
    StageReadme,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct LearningTypeTuiAdapter {
    pub(super) kind: &'static str,
    pub(super) label: &'static str,
    pub(super) current_unit_label: &'static str,
    pub(super) progress_unit_label: &'static str,
    pub(super) current_reading_label: &'static str,
    pub(super) closeout_label: &'static str,
    closeout_stage_id: &'static str,
    closeout_loop: &'static str,
    closeout_existing_next: &'static str,
    closeout_missing_next: &'static str,
    guide_lookup: GuideLookupStrategy,
}

const REPO_LEARNING_TUI: LearningTypeTuiAdapter = LearningTypeTuiAdapter {
    kind: "repo-learning",
    label: "Repo Learning",
    current_unit_label: "Stage",
    progress_unit_label: "repo stages",
    current_reading_label: "Current Guide",
    closeout_label: "Reflection Loop",
    closeout_stage_id: "10-reflection",
    closeout_loop: "知识候选表 -> 用户 closeout -> Agent challenge -> 知识库归档",
    closeout_existing_next: "打开 reflection/candidate-map.md，围绕候选表逐项确认状态",
    closeout_missing_next: "创建 reflection/candidate-map.md，并用既有 guides / notes / demo / tests 查漏补缺",
    guide_lookup: GuideLookupStrategy::RepoActionGuide,
};

const COURSE_LEARNING_TUI: LearningTypeTuiAdapter = LearningTypeTuiAdapter {
    kind: "course-learning",
    label: "Course Learning",
    current_unit_label: "Course Stage",
    progress_unit_label: "course stages",
    current_reading_label: "Course Guide",
    closeout_label: "Closeout Loop",
    closeout_stage_id: "09-closeout-archive",
    closeout_loop: "lesson evidence -> mastery review -> transfer patterns -> archive",
    closeout_existing_next: "打开 reflection/candidate-map.md，确认课程概念、实验、迁移和薄弱点候选",
    closeout_missing_next: "创建 reflection/candidate-map.md，并从 lesson notes / review / demo evidence 提取候选",
    guide_lookup: GuideLookupStrategy::StageReadme,
};

const GENERIC_LEARNING_TUI: LearningTypeTuiAdapter = LearningTypeTuiAdapter {
    kind: "unknown",
    label: "Learning",
    current_unit_label: "Stage",
    progress_unit_label: "stages",
    current_reading_label: "Current Guide",
    closeout_label: "Closeout Loop",
    closeout_stage_id: "closeout",
    closeout_loop: "evidence -> review -> archive",
    closeout_existing_next: "打开 reflection/candidate-map.md，确认候选和缺口",
    closeout_missing_next: "创建 reflection/candidate-map.md，并从现有学习证据中提取候选",
    guide_lookup: GuideLookupStrategy::StageReadme,
};

pub(super) fn learning_type_adapter(kind: &str) -> LearningTypeTuiAdapter {
    match kind {
        "repo-learning" => REPO_LEARNING_TUI,
        "course-learning" => COURSE_LEARNING_TUI,
        _ => LearningTypeTuiAdapter {
            kind: "unknown",
            ..GENERIC_LEARNING_TUI
        },
    }
}

pub(super) fn build_next_action(
    adapter: LearningTypeTuiAdapter,
    artifact_root: &Path,
    current_phase: &str,
    current_status: &str,
    fallback: String,
) -> String {
    if current_phase == adapter.closeout_stage_id {
        return closeout_next_action(adapter, artifact_root, current_status, fallback);
    }

    if let Some(guide) = find_current_guide(adapter, artifact_root, current_phase)
        && let Ok(content) = fs::read_to_string(&guide)
    {
        return match adapter.guide_lookup {
            GuideLookupStrategy::RepoActionGuide => repo_next_action_from_guide(
                adapter,
                artifact_root,
                current_phase,
                current_status,
                &guide,
                &content,
            ),
            GuideLookupStrategy::StageReadme => stage_readme_next_action(
                adapter,
                artifact_root,
                current_phase,
                current_status,
                &guide,
                &content,
            ),
        };
    }

    if let Some(todo_summary) = todo_now_summary(&artifact_root.join(".daedalus").join("todo.md")) {
        return todo_summary;
    }

    fallback
}

fn repo_next_action_from_guide(
    adapter: LearningTypeTuiAdapter,
    artifact_root: &Path,
    current_phase: &str,
    current_status: &str,
    guide: &Path,
    content: &str,
) -> String {
    if is_actionable_guide(guide) {
        let mut lines = Vec::new();
        let relative_guide = relative_display(artifact_root, guide);
        lines.push(format!(
            "{}: {current_phase} / {current_status}",
            adapter.current_unit_label
        ));
        if let Some(slice) = markdown_value(content, "Current slice") {
            lines.push(format!("Slice: {slice}"));
        }
        if let Some(gap) = markdown_value(content, "Current gap") {
            lines.push(format!("Gap: {gap}"));
        }
        let actions = action_card_items(content);
        if !actions.is_empty() {
            lines.push(format!("Next: {}", actions.join(" -> ")));
        }
        if let Some(after) = markdown_value(content, "After this") {
            lines.push(format!("Unlocks: {after}"));
        }
        lines.push(format!(
            "{}: {relative_guide}",
            adapter.current_reading_label
        ));
        return lines.join("\n");
    }

    stage_readme_next_action(
        adapter,
        artifact_root,
        current_phase,
        current_status,
        guide,
        content,
    )
}

fn stage_readme_next_action(
    adapter: LearningTypeTuiAdapter,
    artifact_root: &Path,
    current_phase: &str,
    current_status: &str,
    guide: &Path,
    content: &str,
) -> String {
    let mut lines = Vec::new();
    let relative_guide = relative_display(artifact_root, guide);
    lines.push(format!(
        "{}: {current_phase} / {current_status}",
        adapter.current_unit_label
    ));
    if let Some(lesson) = markdown_value(content, "Lesson")
        .or_else(|| markdown_value(content, "当前目标"))
        .or_else(|| markdown_value(content, "目标"))
    {
        lines.push(format!("Focus: {lesson}"));
    }
    if let Some(gap) = markdown_value(content, "Current gap")
        .or_else(|| markdown_value(content, "当前问题"))
        .or_else(|| markdown_value(content, "Current open decision"))
    {
        lines.push(format!("Gap: {gap}"));
    }
    let next_items = section_bullets(content, "Next Lesson Lab")
        .into_iter()
        .chain(section_bullets(content, "Action Card"))
        .take(3)
        .collect::<Vec<_>>();
    if !next_items.is_empty() {
        lines.push(format!("Next: {}", next_items.join(" -> ")));
    }
    lines.push(format!(
        "{}: {relative_guide}",
        adapter.current_reading_label
    ));
    lines.join("\n")
}

fn closeout_next_action(
    adapter: LearningTypeTuiAdapter,
    artifact_root: &Path,
    current_status: &str,
    fallback: String,
) -> String {
    let root = artifact_root.join("reflection");
    if !root.exists() {
        return fallback;
    }
    [
        format!(
            "{}: {} / {current_status}",
            adapter.current_unit_label, adapter.closeout_stage_id
        ),
        format!("Loop: {}", adapter.closeout_loop),
        format!(
            "Next: {}",
            if root.join("candidate-map.md").exists() {
                adapter.closeout_existing_next
            } else {
                adapter.closeout_missing_next
            }
        ),
        format!(
            "{}: {}",
            adapter.current_reading_label,
            relative_display(artifact_root, &root.join("candidate-map.md"))
        ),
    ]
    .join("\n")
}

pub(super) fn find_current_guide(
    adapter: LearningTypeTuiAdapter,
    artifact_root: &Path,
    current_phase: &str,
) -> Option<PathBuf> {
    if current_phase == adapter.closeout_stage_id {
        let candidate_map = artifact_root.join("reflection").join("candidate-map.md");
        if candidate_map.exists() {
            return Some(candidate_map);
        }
    }
    let guide_dir = artifact_root.join("guides").join(current_phase);
    match adapter.guide_lookup {
        GuideLookupStrategy::RepoActionGuide => {
            find_action_guide(&guide_dir).or_else(|| find_stage_readme(&guide_dir))
        }
        GuideLookupStrategy::StageReadme => find_stage_readme(&guide_dir),
    }
}

fn find_action_guide(guide_dir: &Path) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    let entries = fs::read_dir(guide_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if path.is_file()
            && name.ends_with(".md")
            && name != "README.md"
            && name.contains("slice")
            && is_actionable_guide(&path)
        {
            candidates.push(path);
        }
    }
    candidates.sort();
    candidates.pop()
}

fn find_stage_readme(guide_dir: &Path) -> Option<PathBuf> {
    let readme = guide_dir.join("README.md");
    if readme.exists() {
        return Some(readme);
    }

    let mut candidates = Vec::new();
    let entries = fs::read_dir(guide_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if path.is_file() && name.ends_with(".md") {
            candidates.push(path);
        }
    }
    candidates.sort();
    candidates.pop()
}

fn is_actionable_guide(path: &Path) -> bool {
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    if markdown_value(&content, "Current slice")
        .map(|value| value.to_lowercase().contains("completed"))
        .unwrap_or(false)
    {
        return false;
    }
    content.contains("## Action Card")
}

fn markdown_value(content: &str, key: &str) -> Option<String> {
    let ascii_prefix = format!("- {key}:");
    let fullwidth_prefix = format!("- {key}：");
    content.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix(&ascii_prefix)
            .or_else(|| trimmed.strip_prefix(&fullwidth_prefix))
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    })
}

fn section_bullets(content: &str, heading: &str) -> Vec<String> {
    let mut in_section = false;
    let mut values = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == format!("## {heading}") {
            in_section = true;
            continue;
        }
        if in_section && trimmed.starts_with("## ") {
            break;
        }
        if in_section && trimmed.starts_with("- ") {
            values.push(trimmed.trim_start_matches("- ").trim().to_owned());
        }
        if values.len() >= 3 {
            break;
        }
    }
    values
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

pub(super) fn learning_source_reference(
    adapter: LearningTypeTuiAdapter,
    doc: &toml_edit::DocumentMut,
) -> Option<String> {
    match adapter.kind {
        "course-learning" => {
            state_toml::course_url(doc).or_else(|| project_field(doc, "source_name"))
        }
        "repo-learning" => project_field(doc, "source_url")
            .or_else(|| project_field(doc, "repo_url"))
            .or_else(|| project_field(doc, "source_name")),
        _ => project_field(doc, "source_name"),
    }
}

fn project_field(doc: &toml_edit::DocumentMut, field: &str) -> Option<String> {
    doc["project"][field]
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
}

pub(super) fn reflection_summary(
    adapter: LearningTypeTuiAdapter,
    artifact_root: &Path,
) -> Vec<String> {
    let root = artifact_root.join("reflection");
    let candidate_label = if adapter.kind == "course-learning" {
        "课程候选表"
    } else {
        "候选表"
    };
    [(candidate_label, "candidate-map.md")]
        .into_iter()
        .filter_map(|(label, file)| {
            let path = root.join(file);
            path.exists()
                .then(|| format!("{label}: {}", relative_display(artifact_root, &path)))
        })
        .collect()
}

pub(super) fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
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

        let next_action = build_next_action(
            REPO_LEARNING_TUI,
            root,
            "08-demo-coder",
            "active",
            "short fallback".to_owned(),
        );

        assert!(next_action.contains("Stage: 08-demo-coder / active"));
        assert!(next_action.contains("Slice: Slice 6 Agent Orchestrator"));
        assert!(next_action.contains("Gap: live path 已跑通"));
        assert!(next_action.contains("Next: Add `max_turns` -> Emit `ToolCallFinished`"));
        assert!(next_action.contains("Current Guide: guides/08-demo-coder/slice-6-hardening.md"));
        assert!(!next_action.contains("short fallback"));
    }

    #[test]
    fn next_action_accepts_numbered_slice_guide_and_ignores_completed_guide() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let root = temp.path();
        let guide_dir = root.join("guides/08-demo-coder");
        fs::create_dir_all(&guide_dir).expect("guide dir");
        fs::write(
            guide_dir.join("07-slice-7-policy-composition.md"),
            r#"# Slice 7

## Learning Navigation

- Current slice: Slice 7 completed
- Current gap: old completed work

## Action Card

### 1. Old completed action
"#,
        )
        .expect("completed guide");
        fs::write(
            guide_dir.join("08-slice-8-event-protocol.md"),
            r#"# Slice 8

## Learning Navigation

- Current slice: Slice 8 Event Protocol Hardening
- Current gap: event protocol is not observable enough
- After this: README can explain the command safety trace

## Action Card

### 1. Define event protocol

### 2. Emit shell runtime events
"#,
        )
        .expect("active guide");

        let next_action = build_next_action(
            REPO_LEARNING_TUI,
            root,
            "08-demo-coder",
            "active",
            "short fallback".to_owned(),
        );

        assert!(next_action.contains("Slice: Slice 8 Event Protocol Hardening"));
        assert!(next_action.contains("Gap: event protocol is not observable enough"));
        assert!(next_action.contains("Next: Define event protocol -> Emit shell runtime events"));
        assert!(
            next_action
                .contains("Current Guide: guides/08-demo-coder/08-slice-8-event-protocol.md")
        );
        assert!(!next_action.contains("Old completed action"));
    }

    #[test]
    fn reflection_phase_uses_candidate_map_as_current_reading() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let root = temp.path();
        let guide = root.join("reflection/candidate-map.md");
        fs::create_dir_all(guide.parent().expect("reflection parent")).expect("reflection dir");
        fs::write(&guide, "# 知识候选表\n").expect("reflection");

        let current_guide =
            find_current_guide(REPO_LEARNING_TUI, root, "10-reflection").expect("current guide");

        assert_eq!(current_guide, guide);
    }
}
