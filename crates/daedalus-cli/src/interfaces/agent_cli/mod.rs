//! Agent-friendly CLI 接口。

/// clap 命令行参数定义。
pub mod args;
/// Agent CLI 命令实现。
pub mod commands;
/// 命令执行上下文。
pub mod context;
/// trait + enum_dispatch 命令执行入口。
pub mod executor;
/// 稳定文本和 JSON 输出。
pub mod presenter;
