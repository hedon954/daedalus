# Slice 10 Approval Persistence

## Learning Navigation

- Final artifact: `demo/README.md`
- Current stage: `08-demo-coder`
- Current slice: Slice 10 Approval Persistence
- Current gap: approval result 已能通过 `approval_id + oneshot` 回流，但批准结果还不会形成 session 级复用。
- After this: 可以补 `demo/README.md`，用 trace 展示一次 approval 如何减少后续重复询问。

## North Star

用户点一次“允许本 session”，不能变成全局裸奔；它只能在相同 scope 下复用：

```text
command_prefix + cwd + sandbox_profile + network_policy
```

如果这些字段任一变化，旧 approval 都不能复用。

这里的 `Session` 对齐主流 coding agent 的 CLI session 心智模型：

```text
一个 CLI 进程 / CLI session
  -> 初始化一个 ApprovalGateway
  -> 多次 ReAct runs 共享同一个 ApprovalGateway
  -> Session approval 可跨多个 ReAct runs 复用
```

所以 `ApprovalPersistence::Session` 不是“单轮 ReAct loop 内有效”，也不是“写入磁盘后跨进程有效”。它表示：

```text
在当前 ApprovalGateway / ApprovalStore 生命周期内有效
```

当前 demo 推荐让 `ApprovalGateway` 在进程启动时创建一次，并传给多个 `ReActAgent` / `ToolRuntime` 使用。

## Current Code Boundary

当前相关代码：

- `model/approval.rs`
  - `ApprovalScope`
  - `ApprovalPersistence`
  - `ApprovalRequirement`
- `tool/shell/approval.rs`
  - `ApprovalGateway`
  - `PendingApproval`
  - `ToolApprovalRequest`
  - `ToolApprovalResult`
  - `resolve_approval_requirement`
  - `build_approval_scope`
- `tool/shell/mod.rs`
  - `request_approval`
  - `run_shell_command`

当前 `ApprovalGateway` 只管理 pending approval，不保存已批准的 session scope。

## Design Shape

最小实现可以把 session store 放进 `ApprovalGateway`：

```rust
pub struct ApprovalGateway {
    pending: Arc<DashMap<String, oneshot::Sender<UserApprovalDecision>>>,
    session_approvals: Arc<DashMap<ApprovalScopeKey, ApprovalGrant>>,
}
```

建议新增两个小类型：

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApprovalScopeKey {
    pub command_prefix: Vec<String>,
    pub cwd: PathBuf,
    pub sandbox_profile: SandboxProfile,
    pub network_policy: NetworkPolicy,
}

pub struct ApprovalGrant {
    pub persistence: ApprovalPersistence,
}
```

不要直接用 `ApprovalScope` 当 key，因为它里面有 `persistence`。复用判断应该比较“授权对象”，不是比较“这次希望怎么持久化”。

也不要把 `session_id` 放进 `ApprovalScopeKey`。当前设计假设 store 本身就是 CLI-session-scoped：

```text
ApprovalGateway lifetime = approval session boundary
ApprovalScopeKey = session 内部的授权匹配条件
```

只有当未来把多个 CLI sessions / 多用户 / 多 workspace 的 approvals 放进同一个全局 store 时，才需要把 key 升级为：

```text
(session_id, ApprovalScopeKey)
```

当前不为云端多租户提前设计。主流 coding agent 的 local CLI 形态通常是一个 CLI session 对应一个 agent process，`ApprovalGateway` 单例已经提供足够清晰的隔离边界。只有当 approval store 跨过当前进程，进入多用户 / 多 session / 多 workspace 的共享服务时，才把 `user_id`、`session_id`、`workspace_id` 作为外层命名空间引入。

## Implementation Steps

### Step 1: Extract Scope Key

给 `ApprovalScope` 增加一个转换方法：

```rust
impl ApprovalScope {
    pub fn key(&self) -> ApprovalScopeKey { ... }
}
```

测试：

- same prefix / cwd / sandbox / network -> same key。
- persistence 不同 -> key 仍相同。
- cwd 不同 -> key 不同。
- network policy 不同 -> key 不同。

### Step 2: Add Session Store

在 `ApprovalGateway` 增加：

```rust
pub fn remember_approval(&self, scope: &ApprovalScope, decision: &UserApprovalDecision)
pub fn has_session_approval(&self, scope: &ApprovalScope) -> bool
```

规则：

- `Approved { persistence: Session }` 才写入 store。
- `Approved { persistence: Once }` 不写入。
- `Rejected` 不写入。

创建位置：

- CLI-like demo：进程启动时创建一个 `Arc<ApprovalGateway>`，多次 `agent.run(...)` 共享。
- 单测可以直接复用同一个 `ApprovalGateway` 调两次 `run_shell_command`。
- 不要在每次 `run_shell_command` 或每个 tool call 内 new gateway，否则 session approval 无法跨 run 复用。

### Step 3: Reuse Before Prompt

在真正发 `CommandNeedsApproval` 之前先查：

```text
if approval_gateway.has_session_approval(scope) {
    return UserApprovalDecision::Approved { persistence: Session };
}
```

也就是说，复用 approval 是 approval gateway 的职责，不应该散落到 ReAct loop。

### Step 4: Remember After Approval

用户审批通过后：

```text
pending.wait()
  -> if Session, remember scope
  -> return decision
```

注意：`request_approval` 当前拿得到 `ToolApprovalRequest`，里面已有 scope，所以这里能完成记忆。

## Tests To Write First

优先写这些单测：

- `session_approval_reuses_same_scope_without_new_prompt`
- `once_approval_does_not_reuse`
- `rejected_approval_does_not_reuse`
- `session_approval_does_not_reuse_when_cwd_changes`
- `session_approval_does_not_reuse_when_network_policy_changes`

再补一条 shell integration test：

```text
same npm install command twice
  -> first call emits CommandNeedsApproval
  -> approve with Session
  -> second call does not emit CommandNeedsApproval
```

再补一条 lifecycle test：

```text
two ReAct runs share the same ApprovalGateway
  -> first run approves npm install with Session
  -> second run with same scope does not prompt again
```

## Critical Lens

Codex / Claude Code 的 session allow 很方便，本质是 CLI session 内的交互体验优化。这个 demo 要模仿“同一 CLI session 内复用授权”，同时保留强 scope 绑定，避免把它简化成“命令名允许一次以后永远允许”。

## Stop Rules

- 不做磁盘持久化；Slice 10 只做当前进程 / 当前 CLI session。
- 不做多用户 / 多 workspace approval store；那是云端部署模型，不属于当前 local CLI demo。
- 不做 UI。
- 不做 wildcard rule 编辑器。
- 不改 `ApprovalPolicy::OnRequest` escalation 语义；那是后续 hardening gap。
