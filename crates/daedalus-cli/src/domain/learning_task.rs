use std::path::PathBuf;

/// 学习任务生命周期状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskLifecycle {
    /// 正在 active learning 区域中推进。
    Active,
    /// 已完成并归档到 completed 区域。
    Completed,
    /// 已放弃并归档到 abandoned 区域。
    Abandoned,
}

impl TaskLifecycle {
    /// 返回写入 `state.toml` 的稳定字符串。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Abandoned => "abandoned",
        }
    }

    /// 从 `state.toml` 字符串解析生命周期状态。
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "completed" => Some(Self::Completed),
            "abandoned" => Some(Self::Abandoned),
            _ => None,
        }
    }

    /// 返回该生命周期应该落入的 workspace bucket。
    pub fn expected_bucket(self) -> WorkspaceBucket {
        match self {
            Self::Active => WorkspaceBucket::Learning,
            Self::Completed => WorkspaceBucket::Completed,
            Self::Abandoned => WorkspaceBucket::Abandoned,
        }
    }
}

/// 学习任务所在 workspace bucket。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceBucket {
    /// active learning bucket。
    Learning,
    /// completed bucket。
    Completed,
    /// abandoned bucket。
    Abandoned,
}

impl WorkspaceBucket {
    /// 返回写入 `state.toml` 的稳定字符串。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Learning => "02-learning",
            Self::Completed => "03-completed",
            Self::Abandoned => "04-abandoned",
        }
    }

    /// 从 `state.toml` 字符串解析 workspace bucket。
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "02-learning" => Some(Self::Learning),
            "03-completed" => Some(Self::Completed),
            "04-abandoned" => Some(Self::Abandoned),
            _ => None,
        }
    }
}

/// 一个 daedalus 学习任务的最小领域表示。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearningTask {
    /// 学习任务名称。
    pub name: String,
    /// 学习任务在文件系统中的根目录。
    pub path: PathBuf,
}
