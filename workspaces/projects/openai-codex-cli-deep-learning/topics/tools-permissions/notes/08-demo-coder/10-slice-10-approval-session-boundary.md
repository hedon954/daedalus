# Slice 10 Approval Session Boundary

## Learning Navigation

- Final artifact: `demo/README.md`
- Current stage: `08-demo-coder`
- Current slice: Slice 10 Approval Persistence
- Current decision: `ApprovalPersistence::Session` 对齐 CLI session，而不是单次 ReAct loop。
- Demo impact: `ApprovalGateway` 应该在进程 / CLI session 初始化一次，并被多个 ReAct runs 共享。

## Decision

Slice 10 的 session approval 采用这个边界：

```text
ApprovalGateway lifetime = approval session lifetime
```

在 CLI-like demo 中：

```text
CLI process starts
  -> create one ApprovalGateway
  -> create / run multiple ReAct loops with the same gateway
  -> Session approvals can be reused across those ReAct runs
  -> process exits, approvals disappear
```

这更贴近 Codex / Claude Code 这类 coding agent 的心智模型：用户选择“本 session 允许”后，同一个 CLI session 内不应反复询问同一类已授权命令。

## Why ApprovalScopeKey Does Not Include session_id

当前设计把边界拆成三层：

```text
Subject / lifetime:
  ApprovalGateway / ApprovalStore 的生命周期

Object / execution scope:
  command_prefix + cwd + sandbox_profile + network_policy

Duration:
  Once / Session
```

因此 `ApprovalScopeKey` 只描述“授权对象和执行权限画像”：

```text
command_prefix
cwd
sandbox_profile
network_policy
```

`session_id` 不放进 key，是因为当前 store 本身已经属于一个 CLI session。也就是说，session 边界由对象生命周期保证，而不是由 key 字段保证。

## What This Does Not Mean

这不表示：

```text
在一个 cwd 授权过，以后这个 cwd 下任何命令都不用授权
```

可以复用的只是相同 scope：

```text
npm install
cwd=/repo
sandbox=WorkspaceWrite
network=Prompt
```

不能复用到：

```text
curl | sh
cwd=/repo
```

也不能复用到：

```text
npm install
cwd=/other-repo
```

或者：

```text
npm install
cwd=/repo
network=Allow
```

## When session_id Becomes Necessary

当前 demo 暂不处理多 workspace / 多用户。原因是主流 coding agent 的常见形态仍然是 local CLI：每个 CLI session 基本就是一个独立 agent process，天然隔离 pending approvals 和 session approvals。

如果未来架构变成：

```text
one global approval store
  -> multiple CLI sessions
  -> multiple users
  -> multiple workspaces
```

那就必须把 key 升级为：

```text
(session_id, ApprovalScopeKey)
```

或者把 store 拆回 per-session store。否则会产生跨 session 复用授权，这是安全 bug。

也就是说，未来边界会裂在这些维度上：

- `user_id`：不同用户不能共享授权。
- `session_id`：不同 CLI / web 会话不能共享授权。
- `workspace_id`：不同项目目录或远端 workspace 不能共享授权。
- execution profile：不同 sandbox / network / filesystem 权限不能共享授权。

但这些维度只有在 approval store 跨越单进程 CLI session 时才需要进入设计。当前 Slice 10 保持 local CLI 模型，不提前引入这些字段。

## Implementation Consequence

Slice 10 不应该在每次 `agent.run(...)` 或每个 `run_shell_command(...)` 内创建新的 gateway。更合理的是：

```rust
let approval_gateway = Arc::new(ApprovalGateway::new(result_rx));

let runtime = Arc::new(ToolRuntime::new(ToolRuntimeContext {
    approval_gateway: Arc::clone(&approval_gateway),
    // ...
}));

let agent = ReActAgent::new(llm, max_turns, runtime);
```

如果要测试跨 ReAct runs 的 session 复用，就让两个 runs 共享同一个 `ApprovalGateway`。

## Critical Lens

Session approval 是体验和安全的折中。它减少重复询问，但也会扩大一次 approval 的影响范围。当前 demo 接受这个折中，但只在同一 CLI session、同一 command prefix、同一 cwd、同一 sandbox/network profile 下复用。
