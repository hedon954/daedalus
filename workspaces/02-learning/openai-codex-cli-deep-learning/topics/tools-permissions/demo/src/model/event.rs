use std::path::PathBuf;

use crate::model::approval::{ApprovalPersistence, ApprovalScope, SandboxProfile};

/// Agent 执行过程中生产的高层事件。
///
/// TODO: 当前 demo 的对外 stream event 主要定义在 `agent::stream_event`；
/// 后续如果要完整对齐设计文档，可把 approval / sandbox / retry 事件统一收拢到这里。
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

/// 用户审批决策。
///
/// Phase 1 还没有真实 UI，可以由 scripted approval / fake decider 产生；
/// Phase 2 再接入真实交互与 session 级持久化。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UserApprovalDecision {
    /// 批准
    Approved { persistence: ApprovalPersistence },
    /// 拒绝
    Rejected,
}

/// 执行尝试。
///
/// 同一个 `CommandRequest` 可能经历 sandbox first 和 no-sandbox retry。
/// runner 必须接收 attempt，才能让测试和事件流看见“这一次为什么这样执行”。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExecutionAttempt {
    /// 带沙箱首次尝试
    SandboxFirst { sandbox_profile: SandboxProfile },
    /// 不带沙箱首次尝试
    NoSandboxFirst { reason: String },
    /// 无沙箱重试
    NoSandboxRetry { reason: String },
}

/// 重试决策。
///
/// 这是 sandbox 失败后的二级 gate，只处理“是否允许第二次尝试”；
/// 它不负责真正执行，也不直接修改 approval policy。
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
