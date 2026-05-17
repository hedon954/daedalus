use std::path::PathBuf;

use crate::model::approval::{ApprovalPersistence, ApprovalScope, SandboxProfile};

/// Agent 执行过程中生产的事件
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AgentEvent {
    /// Agent 启动
    AgentStarted { session_id: String, cwd: PathBuf },
}

/// Agent 执行过程中的状态
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum AgentStatus {
    /// Agent 成功
    Success,
    /// Agent 失败
    Failed,
    /// Agent 取消
    Cancelled,
}

/// 用户审批决策
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UserApprovalDecision {
    /// 批准
    Approved { persistence: ApprovalPersistence },
    /// 拒绝
    Rejected,
}

/// 执行尝试
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExecutionAttempt {
    /// 带沙箱首次尝试
    SandboxFirst { sandbox_profile: SandboxProfile },
    /// 不带沙箱首次尝试
    NoSandboxFirst { reason: String },
    /// 无沙箱重试
    NoSandboxRetry { reason: String },
}

/// 重试决策
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RetryDecision {
    /// 不重试
    DoNotRetry {
        /// 原因
        reason: String,
    },
    /// 无请求批准重试
    RetryWithoutApproval {
        /// 原因
        reason: String,
    },
    /// 请求批准重试
    RetryWithApproval {
        /// 原因
        reason: String,
        /// 审批范围
        approval_scope: ApprovalScope,
    },
}
