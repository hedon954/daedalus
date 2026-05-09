use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::application::render::render_state;
use crate::application::state_machine::StateTransition;
use crate::domain::transition::Transition;
use crate::domain::{
    ApprovalSource, DaedalusError, Result, StageState, StageTransitionKind, TaskLifecycle,
    WorkspaceBucket,
};
use crate::infrastructure::{clock, state_toml, workspace_fs};

const FINAL_STAGE_ID: &str = "10-archivist";

/// 任务生命周期关闭动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseTaskAction {
    /// 完成学习任务。
    Complete,
    /// 放弃学习任务。
    Abandon,
}

impl CloseTaskAction {
    /// 返回写入 `state.toml` 的稳定动作名。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "task-complete",
            Self::Abandon => "abandon",
        }
    }

    fn destination_root(self, repo_root: &std::path::Path) -> PathBuf {
        match self {
            Self::Complete => workspace_fs::completed_root(repo_root),
            Self::Abandon => workspace_fs::abandoned_root(repo_root),
        }
    }

    fn lifecycle(self) -> TaskLifecycle {
        match self {
            Self::Complete => TaskLifecycle::Completed,
            Self::Abandon => TaskLifecycle::Abandoned,
        }
    }

    fn bucket(self) -> WorkspaceBucket {
        match self {
            Self::Complete => WorkspaceBucket::Completed,
            Self::Abandon => WorkspaceBucket::Abandoned,
        }
    }

    fn next_message(self) -> &'static str {
        match self {
            Self::Complete => {
                "active WIP slot released; start a new task with daedalus init repo-learning <name>"
            }
            Self::Abandon => {
                "active WIP slot released; review the abandoned task before starting a replacement"
            }
        }
    }
}

impl CloseTaskOptions {
    fn destination_root(&self) -> PathBuf {
        self.action.destination_root(&self.repo_root)
    }
}

/// 关闭学习任务 use case 的输入参数。
#[derive(Debug, Clone)]
pub struct CloseTaskOptions {
    /// daedalus 项目根目录。
    pub repo_root: PathBuf,
    /// 学习任务根目录。
    pub task_dir: PathBuf,
    /// 生命周期关闭动作。
    pub action: CloseTaskAction,
    /// 关闭原因。
    pub reason: String,
    /// 操作者标识。
    pub actor: String,
    /// 是否在关闭前完成最终归档阶段。
    pub complete_final_stage: bool,
    /// 完成最终归档阶段时是否强制通过缺失产物。
    pub final_stage_force: bool,
    /// 强制完成最终归档阶段时的批准来源。
    pub final_stage_approval_source: Option<ApprovalSource>,
}

/// 关闭学习任务后的结构化输出。
#[derive(Debug, Clone)]
pub struct CloseTaskOutput {
    /// 生命周期动作。
    pub action: String,
    /// 关闭后的任务生命周期。
    pub lifecycle: String,
    /// 是否发生目录移动。
    pub moved: bool,
    /// 移动前任务目录。
    pub from_task_dir: PathBuf,
    /// 移动后任务目录。
    pub to_task_dir: PathBuf,
    /// 移动后重新渲染的 `state.md` 路径。
    pub state_md: PathBuf,
    /// 追加生命周期决策记录的路径。
    pub decision_log: PathBuf,
    /// 面向 Agent 的下一步建议。
    pub next: String,
}

/// 关闭学习任务并移动到 completed 或 abandoned bucket。
pub fn close_task(options: CloseTaskOptions) -> Result<CloseTaskOutput> {
    options.apply()
}

impl StateTransition for CloseTaskOptions {
    type Output = CloseTaskOutput;

    fn pre_check(&self) -> Result<()> {
        let reason = normalize_reason(&self.reason)?;
        let task_dir = canonical_task_dir(&self.task_dir)?;
        let doc = state_toml::load_state_doc(&state_toml::state_path(&task_dir))?;
        ensure_active_learning_task(&doc, &task_dir)?;
        let destination = planned_destination(&task_dir, &self.destination_root())?;
        if destination.exists() {
            return Err(DaedalusError::TaskMoveDestinationExists(destination));
        }
        if self.complete_final_stage {
            validate_final_stage_completion(
                &doc,
                &task_dir,
                reason,
                self.final_stage_force,
                self.final_stage_approval_source,
            )?;
        }
        Ok(())
    }

    fn commit(self) -> Result<Self::Output> {
        commit_close_task(self)
    }
}

fn commit_close_task(options: CloseTaskOptions) -> Result<CloseTaskOutput> {
    let reason = normalize_reason(&options.reason)?;
    let task_dir = canonical_task_dir(&options.task_dir)?;
    let state_path = state_toml::state_path(&task_dir);
    let mut doc = state_toml::load_state_doc(&state_path)?;
    let closed_at = clock::now_local_timestamp();
    let destination_root = options.destination_root();
    let planned_to_task_dir = planned_destination(&task_dir, &destination_root)?;

    if options.complete_final_stage {
        complete_final_stage(&mut doc, reason, &options.actor, closed_at.clone())?;
    }
    state_toml::pause_active_stages(&mut doc);

    let next = options.action.next_message().to_owned();
    let lifecycle = options.action.lifecycle();
    let bucket = options.action.bucket();
    state_toml::set_task_lifecycle(&mut doc, lifecycle);
    state_toml::set_workspace_bucket(&mut doc, bucket);
    state_toml::set_task_close_info(&mut doc, &closed_at, reason);
    state_toml::set_next_action(&mut doc, &next);
    state_toml::append_transition(
        &mut doc,
        Transition {
            stage: "task".to_owned(),
            action: options.action.as_str().to_owned(),
            timestamp: closed_at.clone(),
            actor: options.actor,
            reason: reason.to_owned(),
            approval_source: None,
        },
    );
    state_toml::save_state_doc(&state_path, &doc)?;
    append_close_decision(
        &task_dir,
        &planned_to_task_dir,
        options.action,
        lifecycle,
        &closed_at,
        reason,
    )?;
    let _ = render_state(&task_dir)?;

    let to_task_dir = workspace_fs::move_task(&task_dir, &destination_root)?;
    let state_md = render_state(&to_task_dir)?.path;
    let decision_log = to_task_dir.join(".daedalus").join("decision-log.md");

    Ok(CloseTaskOutput {
        action: options.action.as_str().to_owned(),
        lifecycle: lifecycle.as_str().to_owned(),
        moved: true,
        from_task_dir: task_dir,
        to_task_dir,
        state_md,
        decision_log,
        next,
    })
}

fn complete_final_stage(
    doc: &mut toml_edit::DocumentMut,
    reason: &str,
    actor: &str,
    timestamp: String,
) -> Result<()> {
    state_toml::set_stage_state(doc, FINAL_STAGE_ID, StageState::Done)?;
    state_toml::set_current_phase(doc, FINAL_STAGE_ID);
    state_toml::append_transition(
        doc,
        Transition {
            stage: FINAL_STAGE_ID.to_owned(),
            action: "complete".to_owned(),
            timestamp,
            actor: actor.to_owned(),
            reason: reason.to_owned(),
            approval_source: None,
        },
    );
    Ok(())
}

fn canonical_task_dir(task_dir: &std::path::Path) -> Result<PathBuf> {
    task_dir.canonicalize().map_err(|source| DaedalusError::Io {
        path: task_dir.to_path_buf(),
        source,
    })
}

fn planned_destination(task_dir: &Path, destination_root: &Path) -> Result<PathBuf> {
    let task_name = task_dir
        .file_name()
        .map(ToOwned::to_owned)
        .ok_or(DaedalusError::NoActiveWorkspace)?;
    Ok(destination_root.join(task_name))
}

fn append_close_decision(
    task_dir: &Path,
    to_task_dir: &Path,
    action: CloseTaskAction,
    lifecycle: TaskLifecycle,
    timestamp: &str,
    reason: &str,
) -> Result<PathBuf> {
    let decision_log = task_dir.join(".daedalus").join("decision-log.md");
    let decision = match action {
        CloseTaskAction::Complete => "完成学习任务",
        CloseTaskAction::Abandon => "放弃学习任务",
    };
    let entry = format!(
        "\n## {timestamp}\n\n- 决策：{decision}，将 lifecycle 设为 `{}`。\n- 原因：{reason}\n- 影响：任务从 active WIP 释放，并移动到 [`{}`](..)。\n",
        lifecycle.as_str(),
        to_task_dir.display(),
    );
    let mut file = OpenOptions::new()
        .append(true)
        .open(&decision_log)
        .map_err(|source| DaedalusError::Io {
            path: decision_log.clone(),
            source,
        })?;
    file.write_all(entry.as_bytes())
        .map_err(|source| DaedalusError::Io {
            path: decision_log.clone(),
            source,
        })?;
    Ok(decision_log)
}

fn ensure_active_learning_task(
    doc: &toml_edit::DocumentMut,
    task_dir: &std::path::Path,
) -> Result<()> {
    let lifecycle = state_toml::task_lifecycle(doc)?;
    if lifecycle != TaskLifecycle::Active {
        return Err(DaedalusError::InvalidTaskLifecycleTransition(format!(
            "expected active task, got {}",
            lifecycle.as_str()
        )));
    }
    let state_bucket = state_toml::workspace_bucket(doc)?;
    let actual_bucket = workspace_fs::bucket_from_task_dir(task_dir)?;
    if state_bucket != WorkspaceBucket::Learning || actual_bucket != WorkspaceBucket::Learning {
        return Err(DaedalusError::TaskLifecycleLocationMismatch(format!(
            "expected active task in 02-learning, state={}, actual={}",
            state_bucket.as_str(),
            actual_bucket.as_str()
        )));
    }
    Ok(())
}

fn validate_final_stage_completion(
    doc: &toml_edit::DocumentMut,
    task_dir: &Path,
    reason: &str,
    force: bool,
    approval_source: Option<ApprovalSource>,
) -> Result<()> {
    let state = state_toml::stage_state(doc, FINAL_STAGE_ID)?;
    state.transition(StageTransitionKind::Complete)?;

    let missing = state_toml::collect_missing_artifacts(doc, task_dir, FINAL_STAGE_ID)?;
    if missing.is_empty() {
        return Ok(());
    }
    if !force {
        return Err(DaedalusError::MissingRequiredArtifact {
            artifact: missing[0].clone(),
            stage: FINAL_STAGE_ID.to_owned(),
        });
    }
    validate_final_stage_force(reason, approval_source)
}

fn validate_final_stage_force(reason: &str, approval_source: Option<ApprovalSource>) -> Result<()> {
    normalize_reason(reason)?;
    if approval_source.is_none() {
        return Err(DaedalusError::ForceRequiresApproval);
    }
    Ok(())
}

fn normalize_reason(reason: &str) -> Result<&str> {
    let reason = reason.trim();
    if reason.is_empty() || matches!(reason, "continue" | "skip" | "not needed") {
        return Err(DaedalusError::TaskLifecycleReasonRequired);
    }
    Ok(reason)
}
