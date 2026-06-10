use std::path::PathBuf;

/// 学习任务生命周期状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskLifecycle {
    /// 至少一个专题正在推进。
    Active,
    /// 没有 active topic，但项目仍是正式学习对象。
    Idle,
    /// Legacy completed state；新稳定工作区用 `idle` 表达项目空闲。
    Completed,
    /// 项目不进入默认学习视图。
    Abandoned,
}

impl TaskLifecycle {
    /// 返回写入 `state.toml` 的稳定字符串。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Idle => "idle",
            Self::Completed => "completed",
            Self::Abandoned => "abandoned",
        }
    }

    /// 从 `state.toml` 字符串解析生命周期状态。
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "idle" => Some(Self::Idle),
            "completed" => Some(Self::Completed),
            "abandoned" => Some(Self::Abandoned),
            _ => None,
        }
    }

    /// 返回该生命周期对应的稳定 workspace bucket。
    pub fn expected_bucket(self) -> WorkspaceBucket {
        match self {
            Self::Active | Self::Idle | Self::Completed | Self::Abandoned => {
                WorkspaceBucket::Projects
            }
        }
    }
}

/// 学习任务所在 workspace bucket。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceBucket {
    /// 稳定 project bucket。
    Projects,
    /// Legacy active learning bucket。
    /// active learning bucket。
    Learning,
    /// Legacy completed bucket。
    Completed,
    /// Legacy abandoned bucket。
    Abandoned,
}

impl WorkspaceBucket {
    /// 返回写入 `state.toml` 的稳定字符串。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Projects => "projects",
            Self::Learning => "02-learning",
            Self::Completed => "03-completed",
            Self::Abandoned => "04-abandoned",
        }
    }

    /// 从 `state.toml` 字符串解析 workspace bucket。
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "projects" => Some(Self::Projects),
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
