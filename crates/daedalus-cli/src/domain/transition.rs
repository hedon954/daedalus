//! 状态流转记录。

/// 一次学习阶段状态变化。
///
/// 每次进入、完成、阻塞、恢复或回退阶段时，都应该追加一条 transition，
/// 以便后续从文件系统恢复学习过程。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    /// 被操作的阶段 ID。
    pub stage: String,
    /// 操作名称，例如 `enter`、`complete`、`block`、`resume` 或 `rollback`。
    pub action: String,
    /// RFC3339 时间戳。
    pub timestamp: String,
    /// 操作者标识，通常是 `agent` 或 `daedalus-cli`。
    pub actor: String,
    /// 本次状态变化的原因。
    pub reason: String,
    /// 强制通过类操作的批准来源。
    pub approval_source: Option<String>,
}
