//! 应用层 use case。
//!
//! 本层负责编排领域规则和基础设施能力，是 CLI 与 TUI 共享业务逻辑的入口。

/// 关闭学习任务并移动 workspace。
pub mod close_task;
/// IDE 派生配置同步。
pub mod ide;
/// 初始化学习任务 workspace。
pub mod init_task;
/// 知识库结构、模板、索引和校验底座。
pub mod knowledge;
/// workspace 迁移 use cases。
pub mod migrate;
/// 从 `state.toml` 渲染 Agent 友好的 `state.md`。
pub mod render;
/// 管理复习计划。
pub mod review;
/// 状态机流转 trait。
pub mod state_machine;
/// 管理 repo learning topics。
pub mod topic;
/// 执行学习阶段状态流转。
pub mod transition_stage;
/// 校验学习任务 workspace 的一致性。
pub mod validate_workspace;
