use std::path::PathBuf;

use crate::model::approval::{ApprovalPolicy, NetworkPolicy, SandboxProfile};
use crate::model::capability::CapabilityKind;

/// 命令请求。
///
/// `CommandRequest` 是 shell tool 的执行上下文快照，由 host 从 tool call、
/// capability registry 和 runtime context 组装出来。模型只提供 raw command
/// 和可选 justification，不能直接提供权限字段。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandRequest {
    /// 原始命令
    pub raw_command: String,
    /// 简化后的 argv。
    ///
    /// TODO: 当前 Phase 1 只处理单命令和 `split_whitespace`；
    /// 后续多命令阶段需要用 command segment 取代单个 argv。
    pub argv: Vec<String>,
    /// 当前工作目录
    pub cwd: PathBuf,
    /// 命令能力
    pub capability: CapabilityKind,
    /// 权限策略
    pub approval_policy: ApprovalPolicy,
    /// 沙箱策略
    pub sandbox_profile: SandboxProfile,
    /// 网络策略
    pub network_policy: NetworkPolicy,
    /// 用户说明
    pub justification: Option<String>,
}
