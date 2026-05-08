use std::path::PathBuf;

use crate::application::render::render_state;
use crate::domain::transition::Transition;
use crate::domain::{ApprovalSource, DaedalusError, Result, StageStatus};
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
    let state_path = state_toml::state_path(&options.task_dir);
    let mut doc = state_toml::load_state_doc(&state_path)?;
    if !state_toml::stage_exists(&doc, &options.stage_id) {
        return Err(DaedalusError::InvalidStageId(options.stage_id));
    }

    let action_name = match &options.action {
        StageAction::Enter => {
            state_toml::set_other_active_to_blocked(&mut doc, &options.stage_id);
            state_toml::set_current_phase(&mut doc, &options.stage_id);
            state_toml::set_stage_status(&mut doc, &options.stage_id, StageStatus::Active)?;
            "enter"
        }
        StageAction::Complete {
            force,
            approval_source,
        } => {
            let missing =
                state_toml::collect_missing_artifacts(&doc, &options.task_dir, &options.stage_id)?;
            if !missing.is_empty() {
                if !force {
                    return Err(DaedalusError::MissingRequiredArtifact {
                        artifact: missing[0].clone(),
                        stage: options.stage_id,
                    });
                }
                validate_force(options.reason.as_deref(), *approval_source)?;
            }
            state_toml::set_stage_status(&mut doc, &options.stage_id, StageStatus::Done)?;
            "complete"
        }
        StageAction::Block => {
            require_specific_reason(options.reason.as_deref())?;
            state_toml::set_stage_status(&mut doc, &options.stage_id, StageStatus::Blocked)?;
            "block"
        }
        StageAction::Resume => {
            state_toml::set_current_phase(&mut doc, &options.stage_id);
            state_toml::set_stage_status(&mut doc, &options.stage_id, StageStatus::Active)?;
            "resume"
        }
    };

    let approval_source = match &options.action {
        StageAction::Complete {
            approval_source, ..
        } => approval_source.map(|source| source.as_str().to_owned()),
        _ => None,
    };
    state_toml::append_transition(
        &mut doc,
        Transition {
            stage: options.stage_id.clone(),
            action: action_name.to_owned(),
            timestamp: clock::now_rfc3339(),
            actor: options.actor,
            reason: options.reason.clone().unwrap_or_else(|| {
                format!("Run `{action_name}` for stage `{}`.", options.stage_id)
            }),
            approval_source,
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

fn validate_force(reason: Option<&str>, approval_source: Option<ApprovalSource>) -> Result<()> {
    require_specific_reason(reason)?;
    if approval_source.is_none() {
        return Err(DaedalusError::ForceRequiresApproval);
    }
    Ok(())
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
