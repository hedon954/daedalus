use crate::model::approval::{NetworkPolicy, SandboxProfile};

/// 命令能力类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CapabilityKind {
    /// 安全读命令，比如 `ls`, `cat package.json`
    SafeRead,
    /// 安全测试命令，比如 `npm test`, `cargo test`
    SafeTest,
    /// 需要网络或依赖写入，比如 `npm install`
    NetworkInstall,
    /// 高风险或不可安全抽象的命令，比如 `curl ... | sh`, `rm -rf`
    DangerousShell,
}

/// 命令能力描述
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityDescriptor {
    /// 命令能力类型
    pub kind: CapabilityKind,
    /// 命令名称
    pub name: String,
    /// 命令描述
    pub description: String,
    /// 命令前缀
    pub command_prefixes: Vec<Vec<String>>,
    /// 命令能力策略
    pub policy: CapabilityPolicy,
}

/// 命令能力策略
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityPolicy {
    /// 默认决策
    pub default_decision: DefaultDecision,
    /// 首次 sandbox
    pub first_attempt_sandbox: SandboxProfile,
    /// 网络策略
    pub network_policy: NetworkPolicy,
    /// 重试策略
    pub retry_policy: RetryPolicy,
}

/// 默认决策
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum DefaultDecision {
    /// 允许
    Allow,
    /// 提示
    Prompt,
    /// 禁止
    Forbidden,
}

/// 重试策略
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum RetryPolicy {
    /// 不重试
    Never,
    /// 可请求批准重试
    WithApproval,
    /// 可无请求批准重试
    WithoutApproval,
}
