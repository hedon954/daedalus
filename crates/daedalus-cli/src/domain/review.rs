//! review plan 领域类型。

/// 复习计划目标。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewTarget {
    /// Project-level review。
    Project,
    /// Topic-level review。
    Topic(String),
}

/// 复习模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewMode {
    /// 不看笔记主动回忆。
    Recall,
    /// 从空白重建设计。
    Rebuild,
    /// 迁移到新业务场景。
    Application,
    /// 针对薄弱点修复。
    WeaknessRepair,
    /// 混合模式。
    Mixed,
}

impl ReviewMode {
    /// 返回写入 `state.toml` 的稳定字符串。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Recall => "recall",
            Self::Rebuild => "rebuild",
            Self::Application => "application",
            Self::WeaknessRepair => "weakness-repair",
            Self::Mixed => "mixed",
        }
    }

    /// 从字符串解析复习模式。
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "recall" => Some(Self::Recall),
            "rebuild" => Some(Self::Rebuild),
            "application" => Some(Self::Application),
            "weakness-repair" => Some(Self::WeaknessRepair),
            "mixed" => Some(Self::Mixed),
            _ => None,
        }
    }
}

/// Review 生命周期。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewLifecycle {
    /// 已规划。
    Planned,
    /// 正在复习。
    Active,
    /// 暂停。
    Paused,
    /// 完成。
    Completed,
    /// 放弃。
    Abandoned,
}

impl ReviewLifecycle {
    /// 返回写入 `state.toml` 的稳定字符串。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Abandoned => "abandoned",
        }
    }

    /// 从字符串解析 review 生命周期。
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "planned" => Some(Self::Planned),
            "active" => Some(Self::Active),
            "paused" => Some(Self::Paused),
            "completed" => Some(Self::Completed),
            "abandoned" => Some(Self::Abandoned),
            _ => None,
        }
    }
}

/// Review 列表展示快照。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewSnapshot {
    /// Review id。
    pub id: String,
    /// target_type: project / topic。
    pub target_type: String,
    /// target 名称。
    pub target: String,
    /// 复习模式。
    pub mode: String,
    /// 生命周期。
    pub lifecycle: String,
    /// review 目录路径。
    pub path: String,
    /// 下一步。
    pub next_action: String,
}
