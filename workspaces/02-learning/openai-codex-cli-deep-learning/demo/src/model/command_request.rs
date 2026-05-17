use std::path::PathBuf;

use crate::model::approval::{ApprovalPolicy, NetworkPolicy, SandboxProfile};
use crate::model::capability::CapabilityKind;

/// 命令请求
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandRequest {
    /// 原始命令
    pub raw_command: String,
    /// 命令参数
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
