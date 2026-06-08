use std::path::PathBuf;

/// 工具权限批准策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApprovalPolicy {
    /// 永远不要问用户。
    /// 如果命令需要审批，或者 sandbox 失败后想提权重试，系统都不能弹审批。
    /// 适合自动化、CI、无交互环境。
    Never,

    /// 先按默认权限执行。
    /// 如果命令本身需要审批，或者 sandbox 因权限不足失败，
    /// 系统可以请求用户批准，然后再继续或提权重试。
    /// 适合交互式开发环境。
    OnFailure,

    /// 只有调用方明确请求提权时，才允许问用户。
    /// 普通失败不会自动弹审批；需要有独立的提权信号，
    /// 例如 runtime request 标记 require_escalated / sandbox_permissions = RequireEscalated。
    /// justification 只是解释文本，不能作为提权信号。
    OnRequest,
}

/// 工具权限批准情况。
///
/// 这是 approval gate 给 command runtime 的编排指令：
/// - `Skip` 表示不需要用户确认，但不等于跳过沙箱。
/// - `NeedsApproval` 表示进入审批暂停点。
/// - `Forbidden` 表示当前策略直接拒绝。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ApprovalRequirement {
    /// 跳过审批
    Skip {
        /// 是否绕过沙箱
        bypass_sandbox: bool,
        /// 原因
        reason: String,
    },
    /// 需要审批
    NeedsApproval {
        /// 原因
        reason: String,
        /// 审批范围
        approval_scope: ApprovalScope,
    },
    /// 禁止执行
    Forbidden {
        /// 原因
        reason: String,
    },
}

/// 审批范围。
///
/// 用户一次“允许”不能只绑定到裸命令字符串，还必须绑定 cwd、sandbox、
/// network 和持久化粒度。这样才能避免把一次窄授权扩大成隐式全局授权。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApprovalScope {
    /// 命令前缀
    pub command_prefix: Vec<String>,
    /// 当前工作目录
    pub cwd: PathBuf,
    /// 沙箱策略
    pub sandbox_profile: SandboxProfile,
    /// 网络策略
    pub network_policy: NetworkPolicy,
    /// 持久化策略
    pub persistence: ApprovalPersistence,
}

/// 可复用审批的匹配 key。
///
/// 它只描述“授权对象和执行权限画像”，不包含 `persistence`。
/// `ApprovalPersistence::Session` 的边界由 `ApprovalGateway` / store 生命周期承载。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApprovalScopeKey {
    pub command_prefix: Vec<String>,
    pub cwd: PathBuf,
    pub sandbox_profile: SandboxProfile,
    pub network_policy: NetworkPolicy,
}

/// 沙箱策略
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum SandboxProfile {
    /// 只读
    ReadOnly,
    /// 工作区写入
    WorkspaceWrite,
    /// 无沙箱
    NoSandbox,
}

/// 网络策略
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum NetworkPolicy {
    /// 不允许
    Deny,
    /// 提示用户
    Prompt,
    /// 允许
    Allow,
}

/// 权限持久化策略
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum ApprovalPersistence {
    /// 只允许一次
    Once,
    /// 会话内有效
    Session,
}

impl ApprovalScope {
    pub fn key(&self) -> ApprovalScopeKey {
        ApprovalScopeKey {
            command_prefix: self.command_prefix.clone(),
            cwd: self.cwd.clone(),
            sandbox_profile: self.sandbox_profile,
            network_policy: self.network_policy,
        }
    }
}
