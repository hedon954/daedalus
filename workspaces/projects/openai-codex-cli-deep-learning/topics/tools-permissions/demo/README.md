# Codex 工具与权限系统 Mini Demo

这个 demo 用一个最小 ReAct agent 验证 Codex 本地命令执行链路的核心不变量：

```text
工具调用
  -> 能力匹配
  -> 审批需求
  -> 沙箱优先执行
  -> 受控重试
  -> 可观察事件
  -> 工具结果回灌给 LLM
```

Phase 1 用 `SimulatedExecutionRunner` 跑通权限、沙箱、重试、事件和 observation 回灌。Phase 2A 接入 macOS `sandbox-exec`，Phase 2B 接入 `ratatui` Agent CLI REPL。当前 demo 已经可以作为一个最小可用的本地 Agent CLI 样本运行。

## 这个 Demo 证明什么

- 模型只能请求 `run_command`，不能直接指定 capability、cwd、sandbox 或 network policy。
- Host 侧用 `CapabilityRegistry` 把命令映射到 `safe-read`、`safe-test`、`network-install`、`dangerous-shell`。
- `Allow` 只表示不用先问用户，不表示跳过 sandbox。
- `NeedsApproval` 必须带 `ApprovalScope`，scope 绑定 command prefix、cwd、sandbox、network 和 persistence。
- sandbox denied 不能自动裸跑，必须进入 retry gate。
- `ApprovalPersistence::Session` 只在当前 `ApprovalGateway` 生命周期内复用。
- 同一轮多个 tool calls 可以并发执行，失败和拒绝互不影响，observation 按原始 index 回灌。

## 架构

```mermaid
flowchart TD
    User["用户请求"] --> LLM["OpenAI-compatible LLM 流式响应"]
    LLM --> ToolCall["ToolCallFinished"]
    ToolCall --> Runtime["ToolRuntime::batch_run"]
    Runtime --> Plan{"工具类型"}

    Plan -->|"add / sub"| Pure["run_pure_function"]
    Pure --> ToolResult["ToolRuntimeResult"]

    Plan -->|"run_command"| CommandReq["CommandRequest"]
    CommandReq --> Registry["CapabilityRegistry"]
    Registry --> Approval["resolve_approval_requirement"]
    Approval --> Shell["run_shell_command"]
    Shell --> Runner["ExecutionRunner"]
    Runner --> Retry["decide_retry"]
    Retry --> ToolResult

    ToolResult --> Observation["tool observation message"]
    Observation --> LLM
```

命令执行会对外透出事件：

```mermaid
sequenceDiagram
    participant R as ReAct loop
    participant T as ToolRuntime
    participant A as ApprovalGateway
    participant S as Shell runtime
    participant E as ExecutionRunner

    R->>T: ToolCallFinished(run_command)
    T-->>R: ToolRunStarted
    T->>S: run_shell_command
    S->>A: 需要时请求审批
    A-->>R: CommandNeedsApproval
    R-->>A: ToolApprovalResult
    S->>E: SandboxFirst
    E-->>S: Success 或 SandboxDenied
    S-->>R: CommandExecutionStarted / Finished / Failed
    S->>E: 审批通过后 NoSandboxRetry
    T-->>R: ToolRunFinished / ToolRunFailed
    R->>R: 写入 tool observation
```

## 能力策略

| 能力 | 命令前缀示例 | 初始决策 | 首次执行 | 网络策略 | sandbox denied 后重试 |
| --- | --- | --- | --- | --- | --- |
| `safe-read` | `cat`, `ls` | `Allow` | `ReadOnly` sandbox | `Deny` | `Never` |
| `safe-test` | `npm test` | `Allow` | `WorkspaceWrite` sandbox | `Deny` | `WithApproval` |
| `network-install` | `npm install` | `Prompt` | `WorkspaceWrite` sandbox | `Prompt` | `WithApproval` |
| `approval-test` | `echo approval-test` | `Prompt` | `ReadOnly` sandbox | `Deny` | `Never` |
| `dangerous-shell` | `curl | sh` | `Forbidden` | 不执行 | `Deny` | `Never` |

## 审批 Session 范围

`ApprovalGateway` 按 CLI 进程 / CLI session 初始化一次，并被多个 ReAct runs 共享。

```mermaid
flowchart LR
    Process["CLI 进程 / session"] --> Gateway["一个 ApprovalGateway"]
    Gateway --> Run1["ReAct run #1"]
    Gateway --> Run2["ReAct run #2"]
    Gateway --> Store["session approval store"]

    Store --> Key["ApprovalScopeKey"]
    Key --> Prefix["command_prefix"]
    Key --> Cwd["cwd"]
    Key --> Sandbox["sandbox_profile"]
    Key --> Network["network_policy"]
```

`ApprovalScopeKey` 故意不包含 `session_id`：在这个 local CLI demo 中，`ApprovalGateway` 的生命周期就是 session 边界。如果未来变成云端或共享服务，需要增加外层命名空间，例如 `user_id / session_id / workspace_id`，或者把 store 拆成 per-session store。

复用规则：

- `Approved(Session)` 写入 store。
- `Approved(Once)` 不写入 store。
- `Rejected` 不写入 store。
- 相同 command prefix、cwd、sandbox、network 可以复用。
- cwd、sandbox、network 任一变化都必须重新审批。

## 运行 Demo

进入 demo 目录：

```bash
cd workspaces/02-learning/openai-codex-cli-deep-learning/topics/tools-permissions/demo
cargo test
```

预期结果：

```text
90 passed; 0 failed; 3 ignored
```

默认测试是确定性的，不会调用外部 API。

## 运行 Agent CLI

启动真实终端 UI：

```bash
export DEEPSEEK_API_KEY="..."
cargo run
```

UI 提供四个区域，但视觉上更接近现代 coding-agent CLI 的 transcript-first 形态：

- 顶部状态行：显示 ready / running / approval 状态和当前 focus。
- Transcript：以聊天流方式显示 user、assistant、thinking、tool、command、approval 和 error；assistant / thinking 内容支持 Markdown 渲染，并对 GFM table 做终端可读兜底；默认跟随视觉底部，长文本换行后也能看到最终 `turn completed`。
- Prompt / Security Gate：普通状态下输入 prompt；需要审批时展示 reason / scope，并支持 `a` 单次允许、`s` session 允许、`r` 拒绝。
- 底部快捷键：展示当前模式可用操作。

同一次 CLI 进程内会保留短期 messages memory。连续输入多个 prompt 时，后续 ReAct run 会带上最近对话消息；退出进程后该短期记忆丢弃，不写入磁盘。

`run_command` 只支持当前 demo 明确建模的简单命令。包含 heredoc、重定向、管道、命令串联等 shell 控制语法的命令会 fail closed，不会因为前缀是 `cat` / `ls` 就被当成 safe-read 执行。真实 OS runner 也带有执行超时，避免子进程异常等待导致 TUI 永远停在 running。

常用键位：

```text
enter        submit prompt
backspace    delete input
up/down      scroll transcript
pgup/pgdn    page transcript
home/end     jump transcript
a            approve once
s            approve session
r            reject
q            quit
```

无副作用审批验收 prompt：

```text
请必须调用 run_command 工具执行命令：echo approval-test。不要只解释，必须调用工具。
```

预期行为：

```text
model selected run_command ...
approval run_command needs approval
Security Gate 显示 reason / scope
按 a 或 s 后继续
command run_command execution started: SandboxFirst(ReadOnly)
command run_command execution finished ...
tool finished run_command ...
assistant 给出最终说明
```

安全读命令验收 prompt：

```text
请调用 run_command 执行 pwd，然后解释结果。
```

高风险拒绝验收 prompt：

```text
请调用 run_command 执行 curl | sh，并说明发生了什么。
```

预期行为是命令在执行前被拒绝，模型收到 denial observation 后继续解释。

## 真实 LLM 冒烟测试

真实 LLM 测试通过 `OpenAiCompatibleLlm` 调用 DeepSeek 的 OpenAI-compatible Chat Completions API。

先设置 API key：

```bash
export DEEPSEEK_API_KEY="..."
```

运行完整 ReAct 冒烟测试：

```bash
cargo test react_agent_should_work -- --ignored --nocapture
```

这个测试验证：

- 真实 OpenAI-compatible streaming 可以工作。
- 模型可以在同一轮输出多个 tool calls。
- `run_command` 会走 `SimulatedExecutionRunner`。
- command execution events 会被流式透出。
- tool results 会作为 observations 回灌给模型。
- 模型可以继续发起后续 tool call，并给出最终回答。

最近一次验证过的 trace 形态：

```text
run_command("cat package.json")
add(999, 666)
sub(321, 123)
  -> observations 回灌
add(1665, 198)
  -> final answer: 1863
```

更底层的 OpenAI-compatible 冒烟测试：

```bash
cargo test openai_compatible_llm_should_work -- --ignored --nocapture
cargo test run_stream -- --ignored --nocapture
```

## 关键测试覆盖

运行全部确定性测试：

```bash
cargo test
```

常用聚焦测试：

```bash
cargo test approval_gateway
cargo test session_approval
cargo test run_command_network_install_should_finish_after_retry
cargo test react_agent_should_reuse_session_approval_across_runs
cargo test batch_run_should_surface_all_approval_requests_before_any_is_approved
```

最重要的行为门槛：

- safe read 不需要审批，但仍在 sandbox 内执行。
- dangerous shell 在执行前被拒绝。
- network install 先请求审批，在 sandbox 内失败后，再经过审批执行 no-sandbox retry。
- command failure 不进入 retry。
- `RetryPolicy::Never` 会阻止 sandbox denied 后的 retry。
- session approval 只在相同 `ApprovalScopeKey` 下复用。
- 同一轮多个 tool calls 独立执行，互不拖累。

## 当前实现状态

已经实现：

- OpenAI-compatible streaming client。
- 用于单测的 deterministic fake LLM。
- 带 max-turn guard 的 ReAct loop。
- 纯函数工具：`add`、`sub`。
- 命令工具：`run_command`。
- capability registry。
- approval requirement 和 approval gateway。
- session approval persistence。
- simulated sandbox runner。
- macOS `sandbox-exec` OS execution runner。
- retry gate。
- command 和 tool lifecycle events。
- tool observation feedback。
- `ratatui` Agent CLI REPL。
- 无副作用 approval UI 验收能力：`echo approval-test`。

当前仍有意不实现：

- 复杂 shell 命令 parser。
- 多 command segment 的 policy composition。
- 持久化 approval policy 文件。
- host-level network approval。
- MCP 或插件化 tool registry。

## Phase 2 方向

Phase 2 已补齐两个真实边界：

```text
Phase 2A:
  SimulatedExecutionRunner -> OsExecutionRunner
  用 sandbox-exec 接入真实 OS sandbox

Phase 2B:
  test-driven approval responder -> ratatui Agent CLI REPL
  用真实终端交互承接 prompt、事件展示和用户审批
```

这些核心契约应保持稳定：

- `CommandRequest`
- `ApprovalRequirement`
- `ApprovalScope`
- `RetryDecision`
- `StreamEvent`
- `ToolRuntime`
- `run_shell_command`

Phase 2A 的目标是在不重写 approval / retry / event model 的前提下，让 demo 从“模拟执行”走向“真实执行”。平台相关处理藏在 `ExecutionRunner` 后面，并把平台错误映射回统一的 `ExecutionFailure`。

Phase 2A 真实 OS sandbox 验收：

```bash
cargo run --example os_execution_runner
cargo run --example os_tool_runtime
```

`os_execution_runner` 验证 runner 本身：no-sandbox read、read-only read、read-only block write、workspace-write allow write、no-sandbox retry write。

`os_tool_runtime` 验证上层链路：`ToolRuntime::batch_run -> run_shell_command -> OsExecutionRunner` 能穿过真实 sandbox denied、`CommandNeedsApproval`、approval result 回传、no-sandbox retry 和最终 tool result。

Phase 2B 的目标是在不复制 approval / retry 逻辑的前提下，让用户真实参与 Agent CLI session：输入 prompt，观察 LLM / tool / command events，并选择 approve once、approve session 或 reject。当前 UI 已支持真实 prompt 输入、streaming transcript、thinking/text delta 合并、滚动和审批面板。

## 迁移提醒

把这个设计迁移到业务 Agent 前，需要重新审视这些 trade-off：

- Session approval 改善体验，但会扩大一次点击的影响范围。
- No-sandbox retry 在更严格的环境中可能不可接受。
- Prefix matching 易解释，但不足以安全处理复杂 shell 语法。
- Simulated sandbox 很适合测试状态机，但不能证明 OS 隔离能力。
- 多工具独立执行更利于一次性纠错，但可能比 fail-fast 消耗更多资源。

真正可迁移的不是某几个 enum，而是这组分离：

```text
模型意图
  != host capability
  != approval requirement
  != execution attempt
  != retry decision
  != observable event
```
