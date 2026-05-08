use enum_dispatch::enum_dispatch;

use crate::domain::Result;
use crate::interfaces::agent_cli::commands::*;
use crate::interfaces::agent_cli::context::AgentCliContext;

/// Agent CLI 命令执行 trait。
///
/// 每个 clap 参数结构体自己实现执行逻辑，顶层和嵌套 enum 通过
/// `enum_dispatch` 自动分发，避免在 `main` 中集中维护大块 `match`。
#[allow(async_fn_in_trait)]
#[enum_dispatch]
pub trait CmdExecutor {
    /// 执行命令。
    async fn execute(self, ctx: AgentCliContext) -> Result<()>;
}
