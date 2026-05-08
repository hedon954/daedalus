//! 应用层 use case。
//!
//! 本层负责编排领域规则和基础设施能力，是 CLI 与 TUI 共享业务逻辑的入口。

/// 初始化学习任务 workspace。
pub mod init_task;
/// 从 `state.toml` 渲染 Agent 友好的 `state.md`。
pub mod render;
/// 执行学习阶段状态流转。
pub mod transition_stage;
/// 校验学习任务 workspace 的一致性。
pub mod validate_workspace;
