use crate::model::approval::{NetworkPolicy, SandboxProfile};

/// 命令能力类型。
///
/// 它不是模型直接提供的字段，而是 host 根据命令 argv 匹配出来的安全分类。
/// 后续 approval / sandbox / retry 都以这个分类作为策略入口。
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

/// 命令能力描述。
///
/// `CapabilityDescriptor` 是“支持哪些命令”的配置单元：它把一组命令前缀
/// 绑定到一个能力类型和默认执行策略。模型只能请求 `run_command`，
/// 不能直接指定 capability。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityDescriptor {
    /// 命令能力类型
    pub kind: CapabilityKind,
    /// 命令名称
    pub name: String,
    /// 命令描述
    pub description: String,
    /// 可匹配的命令前缀。
    ///
    /// 例如 `["npm", "install"]` 能匹配 `npm install vite`。
    pub command_prefixes: Vec<Vec<String>>,
    /// 命令能力策略
    pub policy: CapabilityPolicy,
}

/// 命令能力策略。
///
/// 这是 capability 自身的默认策略；运行时仍需要和 `CommandRequest`
/// 中的全局审批策略、cwd、网络策略等上下文合并判断。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityPolicy {
    /// 默认决策
    pub default_decision: DefaultDecision,
    /// 首次执行应使用的沙箱 profile。
    pub first_attempt_sandbox: SandboxProfile,
    /// 网络策略
    pub network_policy: NetworkPolicy,
    /// 重试策略
    pub retry_policy: RetryPolicy,
}

/// capability 在没有更多上下文时的默认决策。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum DefaultDecision {
    /// 允许
    Allow,
    /// 提示
    Prompt,
    /// 禁止
    Forbidden,
}

/// capability 级别的重试策略。
///
/// 它只在 sandbox denied 后参与 retry gate：先判断这类 capability 是否有 retry
/// 资格，再和全局 `ApprovalPolicy`、请求中的 `NetworkPolicy` 合成最终 retry 决策。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum RetryPolicy {
    /// 不重试
    Never,
    /// 可请求批准重试
    WithApproval,
    /// 可无请求批准重试
    WithoutApproval,
}
