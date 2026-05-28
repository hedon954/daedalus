# Slice 6 Approval Sandbox Retry Integration

这份 guide 服务下一轮 `08-demo-coder`：把已经独立完成的 approval、simulated sandbox runner、retry gate 接入 ReAct tool execution path。

## Learning Navigation

- Final artifact: [`../../demo/README.md`](../../demo/README.md)
- Current stage: `08-demo-coder`
- Current slice: Slice 6 Agent Orchestrator
- Current gap: ReAct loop 现在可以真实调用工具，但还没有保留 Codex 的权限/沙箱不变量
- Evidence already available:
  - [`../../notes/06-code-reader/runtime-request-assembly.md`](../../notes/06-code-reader/runtime-request-assembly.md)
  - [`../../notes/06-code-reader/orchestrator-retry.md`](../../notes/06-code-reader/orchestrator-retry.md)
  - [`../../demo/design.md`](../../demo/design.md)
- After this: Slice 6 达到 Phase 1 主链路验收，可以进入 Slice 7 README / Runbook

## North Star

下一步不是继续增强 LLM，而是把 tool execution 从“直接运行工具”改成“受权限和沙箱编排保护的运行”：

```text
ToolCallFinished
  -> build CommandRequest / ToolRequest
  -> match capability
  -> decide_approval
  -> sandbox first run
  -> if SandboxDenied: decide_retry
  -> optional no-sandbox retry
  -> ToolRunFinished / ToolRunFailed
  -> role=tool observation
```

## Design Questions

开始写代码前，先让用户回答这 3 个问题：

1. 当前 `add/sub` 是计算工具，不是本地命令。它们应该先临时保留为 `safe-test` 风格的 demo tool，还是新增一个 `shell` / `command` tool 来承载 approval/sandbox/retry？
2. `CommandRequest` 应该在哪里构造：在 `react.rs` 的 `run_tools` 里直接构造，还是拆出一个 `tool_runtime` / `orchestrator` helper？
3. 一个 LLM response 里多个 tool call 时，如果前一个 tool 被 forbidden 或 approval rejected，后续 tool 是继续执行、全部停止，还是作为 Phase 1 明确 non-goal？

## Recommended Path

建议采用最小可验收路径：

1. 新增 `CommandToolRuntime` 或等价 helper，不直接塞爆 `react.rs`。
2. 先支持一个命令型 tool，例如 `shell` / `run_command`，参数最小为：

```json
{
  "argv": ["npm", "test"],
  "cwd": "/workspace"
}
```

3. `add/sub` 继续作为 pure tools，暂时不走 approval / sandbox / retry；命令型 tool 走完整安全链路。
4. 命令型 tool 的输出统一映射为现有 `ToolRunFinished / ToolRunFailed`，不要先扩张过多事件。
5. Phase 1 先顺序执行 tool calls；并发带副作用 tool 进入 non-goal。

## Action Card

### 1. Define Tool Runtime Boundary

目标：让 `react.rs` 保持编排清晰。

候选接口：

```rust
fn run_tool_with_policy(call: &ToolCallFinished, context: ToolRuntimeContext) -> ToolRuntimeResult
```

或：

```rust
struct ToolRuntime { ... }

impl ToolRuntime {
    fn run(&self, call: &ToolCallFinished) -> ToolRuntimeResult
}
```

验收：

- `react.rs` 不直接知道 approval / sandbox / retry 的全部细节。
- 测试能单独验证 forbidden、sandbox denied retry、command failed no retry。

### 2. Map Tool Call To Command Request

目标：把模型 tool call 参数转成已有领域模型。

需要决定：

- command argv 从哪里来。
- cwd 用测试 fixture 固定值，还是来自 runtime context。
- approval policy、sandbox profile、network policy 如何注入。
- capability 由 registry 匹配得出，不由模型自报。

验收：

- unknown / dangerous command 不能绕过 capability matching。
- `CommandRequest` 不重复存储已经能从 capability 派生的策略，除非这是 runtime context。

### 3. Insert Approval Gate

目标：tool call 在执行前必须经过 approval decision。

验收：

- `Forbidden`：不执行 runner，返回 tool failed observation。
- `NeedsApproval`：Phase 1 可以先用 scripted approval state 表达 accepted / rejected。
- `Skip { bypass_sandbox: false }`：仍然 sandbox first。
- `Skip { bypass_sandbox: true }`：直接 no-sandbox attempt。

### 4. Insert Sandbox First And Retry

目标：复用 Slice 4 / Slice 5 的实现。

验收：

- `CommandFailed` 直接失败，不 retry。
- `SandboxDenied + retry not allowed` 直接失败。
- `SandboxDenied + RetryWithApproval rejected` 失败。
- `SandboxDenied + RetryWithApproval accepted` 进行 no-sandbox retry。
- 单个 tool call 最多一次 sandbox first + 一次 no-sandbox retry。

### 5. Extend Fake LLM Tests

目标：用 deterministic tests 验证主链路，不依赖真实模型和网络。

建议新增：

- model proposes command tool -> approval skip -> sandbox success -> observation。
- model proposes dangerous command -> forbidden -> runner not called。
- sandbox denied -> retry approval accepted -> no-sandbox success。
- command failed -> no retry。

## Stop Rules

- 不做真实 OS sandbox。
- 不做真实用户审批 UI。
- 不做 tool 并发执行。
- 不把 `add/sub` 强行伪装成本地命令；如果需要安全链路，新增命令型 tool。
- 不扩展多 provider message 抽象。

## Completion Criteria

- [ ] ReAct loop 中至少有一个命令型 tool 走完整 approval / sandbox / retry 链路。
- [ ] fake LLM tests 覆盖 forbidden、sandbox success、sandbox denied retry、command failed no retry。
- [ ] `cargo test` 默认不访问网络。
- [ ] event stream 能解释 tool 为什么没执行、为什么 retry、最终 observation 是什么。
- [ ] `.daedalus/outcome-map.md` 和 `.daedalus/todo.md` 同步 Slice 6 integration 状态。
