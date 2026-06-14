/// 一次命令执行尝试的结果。
///
/// 这个类型只描述 runner 已经尝试执行后的结果，不表达“是否应该审批”。
/// 审批与重试决策分别由 `ApprovalRequirement` 和 `RetryDecision` 表达。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExecutionResult {
    /// 成功
    Success { stdout: String },
    /// 失败
    Failure(ExecutionFailure),
}

/// 一次命令执行尝试的失败原因。
///
/// `CommandFailed` 表示命令自身失败；`SandboxDenied` 表示命令可能正确，
/// 但当前沙箱/网络权限不足。retry gate 依赖这个区分判断是否允许提权重试。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExecutionFailure {
    /// 命令执行失败
    CommandFailed {
        /// 命令退出码
        exit_code: i32,
        /// 命令错误输出
        stderr: String,
    },
    /// 沙箱拒绝
    SandboxDenied {
        /// 沙箱拒绝输出
        output: String,
        /// 网络上下文
        network_context: Option<NetworkApprovalContext>,
    },
}

/// 网络审批上下文。
///
/// 当 sandbox 能识别出被拒绝的网络目标时，后续可以把审批范围收窄到 host/protocol。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NetworkApprovalContext {
    /// 网络主机
    pub host: String,
    /// 网络协议
    pub protocol: String,
}
