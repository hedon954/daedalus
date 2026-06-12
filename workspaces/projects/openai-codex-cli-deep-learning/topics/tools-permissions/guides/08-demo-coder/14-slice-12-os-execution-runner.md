# Slice 12 OsExecutionRunner With sandbox-exec

> Status: next action guide. Phase 2A 的目标是接入真实 OS sandbox，但不改变 Phase 1 已验证的 approval / retry / event 状态机。

## Learning Navigation

- Final artifact: [`../../demo/README.md`](../../demo/README.md)
- Current stage: `08-demo-coder`
- Current phase: Phase 2A
- Current slice: Slice 12 `OsExecutionRunner`
- Current gap: Phase 1 的 `SimulatedExecutionRunner` 只能证明状态机正确，不能证明真实 OS sandbox 接入后状态机仍成立。
- After this: 可以进入 Phase 2B，用 `ratatui` 做真实 Agent CLI REPL。

实现前先读：[`15-sandbox-first-principles.md`](15-sandbox-first-principles.md)。它解释 `sandbox-exec` 的底层原理，以及它和 namespace、seccomp、container、VM 等方案的第一性原理差异。

## North Star

这一 slice 只做一件事：

```text
SimulatedExecutionRunner
  -> OsExecutionRunner backed by sandbox-exec
```

保持不变：

```text
ToolRuntime
  -> run_shell_command
  -> resolve_approval_requirement
  -> request_approval
  -> run_execution_attempt
  -> decide_retry
```

也就是说，真实 OS sandbox 只是替换 `ExecutionRunner` 的一种实现，不应该让 approval / retry / event 逻辑泄漏到 runner 里。

## First Principles

从需求侧看，Phase 2A 要回答的问题不是“怎么把命令跑起来”，而是：

```text
当执行环境从模拟规则表变成真实 OS sandbox 后，
我们是否还能保持同一套安全状态机？
```

因此 `OsExecutionRunner` 的职责边界是：

- 接收 `CommandRequest + ExecutionAttempt`。
- 根据 attempt 决定是否用 `sandbox-exec`。
- 执行真实命令。
- 把真实退出结果映射回 `ExecutionResult`。
- 把 sandbox 拒绝映射成 `ExecutionFailure::SandboxDenied`。

它不负责：

- 判断命令是否允许。
- 向用户请求审批。
- 决定 sandbox denied 后是否重试。
- 发送 command lifecycle events。

这些仍然属于上层 runtime。

## Target Code Shape

建议新增文件：

```text
demo/src/tool/shell/execution/os_execution_runner.rs
```

并在：

```text
demo/src/tool/shell/execution/mod.rs
```

导出：

```rust
pub mod os_execution_runner;
```

核心类型：

```rust
pub struct OsExecutionRunner {
    sandbox_backend: SandboxBackend,
}

pub enum SandboxBackend {
    MacosSandboxExec,
}
```

如果你想先更简单，可以只写：

```rust
pub struct OsExecutionRunner;
```

等需要跨平台时再引入 `SandboxBackend`。

## ExecutionAttempt Mapping

`ExecutionAttempt` 到真实执行方式的映射：

```text
SandboxFirst { sandbox_profile }
  -> 用 sandbox-exec 包住命令执行

NoSandboxFirst { reason }
  -> 直接执行命令

NoSandboxRetry { reason }
  -> 直接执行命令
```

注意：`NoSandboxFirst` 和 `NoSandboxRetry` 都是不进 sandbox，但语义不同，事件层已经保留了 reason。runner 不需要重新解释 reason。

## sandbox-exec Profile

Phase 2A 先只支持 macOS `sandbox-exec`。本机已确认存在：

```text
/usr/bin/sandbox-exec
```

最小 profile 可以先内联生成：

```scheme
(version 1)
(deny default)
(allow process*)
(allow file-read*)
```

这适合 `SandboxProfile::ReadOnly`。

`SandboxProfile::WorkspaceWrite` 可以先保守处理：

```scheme
(version 1)
(deny default)
(allow process*)
(allow file-read*)
(allow file-write* (subpath "<cwd>"))
```

注意这里有一个现实约束：`sandbox-exec` profile 语法和 macOS 版本可能有差异，错误信息也不一定稳定。测试不要断言完整错误文本，只断言失败类别。

实现时还要守住三个硬边界：

- `cwd` 写进 profile 前要先 `canonicalize`，否则 `/var` / `/private/var` 这类路径别名会导致 `(subpath "...")` 匹配不上真实写入路径。
- profile 字符串里的路径要做 Scheme string 转义，至少处理 `\`、`"`、换行、回车和 tab。
- `ExecutionAttempt::SandboxFirst { sandbox_profile: NoSandbox }` 是调用方错误，应 fail closed，而不是偷偷降级成 read-only 或 no-sandbox。

## Command Execution

Phase 2A 可以继续沿用当前 `CommandRequest.argv`，用 `tokio::process::Command` 执行：

```rust
let mut command = Command::new(&request.argv[0]);
command.args(&request.argv[1..]);
command.current_dir(&request.cwd);
```

sandbox-first 时，不直接执行原命令，而是执行：

```text
sandbox-exec -p <profile> <argv...>
```

如果 profile 需要动态 cwd，可以生成临时 profile 文件，或者用 `-p` 传入字符串。第一版建议优先用 `-p`，降低文件生命周期复杂度。

## Rust 调用 sandbox-exec

第一版建议用 `tokio::process::Command`，因为当前 demo 的 runner trait 已经是 async。

整体形状：

```rust
use tokio::process::Command;

async fn run_os_command(
    request: &CommandRequest,
    attempt: &ExecutionAttempt,
) -> ExecutionResult {
    let output = match attempt {
        ExecutionAttempt::SandboxFirst { sandbox_profile } => {
            run_with_sandbox_exec(request, *sandbox_profile).await
        }
        ExecutionAttempt::NoSandboxFirst { .. } | ExecutionAttempt::NoSandboxRetry { .. } => {
            run_without_sandbox(request).await
        }
    };

    map_output(attempt, output)
}
```

### 1. 直接执行 no-sandbox 命令

```rust
async fn run_without_sandbox(request: &CommandRequest) -> std::io::Result<std::process::Output> {
    let Some(program) = request.argv.first() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty argv",
        ));
    };

    let mut command = Command::new(program);
    command.args(&request.argv[1..]);
    command.current_dir(&request.cwd);
    command.output().await
}
```

注意：

- 不要用 `shell -c <raw_command>` 作为默认执行方式，否则会把当前 Phase 1 的 argv 边界绕开。
- `raw_command` 只适合日志和错误信息；真实执行应优先使用 `argv`。
- 如果后续支持多命令或 shell 语法，再显式引入 command segment parser。

### 2. 用 sandbox-exec 执行 sandbox-first 命令

`sandbox-exec` 的命令形态是：

```text
sandbox-exec -p <profile-string> <program> <args...>
```

Rust 里可以这样组装：

```rust
async fn run_with_sandbox_exec(
    request: &CommandRequest,
    sandbox_profile: SandboxProfile,
) -> std::io::Result<std::process::Output> {
    let Some(program) = request.argv.first() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty argv",
        ));
    };

    let profile = sandbox_exec_profile(sandbox_profile, request);

    let mut command = Command::new("/usr/bin/sandbox-exec");
    command.arg("-p");
    command.arg(profile);
    command.arg(program);
    command.args(&request.argv[1..]);
    command.current_dir(&request.cwd);
    command.output().await
}
```

这里有一个容易踩的点：`current_dir` 是设置 `sandbox-exec` 这个进程的 cwd。子命令会继承这个 cwd，所以通常够用。

### 3. 生成 profile 字符串

最小版本：

```rust
fn sandbox_exec_profile(profile: SandboxProfile, request: &CommandRequest) -> String {
    match profile {
        SandboxProfile::ReadOnly => read_only_profile(),
        SandboxProfile::WorkspaceWrite => workspace_write_profile(&request.cwd),
        SandboxProfile::NoSandbox => read_only_profile(),
    }
}

fn read_only_profile() -> String {
    r#"
(version 1)
(deny default)
(allow process*)
(allow file-read*)
"#
    .to_string()
}

fn workspace_write_profile(cwd: &std::path::Path) -> String {
    format!(
        r#"
(version 1)
(deny default)
(allow process*)
(allow file-read*)
(allow file-write* (subpath "{}"))
"#,
        cwd.display()
    )
}
```

不过 `cwd.display()` 直接插入 profile 字符串有一个工程风险：路径里如果有引号或特殊字符，profile 字符串可能被破坏。第一版 demo 可以接受这个简化，但要在代码里留 TODO。更稳的做法是生成 profile 文件，并用 `-D` 参数传 path 变量，或者做 profile 字符串转义。

### 4. 映射 std::process::Output

`Command::output().await` 返回 `std::process::Output`：

```rust
pub struct Output {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
```

映射方式：

```rust
fn map_output(
    attempt: &ExecutionAttempt,
    output: std::io::Result<std::process::Output>,
) -> ExecutionResult {
    let output = match output {
        Ok(output) => output,
        Err(err) => {
            return ExecutionResult::Failure(ExecutionFailure::CommandFailed {
                exit_code: 127,
                stderr: err.to_string(),
            });
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        return ExecutionResult::Success { stdout };
    }

    let exit_code = output.status.code().unwrap_or(1);
    classify_failed_output(attempt, exit_code, stdout, stderr)
}
```

### 5. 分类 sandbox denied

第一版可以做保守分类：

```rust
fn classify_failed_output(
    attempt: &ExecutionAttempt,
    exit_code: i32,
    stdout: String,
    stderr: String,
) -> ExecutionResult {
    let combined = format!("{stdout}\n{stderr}");

    if matches!(attempt, ExecutionAttempt::SandboxFirst { .. })
        && looks_like_sandbox_denied(&combined)
    {
        return ExecutionResult::Failure(ExecutionFailure::SandboxDenied {
            output: combined,
            network_context: None,
        });
    }

    ExecutionResult::Failure(ExecutionFailure::CommandFailed {
        exit_code,
        stderr: combined,
    })
}

fn looks_like_sandbox_denied(output: &str) -> bool {
    let lower = output.to_ascii_lowercase();
    lower.contains("operation not permitted")
        || lower.contains("deny")
        || lower.contains("sandbox")
        || lower.contains("not permitted")
}
```

这个判断不是安全边界，只是错误分类启发式。真正的安全边界是 `sandbox-exec` profile 和 OS enforcement。测试应该断言：

```text
SandboxFirst write attempt -> Failure(SandboxDenied { .. }) 或至少非 Success
NoSandboxRetry same write attempt -> Success
```

不要断言完整 stderr。

### 6. 实现 ExecutionRunner

最后把这些函数包进 trait 实现：

```rust
#[async_trait::async_trait]
impl ExecutionRunner for OsExecutionRunner {
    async fn run(
        &self,
        request: &CommandRequest,
        attempt: &ExecutionAttempt,
    ) -> ExecutionResult {
        run_os_command(request, attempt).await
    }
}
```

如果你引入 `SandboxBackend`，则 `run_os_command` 可以变成 `self.run_os_command(...)`，由 backend 决定是否支持当前平台。

## Failure Mapping

真实执行结果要映射回现有领域模型：

```text
exit status == 0
  -> ExecutionResult::Success { stdout }

exit status != 0, attempt is SandboxFirst, stderr/output looks like sandbox denial
  -> ExecutionFailure::SandboxDenied { output, network_context: None }

exit status != 0
  -> ExecutionFailure::CommandFailed { exit_code, stderr }

spawn failed
  -> ExecutionFailure::CommandFailed { exit_code: 127, stderr }
```

第一版可以用较保守的 sandbox denied 判断：

```text
SandboxFirst + exit != 0 + stderr contains "Operation not permitted" / "deny" / "Sandbox"
```

但测试不要依赖具体 wording。可以把判断函数拆出来：

```rust
fn classify_failure(attempt, status, stderr) -> ExecutionFailure
```

这样后续可以替换成更严谨的策略。

## Tests To Write First

先写小测试，不要一上来改 ReAct live test。

### 1. no-sandbox success

```text
OsExecutionRunner
  request: cat Cargo.toml
  attempt: NoSandboxFirst
  -> Success
```

### 2. read-only sandbox success

```text
OsExecutionRunner
  request: cat Cargo.toml
  attempt: SandboxFirst(ReadOnly)
  -> Success
```

### 3. read-only sandbox blocks write

可以用一个不会破坏工作区的临时目录：

```text
request: sh -c "echo hi > blocked.txt"
attempt: SandboxFirst(ReadOnly)
-> SandboxDenied 或 CommandFailed
```

这里要小心：当前 Phase 1 command parser 还不支持复杂 shell parser，但 runner 级测试可以直接构造 `CommandRequest.argv = ["sh", "-c", "..."]`，不经过 capability registry。

### 4. no-sandbox retry succeeds

```text
same write command
attempt: NoSandboxRetry
-> Success
```

用临时目录，测试结束清理文件。

## Integration Test

runner 单测通过后，再写一个 `run_shell_command` 层 integration test：

```text
request: npm install 这种会触发 sandbox denied 的命令
runner: OsExecutionRunner
approval responder: approve retry
expected:
  SandboxFirst failed
  RetryWithApproval emitted
  NoSandboxRetry attempted
```

不过这个测试如果依赖真实 `npm install`，会慢且不稳定。第一版不要这么做。更好的做法是先通过 runner 级测试证明 OS sandbox 行为，再在 README 里保留手动验证命令。

## Manual Acceptance

手动验证命令：

```bash
cd workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo
cargo test os_execution_runner -- --nocapture
cargo run --example os_execution_runner
cargo run --example os_tool_runtime
```

如果接入后新增 ignored/manual test，可以写成：

```bash
cargo test os_execution_runner_manual -- --ignored --nocapture
```

`os_execution_runner` 只验证 runner 本身；`os_tool_runtime` 验证 `ToolRuntime::batch_run -> run_shell_command -> OsExecutionRunner` 可以穿过真实 sandbox denied、审批事件、approval result 和 no-sandbox retry。

## Critical Lens

`sandbox-exec` 是一个适合 mini demo 的真实边界，但不一定适合生产级 Agent：

- 它偏 macOS，不是跨平台方案。
- profile 语法和系统行为可能随版本变化。
- 网络控制未必能细到 host 级。
- 错误信息不一定稳定。

所以这一 slice 的目标不是宣称“sandbox-exec 就是最终安全方案”，而是验证：

```text
真实 sandbox runner 接入后，
approval / retry / event 状态机不用重写。
```

## Stop Rules

- 不做 ratatui UI；那是 Phase 2B。
- 不做跨平台 Linux / Windows sandbox。
- 不做完整 shell parser。
- 不把 `sandbox-exec` 错误文案当成稳定公共契约。
- 不修改 approval / retry / event 状态机，除非真实 runner 暴露出 Phase 1 设计缺陷。
