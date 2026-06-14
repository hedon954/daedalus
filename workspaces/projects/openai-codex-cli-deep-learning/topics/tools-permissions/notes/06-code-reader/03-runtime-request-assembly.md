# Runtime Request Assembly

状态：`用户已复述 / 源码已核对`

## Learning Navigation

- Final artifact: [`demo/design.md`](../../demo/design.md)
- Current gap: `CommandRequest` 字段来源
- Evidence needed: shell / unified exec 如何组装 command、cwd、approval policy、sandbox policy、sandbox permissions、additional permissions、approval key。

## User Question

> shell / unified exec 组装出来的 `CommandRequest`，哪些字段来自用户请求，哪些字段来自全局 config，哪些字段来自 runtime 自己的判断？

## User Answer

用户先对两个 runtime request 做了结构对比：

```text
ShellRequest：一次性执行完成的命令。
UnifiedExecRequest：进程级命令，允许 Codex 后续与进程多次交互和通信。
```

用户识别出的共同字段：

- `command`：当前执行的命令。
- `hook_command`：hook 使用的命令展示形式。
- `cwd`：当前目录。
- `env`、`environment`、`explicit_env_overrides`：环境变量信息。
- `network`：网络信息、代理等。
- `sandbox_permissions`、`additional_permissions`、`additional_permissions_preapproved`：沙箱和额外权限。
- `exec_approval_requirement`：权限判定结果，包含 `Skip / NeedsApproval / Forbidden`。
- `justification`：模型/用户给出的说明，审批 UI 会用。

用户识别出的差异：

- `ShellRequest` 因为是一次性返回，有 `timeout_ms`。
- `UnifiedExecRequest` 因为是进程级别，有 `process_id`、`tty` 等进程级配置。

用户初步按来源分为三类：

1. 用户自己配置的：当前命令、hook。
2. 全局配置：环境、网络权限、沙箱权限、文件权限、额外权限。
3. runtime 自己判断：cwd、tty、exec_approval_requirement、justification。

## Agent Calibration

整体方向是对的：`ShellRequest` 和 `UnifiedExecRequest` 的差异主要来自生命周期，一次性命令需要 timeout，常驻/可交互进程需要 process id、tty、environment 和后续 stdin 通道。

需要校准的是字段来源不能简单三分：

- `command` / `hook_command` 在 agent 工具链里主要来自模型 tool call 或 handler 解析后的 payload，不是用户手工配置。用户可以通过 prompt 间接影响它，但源码层的直接来源是 tool args / exec params。
- `cwd` 不是 runtime 自己随意判断。shell 路径使用 `exec_params.cwd`；unified exec 会先选择 tool environment，再把 `workdir` 相对环境 cwd 解析成最终 cwd。
- `approval_policy`、`permission_profile`、`file_system_sandbox_policy` 来自 turn/session config，是审批判断的全局上下文。
- `sandbox_permissions` 和 `additional_permissions` 通常从 tool args/request 进入，但会经过已授权 turn permissions、implicit permissions、normalize/validate 等处理，所以是“请求 + 会话已授权状态 + runtime 规范化”的结果。
- `justification` 来自 tool args / exec params，不是 runtime 自己判断；runtime 只是把它传给审批 UI、permission payload 或 guardian review。
- `exec_approval_requirement` 是 runtime/handler 在组装 request 前调用 `ExecPolicyManager::create_exec_approval_requirement_for_command` 得到的派生结果。
- `approval_key` 不在 request struct 字段中直接出现，而是在 runtime 的 `approval_keys(req)` 中从 request 规范化生成；shell 绑定 command、cwd、sandbox permissions、additional permissions，unified exec 还额外绑定 tty。

## Source Evidence

- `ShellRequest` 定义在 `core/src/tools/runtimes/shell.rs`，包含 command、hook command、cwd、timeout、env、network、sandbox permissions、additional permissions、justification、exec approval requirement。
- `ShellRuntime::approval_keys` 从 request 派生 approval cache key：canonical command、cwd、sandbox permissions、additional permissions。
- `run_exec_like` 在 `core/src/tools/handlers/shell.rs` 中组装 `ShellRequest`：从 `ExecParams`、turn/session context、permission normalization 和 exec policy evaluation 合并字段。
- `UnifiedExecRequest` 定义在 `core/src/tools/runtimes/unified_exec.rs`，比 shell 多 process id、environment、exec server env config、tty 等进程级字段。
- `UnifiedExecRuntime::approval_keys` 从 request 派生 approval cache key：canonical command、cwd、tty、sandbox permissions、additional permissions。
- `ExecCommandHandler` 在 `core/src/tools/handlers/unified_exec/exec_command.rs` 解析 `ExecCommandArgs`，选择 environment/cwd，生成 command、process_id、permissions，再交给 process manager。
- `UnifiedExecProcessManager::open_session_with_sandbox` 在 `core/src/unified_exec/process_manager.rs` 里最终组装 `UnifiedExecRequest` 并调用 orchestrator。

## Demo Delta

`CommandRequest` 不应只是命令字符串。demo 应拆成三层：

```text
ToolArgs / UserIntent
  -> CommandRequest
  -> ApprovalKey / ApprovalRequirement / ExecutionAttempt
```

建议 demo 的 `CommandRequest` 至少包含：

```text
command
hook_command
cwd
env
network
sandbox_permissions
additional_permissions
approval_policy
sandbox_policy
justification
timeout_ms or process_mode
tty when process_mode = interactive
```

字段来源分类：

| 字段 | 主要来源 | 说明 |
| --- | --- | --- |
| `command` | tool args / handler parsing | 模型工具调用进入，handler 转成 argv |
| `hook_command` | tool args / display form | hook、guardian、UI 使用 |
| `cwd` | tool environment + workdir / exec params | 不是自由推断，必须绑定执行边界 |
| `env` | tool args + session dependency env + shell env policy | 可能有显式覆盖 |
| `network` | turn/session environment | 决定网络审批和代理 |
| `sandbox_permissions` | tool args + granted turn permissions | 会被已授权权限和 runtime 规范化影响 |
| `additional_permissions` | tool args + granted turn permissions + validation | 决定额外文件/网络等权限粒度 |
| `approval_policy` | turn/session config | 决定能否问用户，以及 prompt 是否会被拒绝 |
| `sandbox_policy` | turn permission profile / filesystem sandbox policy | 决定默认隔离边界 |
| `justification` | tool args / exec params | runtime 传递给审批 UI |
| `exec_approval_requirement` | exec policy evaluation | 从 command + context 派生，不是原始输入 |
| `approval_key` | runtime derives from request | shell 不含 tty，unified exec 含 tty |
