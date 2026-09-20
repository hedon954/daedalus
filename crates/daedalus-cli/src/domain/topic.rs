//! repo learning topic 领域类型。

/// 工作区默认最多搁置的专题数。
pub const DEFAULT_MAX_PARKED_TOPICS: u32 = 1;
/// 工作区最少可配置的搁置专题数。
pub const MIN_MAX_PARKED_TOPICS: u32 = 1;
/// 工作区最多可配置的搁置专题数。
pub const MAX_MAX_PARKED_TOPICS: u32 = 3;

/// 专题生命周期状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopicLifecycle {
    /// 已规划但尚未开始。
    Planned,
    /// 当前正在推进。
    Active,
    /// 专题被外部条件阻塞。
    Blocked,
    /// 学到一半，用户主动搁置。
    Parked,
    /// 主体学习完成，等待用户主动回顾和归档。
    AwaitingReflection,
    /// 专题已完成。
    Completed,
    /// 专题已放弃。
    Abandoned,
    /// 专题被显式跳过。
    Skipped,
}

/// 校验并接受工作区 parked 上限。
pub fn parse_max_parked_topics(value: u32) -> crate::domain::Result<u32> {
    if (MIN_MAX_PARKED_TOPICS..=MAX_MAX_PARKED_TOPICS).contains(&value) {
        Ok(value)
    } else {
        Err(crate::domain::DaedalusError::InvalidParkedTopicLimit(value))
    }
}

impl TopicLifecycle {
    /// 返回写入 `state.toml` 的稳定字符串。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Active => "active",
            Self::Blocked => "blocked",
            Self::Parked => "parked",
            Self::AwaitingReflection => "awaiting-reflection",
            Self::Completed => "completed",
            Self::Abandoned => "abandoned",
            Self::Skipped => "skipped",
        }
    }

    /// 从 `state.toml` 字符串解析专题生命周期。
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "planned" => Some(Self::Planned),
            "active" => Some(Self::Active),
            "blocked" => Some(Self::Blocked),
            "parked" => Some(Self::Parked),
            "awaiting-reflection" => Some(Self::AwaitingReflection),
            "completed" => Some(Self::Completed),
            "abandoned" => Some(Self::Abandoned),
            "skipped" => Some(Self::Skipped),
            _ => None,
        }
    }
}

/// Project state 中登记的专题摘要。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicSnapshot {
    /// 专题 slug。
    pub slug: String,
    /// 专题标题。
    pub title: String,
    /// 专题生命周期。
    pub lifecycle: String,
    /// 专题目录路径，相对 project root。
    pub path: String,
    /// 继承的 shared context 路径。
    pub inherits: Vec<String>,
}
