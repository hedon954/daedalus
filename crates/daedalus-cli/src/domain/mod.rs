//! daedalus CLI 的领域模型。
//!
//! 这里放置和学习任务本身相关的类型，例如阶段、产物、状态流转和业务错误。
//! 本层不应该依赖文件系统、命令行参数或 TUI。

/// 学习产物相关领域类型。
pub mod artifact;
/// 领域错误和统一结果类型。
pub mod error;
/// 学习任务聚合的基础信息。
pub mod learning_task;
/// 学习阶段和阶段状态。
pub mod stage;
/// 状态流转记录。
pub mod transition;

pub use error::{ApprovalSource, DaedalusError, Result};
pub use learning_task::{TaskLifecycle, WorkspaceBucket};
pub use stage::{StageSnapshot, StageStatus};
