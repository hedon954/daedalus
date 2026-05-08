//! 学习阶段领域类型。

/// 学习阶段的状态。
///
/// 字符串形式会直接写入 `.daedalus/state.toml`，因此需要保持稳定。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageStatus {
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

impl StageStatus {
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
