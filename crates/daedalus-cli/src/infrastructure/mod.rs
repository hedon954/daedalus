//! 基础设施适配器。
//!
//! 本层集中处理文件系统、模板复制、TOML 文档更新和时间来源。

/// 时间来源适配器。
pub mod clock;
/// `.daedalus/state.toml` 的读取、查询和局部更新。
pub mod state_toml;
/// 模板目录复制和占位符替换。
pub mod template_fs;
/// daedalus 项目根与学习 workspace 的文件系统发现。
pub mod workspace_fs;
