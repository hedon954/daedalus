use crate::{model::command_request::CommandRequest, tool::shell::RunCommandResult};

/// 真实本地命令执行占位。
///
/// TODO: Phase 1 应优先通过 `SandboxRunner` 的 `NoSandboxFirst/NoSandboxRetry`
/// 模拟无沙箱执行；Phase 2 接入真实 OS runner 时再实现这里。
pub fn run_local_shell(request: &CommandRequest) -> RunCommandResult {
    let _ = request;
    todo!()
}
