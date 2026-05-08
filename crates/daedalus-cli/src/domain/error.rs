use std::path::PathBuf;

/// daedalus CLI 内部统一使用的结果类型。
pub type Result<T> = std::result::Result<T, DaedalusError>;

/// 强制通过类操作的批准来源。
///
/// 该类型用于约束 Agent 使用 `--force` 等绕过选项时必须留下可追溯依据。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalSource {
    /// 用户在对话中明确批准。
    UserConfirmed,
    /// 缺失产物已有等价证据替代。
    ArtifactEquivalent,
    /// 当前阶段被明确判定为不适用。
    StageNotApplicable,
}

impl ApprovalSource {
    /// 返回写入 `state.toml` 的稳定字符串。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UserConfirmed => "user-confirmed",
            Self::ArtifactEquivalent => "artifact-equivalent",
            Self::StageNotApplicable => "stage-not-applicable",
        }
    }

    /// 从 CLI 参数解析批准来源。
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "user-confirmed" => Some(Self::UserConfirmed),
            "artifact-equivalent" => Some(Self::ArtifactEquivalent),
            "stage-not-applicable" => Some(Self::StageNotApplicable),
            _ => None,
        }
    }
}

/// daedalus CLI 的业务错误集合。
///
/// 错误变体需要保持稳定，方便 Agent 根据错误输出选择下一步行动。
#[derive(Debug, thiserror::Error)]
pub enum DaedalusError {
    /// 未找到唯一可用的学习任务 workspace。
    #[error("no active workspace found")]
    NoActiveWorkspace,
    /// 当前目录不在 daedalus 项目根或其子目录下。
    #[error("not a daedalus project directory: {0}")]
    NotDaedalusProject(PathBuf),
    /// WIP 约束阻止创建第二个 active task。
    #[error("task already active: {0}")]
    TaskAlreadyActive(PathBuf),
    /// 任务移动目标目录已存在，拒绝覆盖。
    #[error("task move destination already exists: {0}")]
    TaskMoveDestinationExists(PathBuf),
    /// 任务生命周期操作缺少具体原因。
    #[error("task lifecycle operation requires a specific reason")]
    TaskLifecycleReasonRequired,
    /// 任务生命周期状态不允许当前流转。
    #[error("invalid task lifecycle transition: {0}")]
    InvalidTaskLifecycleTransition(String),
    /// 任务 lifecycle 和所在 workspace bucket 不一致。
    #[error("task lifecycle and workspace bucket mismatch: {0}")]
    TaskLifecycleLocationMismatch(String),
    /// `.daedalus/state.toml` 不存在。
    #[error("state file missing: {0}")]
    StateFileMissing(PathBuf),
    /// 命令传入了不存在的阶段 ID。
    #[error("invalid stage id: {0}")]
    InvalidStageId(String),
    /// 状态文件中出现多个 active 阶段。
    #[error("multiple active stages")]
    MultipleActiveStages,
    /// 完成阶段时缺少必需产物。
    #[error("missing required artifact: {artifact}")]
    MissingRequiredArtifact { artifact: PathBuf, stage: String },
    /// `state.md` 落后于 `state.toml`。
    #[error("state markdown is stale")]
    StateMarkdownStale,
    /// 强制通过缺少理由或批准来源。
    #[error("force requires approval and a specific reason")]
    ForceRequiresApproval,
    /// 强制通过理由过于空泛。
    #[error("invalid force reason")]
    InvalidForceReason,
    /// CLI 传入了不支持的批准来源。
    #[error("invalid approval source: {0}")]
    InvalidApprovalSource(String),
    /// workspace 校验发现一组问题。
    #[error("workspace validation failed")]
    WorkspaceValidationFailed(Vec<String>),
    /// 文件系统操作失败。
    #[error("io error at {path}: {source}")]
    Io {
        /// 发生错误的路径。
        path: PathBuf,
        #[source]
        /// 原始 IO 错误。
        source: std::io::Error,
    },
    /// TOML 解析失败。
    #[error("toml parse error in {path}: {source}")]
    Toml {
        /// 发生错误的 TOML 文件路径。
        path: PathBuf,
        #[source]
        /// `toml_edit` 返回的解析错误。
        source: toml_edit::TomlError,
    },
}
