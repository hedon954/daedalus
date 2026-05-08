use std::fs;
use std::path::Path;

use crate::domain::Result;
use crate::infrastructure::state_toml;

/// TUI 只读总览所需的数据。
#[derive(Debug, Clone)]
pub struct TuiOverview {
    /// 学习任务名称。
    pub task_name: String,
    /// 当前阶段 ID。
    pub current_phase: String,
    /// 当前阶段状态。
    pub current_status: String,
    /// 当前缺失的关键产物。
    pub missing_artifacts: Vec<String>,
    /// `todo.md` 中的待办摘要。
    pub todo_summary: Vec<String>,
    /// 最近几条状态流转摘要。
    pub recent_transitions: Vec<String>,
}

/// 从学习任务目录加载 TUI 只读总览数据。
pub fn load_overview(task_dir: &Path) -> Result<TuiOverview> {
    let doc = state_toml::load_state_doc(&state_toml::state_path(task_dir))?;
    let current_phase = state_toml::current_phase(&doc).unwrap_or_else(|| "unknown".to_owned());
    let current_status = state_toml::stages(&doc)
        .into_iter()
        .find(|stage| stage.id == current_phase)
        .map(|stage| stage.status)
        .unwrap_or_else(|| "unknown".to_owned());

    let mut missing_artifacts = Vec::new();
    for stage in state_toml::stages(&doc) {
        for artifact in stage.required_artifacts {
            if !task_dir.join(&artifact).exists() {
                missing_artifacts.push(format!("{artifact} ({})", stage.id));
            }
        }
    }

    let todo_summary = fs::read_to_string(task_dir.join(".daedalus").join("todo.md"))
        .ok()
        .map(|content| {
            content
                .lines()
                .filter(|line| line.trim_start().starts_with("- [ ]"))
                .take(6)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default();

    let transitions = state_toml::transitions(&doc);
    let start = transitions.len().saturating_sub(5);
    let recent_transitions = transitions[start..]
        .iter()
        .map(|transition| {
            format!(
                "{} {} {}",
                transition.timestamp, transition.action, transition.stage
            )
        })
        .collect();

    Ok(TuiOverview {
        task_name: state_toml::task_name(&doc),
        current_phase,
        current_status,
        missing_artifacts,
        todo_summary,
        recent_transitions,
    })
}
