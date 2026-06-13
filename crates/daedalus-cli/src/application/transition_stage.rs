use std::path::PathBuf;

use crate::application::render::render_state;
use crate::application::state_machine::StateTransition;
use crate::domain::transition::Transition;
use crate::domain::{
    ApprovalSource, DaedalusError, Result, StageState, StageTransitionKind, TopicLifecycle,
};
use crate::infrastructure::{clock, state_toml};

/// 阶段状态流转动作。
#[derive(Debug, Clone)]
pub enum StageAction {
    /// 进入一个阶段，并将其设为 active。
    Enter,
    /// 完成一个阶段，可在受控条件下强制通过缺失产物检查。
    Complete {
        /// 是否强制完成。
        force: bool,
        /// 强制完成时的批准来源。
        approval_source: Option<ApprovalSource>,
    },
    /// 将阶段标记为 blocked。
    Block,
    /// 恢复 blocked 或 paused 阶段。
    Resume,
    /// 回退到某个已到达阶段，并将其后的阶段重置为 pending。
    Rollback,
}

impl StageAction {
    fn kind(&self) -> StageTransitionKind {
        match self {
            Self::Enter => StageTransitionKind::Enter,
            Self::Complete { .. } => StageTransitionKind::Complete,
            Self::Block => StageTransitionKind::Block,
            Self::Resume => StageTransitionKind::Resume,
            Self::Rollback => StageTransitionKind::Rollback,
        }
    }

    fn approval_source(&self) -> Option<ApprovalSource> {
        match self {
            Self::Complete {
                approval_source, ..
            } => *approval_source,
            _ => None,
        }
    }
}

/// 阶段流转 use case 的输入参数。
#[derive(Debug, Clone)]
pub struct TransitionStageOptions {
    /// 学习任务根目录。
    pub task_dir: PathBuf,
    /// 要操作的阶段 ID。
    pub stage_id: String,
    /// 要执行的状态流转动作。
    pub action: StageAction,
    /// 状态流转原因。
    pub reason: Option<String>,
    /// 操作者标识。
    pub actor: String,
}

/// 阶段流转完成后的输出。
#[derive(Debug, Clone)]
pub struct TransitionStageOutput {
    /// 学习任务根目录。
    pub task_dir: PathBuf,
    /// 被操作的阶段 ID。
    pub stage_id: String,
    /// 已执行的动作名称。
    pub action: String,
    /// 重新生成后的 `state.md` 路径。
    pub state_md: PathBuf,
}

/// 执行一次学习阶段状态流转。
///
/// 该函数负责更新 `state.toml`、追加 `[[transitions]]`，并在成功后重新渲染
/// `.daedalus/state.md`。
pub fn transition_stage(options: TransitionStageOptions) -> Result<TransitionStageOutput> {
    options.apply()
}

impl StateTransition for TransitionStageOptions {
    type Output = TransitionStageOutput;

    fn pre_check(&self) -> Result<()> {
        let state_path = state_toml::state_path(&self.task_dir);
        let doc = state_toml::load_state_doc(&state_path)?;
        ensure_stage_transition_topic_lifecycle(&doc, self)?;
        validate_stage_transition(&doc, self)
    }

    fn commit(self) -> Result<Self::Output> {
        commit_stage_transition(self)
    }
}

fn commit_stage_transition(options: TransitionStageOptions) -> Result<TransitionStageOutput> {
    let state_path = state_toml::state_path(&options.task_dir);
    let mut doc = state_toml::load_state_doc(&state_path)?;
    let kind = options.action.kind();
    let action_name = kind.as_str();

    match &options.action {
        StageAction::Enter => {
            state_toml::set_other_active_to_blocked(&mut doc, &options.stage_id);
            state_toml::set_current_phase(&mut doc, &options.stage_id);
        }
        StageAction::Complete { .. } => {
            let next_stage = state_toml::next_pending_stage_after(&doc, &options.stage_id);
            update_next_action_after_complete(&mut doc, next_stage.as_ref());
            if let Some(stage) = next_stage {
                state_toml::set_current_phase(&mut doc, &stage.id);
            }
        }
        StageAction::Block => {}
        StageAction::Resume => {
            state_toml::set_other_active_to_blocked(&mut doc, &options.stage_id);
            state_toml::set_current_phase(&mut doc, &options.stage_id);
        }
        StageAction::Rollback => {
            state_toml::rollback_to_stage(&mut doc, &options.stage_id)?;
            state_toml::set_current_phase(&mut doc, &options.stage_id);
            update_next_action_after_rollback(&mut doc, &options.stage_id);
        }
    }
    state_toml::set_stage_state(&mut doc, &options.stage_id, target_state(kind))?;

    state_toml::append_transition(
        &mut doc,
        Transition {
            stage: options.stage_id.clone(),
            action: action_name.to_owned(),
            timestamp: clock::now_local_timestamp(),
            actor: options.actor,
            reason: options.reason.clone().unwrap_or_else(|| {
                format!("Run `{action_name}` for stage `{}`.", options.stage_id)
            }),
            approval_source: options
                .action
                .approval_source()
                .map(|source| source.as_str().to_owned()),
        },
    );
    state_toml::save_state_doc(&state_path, &doc)?;
    let state_md = render_state(&options.task_dir)?.path;

    Ok(TransitionStageOutput {
        task_dir: options.task_dir,
        stage_id: options.stage_id,
        action: action_name.to_owned(),
        state_md,
    })
}

fn validate_stage_transition(
    doc: &toml_edit::DocumentMut,
    options: &TransitionStageOptions,
) -> Result<()> {
    let state = state_toml::stage_state(doc, &options.stage_id)?;
    if !is_awaiting_reflection_final_stage_complete(doc, options, state)? {
        state.transition(options.action.kind())?;
    }

    match &options.action {
        StageAction::Enter => {}
        StageAction::Complete {
            force,
            approval_source,
        } => {
            validate_completion_artifacts(
                doc,
                &options.task_dir,
                &options.stage_id,
                *force,
                *approval_source,
                options.reason.as_deref(),
            )?;
        }
        StageAction::Block | StageAction::Resume | StageAction::Rollback => {
            require_specific_reason(options.reason.as_deref())?;
        }
    }

    Ok(())
}

fn is_awaiting_reflection_final_stage_complete(
    doc: &toml_edit::DocumentMut,
    options: &TransitionStageOptions,
    state: StageState,
) -> Result<bool> {
    Ok(
        state_toml::topic_lifecycle(doc)? == TopicLifecycle::AwaitingReflection
            && options.stage_id == "10-reflection"
            && matches!(options.action, StageAction::Complete { .. })
            && state == StageState::Pending,
    )
}

fn validate_completion_artifacts(
    doc: &toml_edit::DocumentMut,
    task_dir: &std::path::Path,
    stage_id: &str,
    force: bool,
    approval_source: Option<ApprovalSource>,
    reason: Option<&str>,
) -> Result<()> {
    let missing = state_toml::collect_missing_artifacts(doc, task_dir, stage_id)?;
    if missing.is_empty() {
        return Ok(());
    }
    if !force {
        return Err(DaedalusError::MissingRequiredArtifact {
            artifact: missing[0].clone(),
            stage: stage_id.to_owned(),
        });
    }
    validate_force(reason, approval_source)
}

fn target_state(kind: StageTransitionKind) -> StageState {
    match kind {
        StageTransitionKind::Enter | StageTransitionKind::Resume => StageState::Active,
        StageTransitionKind::Complete => StageState::Done,
        StageTransitionKind::Block => StageState::Blocked,
        StageTransitionKind::Rollback => StageState::Active,
    }
}

fn ensure_stage_transition_topic_lifecycle(
    doc: &toml_edit::DocumentMut,
    options: &TransitionStageOptions,
) -> Result<()> {
    let kind = state_toml::state_kind(doc);
    if kind != "topic" {
        return Err(DaedalusError::InvalidStateDocumentKind {
            expected: "topic".to_owned(),
            actual: kind.to_owned(),
        });
    }
    let lifecycle = state_toml::topic_lifecycle(doc)?;
    if lifecycle == TopicLifecycle::Active {
        return Ok(());
    }
    if lifecycle == TopicLifecycle::AwaitingReflection
        && matches!(options.action, StageAction::Complete { .. })
        && options.stage_id == "10-reflection"
    {
        return Ok(());
    }
    Err(DaedalusError::InvalidTopicLifecycleTransition(format!(
        "expected active topic, got {}",
        lifecycle.as_str()
    )))
}

fn validate_force(reason: Option<&str>, approval_source: Option<ApprovalSource>) -> Result<()> {
    require_specific_reason(reason)?;
    if approval_source.is_none() {
        return Err(DaedalusError::ForceRequiresApproval);
    }
    Ok(())
}

fn update_next_action_after_complete(
    doc: &mut toml_edit::DocumentMut,
    next_stage: Option<&crate::domain::StageSnapshot>,
) {
    let next_action = next_stage
        .map(|stage| {
            let artifacts = if stage.required_artifacts.is_empty() {
                "更新该阶段需要的学习产物".to_owned()
            } else {
                format!("准备产物：{}", stage.required_artifacts.join("、"))
            };
            format!("进入 `{}`：{}，{}。", stage.id, stage.title, artifacts)
        })
        .unwrap_or_else(|| "所有阶段已完成；复核归档产物并关闭学习任务。".to_owned());
    state_toml::set_next_action(doc, &next_action);
}

fn update_next_action_after_rollback(doc: &mut toml_edit::DocumentMut, stage_id: &str) {
    let stage = state_toml::stages(doc)
        .into_iter()
        .find(|stage| stage.id == stage_id);
    let next_action = stage
        .map(|stage| {
            let artifacts = if stage.required_artifacts.is_empty() {
                "复核该阶段学习状态和后续阅读计划".to_owned()
            } else {
                format!("复核产物：{}", stage.required_artifacts.join("、"))
            };
            format!("已回退到 `{}`：{}，{}。", stage.id, stage.title, artifacts)
        })
        .unwrap_or_else(|| format!("已回退到 `{stage_id}`；复核学习状态和后续计划。"));
    state_toml::set_next_action(doc, &next_action);
}

fn require_specific_reason(reason: Option<&str>) -> Result<()> {
    let Some(reason) = reason.map(str::trim).filter(|value| !value.is_empty()) else {
        return Err(DaedalusError::ForceRequiresApproval);
    };
    if matches!(reason, "continue" | "skip" | "not needed") {
        return Err(DaedalusError::InvalidForceReason);
    }
    Ok(())
}
