//! 学习阶段领域类型。

use crate::domain::error::{DaedalusError, Result};

/// 学习阶段状态流转动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageTransitionKind {
    /// 进入阶段。
    Enter,
    /// 完成阶段。
    Complete,
    /// 阻塞阶段。
    Block,
    /// 恢复阶段。
    Resume,
}

impl StageTransitionKind {
    /// 返回写入状态流转记录的稳定动作名。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Enter => "enter",
            Self::Complete => "complete",
            Self::Block => "block",
            Self::Resume => "resume",
        }
    }
}

/// 学习阶段的运行时状态。
///
/// 字符串形式会直接写入 `.daedalus/state.toml`，因此需要保持稳定。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageState {
    /// 阶段尚未开始。
    Pending,
    /// 阶段正在进行。
    Active,
    /// 阶段被阻塞，等待用户或外部条件。
    Blocked,
    /// 阶段暂停，后续可恢复。
    Paused,
    /// 阶段已完成。
    Done,
}

impl StageState {
    /// 返回写入状态文件的稳定字符串。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Blocked => "blocked",
            Self::Paused => "paused",
            Self::Done => "done",
        }
    }

    /// 从状态文件中的字符串解析阶段状态。
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "active" => Some(Self::Active),
            "blocked" => Some(Self::Blocked),
            "paused" => Some(Self::Paused),
            "done" => Some(Self::Done),
            _ => None,
        }
    }

    /// 根据领域状态机计算执行动作后的目标状态。
    pub fn transition(self, kind: StageTransitionKind) -> Result<Self> {
        match (self, kind) {
            (Self::Pending, StageTransitionKind::Enter) => Ok(Self::Active),
            (Self::Active, StageTransitionKind::Complete) => Ok(Self::Done),
            (Self::Active, StageTransitionKind::Block) => Ok(Self::Blocked),
            (Self::Blocked | Self::Paused, StageTransitionKind::Resume) => Ok(Self::Active),
            _ => Err(DaedalusError::InvalidStageStateTransition {
                state: self.as_str().to_owned(),
                action: kind.as_str().to_owned(),
            }),
        }
    }
}

/// 从 `state.toml` 读取出的阶段快照。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageSnapshot {
    /// 阶段 ID，例如 `01-goal-aligner`。
    pub id: String,
    /// 面向用户展示的阶段标题。
    pub title: String,
    /// 阶段状态的原始字符串。
    pub status: String,
    /// 完成该阶段前必须存在的产物路径。
    pub required_artifacts: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::{StageState, StageTransitionKind};

    #[test]
    fn stage_state_transitions_follow_learning_workflow() {
        assert_eq!(
            StageState::Pending
                .transition(StageTransitionKind::Enter)
                .expect("pending can enter"),
            StageState::Active
        );
        assert_eq!(
            StageState::Active
                .transition(StageTransitionKind::Complete)
                .expect("active can complete"),
            StageState::Done
        );
        assert_eq!(
            StageState::Active
                .transition(StageTransitionKind::Block)
                .expect("active can block"),
            StageState::Blocked
        );
        assert_eq!(
            StageState::Blocked
                .transition(StageTransitionKind::Resume)
                .expect("blocked can resume"),
            StageState::Active
        );
        assert_eq!(
            StageState::Paused
                .transition(StageTransitionKind::Resume)
                .expect("paused can resume"),
            StageState::Active
        );
    }

    #[test]
    fn stage_state_transitions_reject_skips_and_terminal_changes() {
        assert!(
            StageState::Pending
                .transition(StageTransitionKind::Complete)
                .is_err()
        );
        assert!(
            StageState::Done
                .transition(StageTransitionKind::Resume)
                .is_err()
        );
        assert!(
            StageState::Blocked
                .transition(StageTransitionKind::Complete)
                .is_err()
        );
    }
}
