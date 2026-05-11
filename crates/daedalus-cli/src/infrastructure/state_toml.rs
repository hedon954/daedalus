use std::fs;
use std::path::{Path, PathBuf};

use toml_edit::{Array, DocumentMut, Item, Table, value};

use crate::domain::transition::Transition;
use crate::domain::{
    DaedalusError, Result, StageSnapshot, StageState, TaskLifecycle, WorkspaceBucket,
};

/// 返回学习任务的 `state.toml` 路径。
pub fn state_path(task_dir: &Path) -> PathBuf {
    task_dir.join(".daedalus").join("state.toml")
}

/// 返回学习任务的生成态 `state.md` 路径。
pub fn state_md_path(task_dir: &Path) -> PathBuf {
    task_dir.join(".daedalus").join("state.md")
}

/// 读取并解析 `state.toml`。
///
/// 使用 `toml_edit::DocumentMut` 是为了在后续写回时尽量保留注释和字段顺序。
pub fn load_state_doc(path: &Path) -> Result<DocumentMut> {
    if !path.exists() {
        return Err(DaedalusError::StateFileMissing(path.to_path_buf()));
    }
    let content = fs::read_to_string(path).map_err(|source| DaedalusError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    content
        .parse::<DocumentMut>()
        .map_err(|source| DaedalusError::Toml {
            path: path.to_path_buf(),
            source,
        })
}

/// 保存修改后的 `state.toml` 文档。
pub fn save_state_doc(path: &Path, doc: &DocumentMut) -> Result<()> {
    fs::write(path, doc.to_string()).map_err(|source| DaedalusError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// 读取当前阶段 ID。
pub fn current_phase(doc: &DocumentMut) -> Option<String> {
    doc["task"]["current_phase"].as_str().map(ToOwned::to_owned)
}

/// 读取任务生命周期。
pub fn task_lifecycle(doc: &DocumentMut) -> Result<TaskLifecycle> {
    let value = doc["task"]["lifecycle"]
        .as_str()
        .ok_or_else(|| DaedalusError::InvalidTaskLifecycleTransition("missing lifecycle".into()))?;
    TaskLifecycle::parse(value)
        .ok_or_else(|| DaedalusError::InvalidTaskLifecycleTransition(value.to_owned()))
}

/// 设置任务生命周期。
pub fn set_task_lifecycle(doc: &mut DocumentMut, lifecycle: TaskLifecycle) {
    doc["task"]["lifecycle"] = value(lifecycle.as_str());
}

/// 读取任务所在 workspace bucket。
pub fn workspace_bucket(doc: &DocumentMut) -> Result<WorkspaceBucket> {
    let value = doc["task"]["workspace_bucket"].as_str().ok_or_else(|| {
        DaedalusError::TaskLifecycleLocationMismatch("missing workspace_bucket".into())
    })?;
    WorkspaceBucket::parse(value)
        .ok_or_else(|| DaedalusError::TaskLifecycleLocationMismatch(value.to_owned()))
}

/// 设置任务所在 workspace bucket。
pub fn set_workspace_bucket(doc: &mut DocumentMut, bucket: WorkspaceBucket) {
    doc["task"]["workspace_bucket"] = value(bucket.as_str());
}

/// 设置任务关闭信息。
pub fn set_task_close_info(doc: &mut DocumentMut, closed_at: &str, reason: &str) {
    doc["task"]["closed_at"] = value(closed_at);
    doc["task"]["close_reason"] = value(reason);
}

/// 读取任务关闭时间。
pub fn closed_at(doc: &DocumentMut) -> Option<String> {
    doc["task"]
        .get("closed_at")
        .and_then(|item| item.as_str())
        .map(ToOwned::to_owned)
}

/// 读取任务关闭原因。
pub fn close_reason(doc: &DocumentMut) -> Option<String> {
    doc["task"]
        .get("close_reason")
        .and_then(|item| item.as_str())
        .map(ToOwned::to_owned)
}

/// 设置当前阶段 ID。
pub fn set_current_phase(doc: &mut DocumentMut, stage_id: &str) {
    doc["task"]["current_phase"] = value(stage_id);
}

/// 设置面向 Agent 的下一步动作提示。
pub fn set_next_action(doc: &mut DocumentMut, next_action: &str) {
    doc["task"]["next_action"] = value(next_action);
}

/// 读取所有阶段快照。
pub fn stages(doc: &DocumentMut) -> Vec<StageSnapshot> {
    doc["stages"]
        .as_array_of_tables()
        .map(|array| {
            array
                .iter()
                .map(|stage| StageSnapshot {
                    id: stage["id"].as_str().unwrap_or_default().to_owned(),
                    title: stage["title"].as_str().unwrap_or_default().to_owned(),
                    status: stage["status"].as_str().unwrap_or_default().to_owned(),
                    required_artifacts: string_array(stage.get("required_artifacts")),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// 统计 active 阶段数量。
pub fn active_stage_count(doc: &DocumentMut) -> usize {
    stages(doc)
        .iter()
        .filter(|stage| stage.status == "active")
        .count()
}

/// 判断阶段 ID 是否存在。
pub fn stage_exists(doc: &DocumentMut, stage_id: &str) -> bool {
    stages(doc).iter().any(|stage| stage.id == stage_id)
}

/// 读取某个阶段的强类型运行时状态。
pub fn stage_state(doc: &DocumentMut, stage_id: &str) -> Result<StageState> {
    let stage = stages(doc)
        .into_iter()
        .find(|stage| stage.id == stage_id)
        .ok_or_else(|| DaedalusError::InvalidStageId(stage_id.to_owned()))?;
    StageState::parse(&stage.status).ok_or(DaedalusError::InvalidStageState {
        stage: stage.id,
        state: stage.status,
    })
}

/// 查找某个阶段之后的下一个 pending 阶段。
pub fn next_pending_stage_after(doc: &DocumentMut, stage_id: &str) -> Option<StageSnapshot> {
    let mut seen_current = false;
    for stage in stages(doc) {
        if seen_current && stage.status == "pending" {
            return Some(stage);
        }
        if stage.id == stage_id {
            seen_current = true;
        }
    }
    None
}

/// 读取某个阶段的必需产物列表。
pub fn stage_required_artifacts(doc: &DocumentMut, stage_id: &str) -> Result<Vec<String>> {
    stages(doc)
        .into_iter()
        .find(|stage| stage.id == stage_id)
        .map(|stage| stage.required_artifacts)
        .ok_or_else(|| DaedalusError::InvalidStageId(stage_id.to_owned()))
}

/// 设置某个阶段的状态。
pub fn set_stage_state(doc: &mut DocumentMut, stage_id: &str, state: StageState) -> Result<()> {
    let Some(array) = doc["stages"].as_array_of_tables_mut() else {
        return Err(DaedalusError::InvalidStageId(stage_id.to_owned()));
    };

    for stage in array.iter_mut() {
        if stage["id"].as_str() == Some(stage_id) {
            stage["status"] = value(state.as_str());
            return Ok(());
        }
    }

    Err(DaedalusError::InvalidStageId(stage_id.to_owned()))
}

/// 将除目标阶段外的 active 阶段标记为 blocked。
pub fn set_other_active_to_blocked(doc: &mut DocumentMut, stage_id: &str) {
    if let Some(array) = doc["stages"].as_array_of_tables_mut() {
        for stage in array.iter_mut() {
            if stage["id"].as_str() != Some(stage_id) && stage["status"].as_str() == Some("active")
            {
                stage["status"] = value("blocked");
            }
        }
    }
}

/// 回退到目标阶段：目标阶段设为 active，目标之后的阶段全部重置为 pending。
pub fn rollback_to_stage(doc: &mut DocumentMut, stage_id: &str) -> Result<()> {
    let Some(array) = doc["stages"].as_array_of_tables_mut() else {
        return Err(DaedalusError::InvalidStageId(stage_id.to_owned()));
    };

    let mut seen_target = false;
    for stage in array.iter_mut() {
        if stage["id"].as_str() == Some(stage_id) {
            stage["status"] = value(StageState::Active.as_str());
            seen_target = true;
        } else if seen_target {
            stage["status"] = value(StageState::Pending.as_str());
        }
    }

    if seen_target {
        Ok(())
    } else {
        Err(DaedalusError::InvalidStageId(stage_id.to_owned()))
    }
}

/// 关闭任务时将仍处于 active 的阶段标记为 paused，避免 closed task 继续呈现进行中状态。
pub fn pause_active_stages(doc: &mut DocumentMut) {
    if let Some(array) = doc["stages"].as_array_of_tables_mut() {
        for stage in array.iter_mut() {
            if stage["status"].as_str() == Some("active") {
                stage["status"] = value(StageState::Paused.as_str());
            }
        }
    }
}

/// 追加一条状态流转记录。
pub fn append_transition(doc: &mut DocumentMut, transition: Transition) {
    let mut table = Table::new();
    table["stage"] = value(transition.stage);
    table["action"] = value(transition.action);
    table["timestamp"] = value(transition.timestamp);
    table["actor"] = value(transition.actor);
    table["reason"] = value(transition.reason);
    if let Some(approval_source) = transition.approval_source {
        table["approval_source"] = value(approval_source);
    }

    if !doc.as_table().contains_key("transitions") {
        doc["transitions"] = Item::ArrayOfTables(Default::default());
    }
    if let Some(array) = doc["transitions"].as_array_of_tables_mut() {
        array.push(table);
    }
}

/// 读取状态流转历史。
pub fn transitions(doc: &DocumentMut) -> Vec<Transition> {
    doc["transitions"]
        .as_array_of_tables()
        .map(|array| {
            array
                .iter()
                .map(|item| Transition {
                    stage: item["stage"].as_str().unwrap_or_default().to_owned(),
                    action: item["action"].as_str().unwrap_or_default().to_owned(),
                    timestamp: item["timestamp"].as_str().unwrap_or_default().to_owned(),
                    actor: item["actor"].as_str().unwrap_or_default().to_owned(),
                    reason: item["reason"].as_str().unwrap_or_default().to_owned(),
                    approval_source: item
                        .get("approval_source")
                        .and_then(|value| value.as_str())
                        .map(ToOwned::to_owned),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// 收集某个阶段缺失的必需产物。
pub fn collect_missing_artifacts(
    doc: &DocumentMut,
    task_dir: &Path,
    stage_id: &str,
) -> Result<Vec<PathBuf>> {
    let artifacts = stage_required_artifacts(doc, stage_id)?;
    Ok(artifacts
        .into_iter()
        .map(PathBuf::from)
        .filter(|artifact| !task_dir.join(artifact).exists())
        .collect())
}

/// 读取学习任务名称。
pub fn task_name(doc: &DocumentMut) -> String {
    doc["task"]["name"].as_str().unwrap_or("unknown").to_owned()
}

/// 读取面向 Agent 的下一步动作提示。
pub fn next_action(doc: &DocumentMut) -> String {
    doc["task"]["next_action"]
        .as_str()
        .unwrap_or("Review current stage and update todo.md.")
        .to_owned()
}

fn string_array(item: Option<&Item>) -> Vec<String> {
    item.and_then(Item::as_array)
        .map(|array: &Array| {
            array
                .iter()
                .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default()
}
