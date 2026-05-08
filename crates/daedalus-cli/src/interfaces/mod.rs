//! 输入输出层。
//!
//! 本层只负责把用户或 Agent 的输入转换为 application use case 调用，
//! 并把结果展示成稳定文本、JSON 或 TUI。

/// 面向 Agent 和自动化脚本的 CLI。
pub mod agent_cli;
/// 面向人类操作者的 TUI。
pub mod tui;
