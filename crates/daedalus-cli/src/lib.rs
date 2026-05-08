//! daedalus 的确定性工具库。
//!
//! 该 crate 将学习任务初始化、状态流转、状态渲染和 workspace 校验封装为
//! 可测试的 Rust 逻辑，供 Agent-friendly CLI 与 Human-friendly TUI 共享。

/// 应用层 use case，负责编排领域规则和基础设施能力。
pub mod application;
/// 领域层类型和业务错误，不依赖文件系统或终端 UI。
pub mod domain;
/// 基础设施适配器，负责文件系统、模板、TOML 和时间来源。
pub mod infrastructure;
/// 输入输出层，包含 Agent CLI 和 TUI 的展示逻辑。
pub mod interfaces;
