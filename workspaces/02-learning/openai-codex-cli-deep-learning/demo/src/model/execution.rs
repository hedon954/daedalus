/// 工具执行结果
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExecutionResult {
    /// 成功
    Success { stdout: String },
    /// 失败
    Failure(ExecutionFailure),
}

/// 工具执行失败原因
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

/// 网络上下文
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NetworkApprovalContext {
    /// 网络主机
    pub host: String,
    /// 网络协议
    pub protocol: String,
}
