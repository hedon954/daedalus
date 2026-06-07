# Slice 9 Multi-Tool Independent Execution

> Status: historical action card. Slice 9 first implementation is now complete in `react.rs`.
> Current follow-up is structural cleanup: [`12-slice-9-tool-batch-runner-refactor.md`](12-slice-9-tool-batch-runner-refactor.md).

## Learning Navigation

- Final artifact: `demo/README.md`
- Current stage: `08-demo-coder`
- Current slice: Slice 9 Multi-Tool Independent Execution
- Current gap: 已完成第一版并行 `run_tools`；当前缺口是从 `agent/react.rs` 抽出 batch boundary，降低 ReAct loop 职责密度。
- Evidence needed: `react.rs` 多 tool 调度、`ToolRuntimeResult` 分支、fake LLM deterministic tests。
- After this: 可以进入 Slice 10 approval persistence，或先写 README trace 展示多 tool observation。

## North Star

把同一轮多个 tool call 的执行语义从“顺序跑 + 未来可能 skipped”改为：

```text
同批 tool_calls 互不影响
能并发执行就并发执行
每个 call 独立产生 Finished / Failed / Denied
最终按原始 index 回灌 observations
```

## Why This Step Matters

用户选择 independent observation policy 的原因是：如果一个失败就强制跳过后续工具，agent 下一轮纠正后还要重新发那些被跳过的 call；如果被跳过的 call 自己也失败，又会继续多轮纠错。全部执行并独立回传结果，可以让 agent 一次性看到完整错误面。

这不是盲目照抄 Codex，但和 Codex 的方向一致：并发能力由工具声明控制，失败作为模型可见 observation 返回，而不是 batch-level hard stop。

## Original Implementation Plan

以下是第一版实现时使用的行动计划。当前已完成，不要再把它当作下一步。

### Step 1: 删除旧 hard-deny TODO 的方向

先在 `agent/react.rs` 找到多 tool call loop 附近的 TODO。它现在表达的是：

```text
一个工具被安全拒绝后，后续依赖它的工具应标记为 Skipped
```

这已经不是当前设计。替换为：

```text
同批 tool calls 独立执行；后续如果支持并发能力声明，再按 tool metadata 决定并发/串行。
```

### Step 2: 引入结果包装类型

不要让并发 future 只返回 `ToolRuntimeResult`，否则回写 observation 时会丢掉 call metadata。建议局部定义：

```rust
struct ToolRunOutcome {
    call: ToolCallFinished,
    result: ToolRuntimeResult,
}
```

如果后续使用 `tokio::spawn` 或 `JoinSet`，需要额外处理 `JoinError`。`JoinError` 属于 task 层错误，应映射成该 tool call 的 `ToolRuntimeResult::Failed`，不要让整个 batch 直接失败。

### Step 3: 并发执行，稳定排序

第一版可以使用 `futures::future::join_all`，因为 demo 的同批 tool call 数量很小，且我们需要全部执行完成后统一按 index 写回：

```text
calls sorted by index
  -> futures = calls.map(runtime.run)
  -> outcomes = join_all(futures)
  -> outcomes.sort_by_key(call.index)
  -> append observations
```

如果后续需要边完成边 emit 更细事件，再考虑 `FuturesUnordered` 或 `JoinSet`。当前验收重点是 message 顺序和 observation 完整性。

### Step 4: 统一 observation mapping

把 `Finished / Failed / Denied / Skipped` 到 event + message 的重复逻辑提成小函数，避免并发改造时复制四段 match：

```text
emit_and_append_tool_observation(tx, messages, call, result)
```

当前策略下：

- `Finished` -> `ToolRunFinished` + normal tool message
- `Failed` -> `ToolRunFailed` + failed tool message
- `Denied` -> `ToolRunFailed` + denied tool message
- `Skipped` -> 如果暂时保留，也只作为显式 result 映射，不由 batch-level hard-deny 自动生成

### Step 5: 测试先行

优先补 fake LLM deterministic tests：

- 同批两个 pure tools 都成功，observations 按 index 写回。
- 同批 unknown tool 失败、`add` 成功，`add` 不被跳过。
- 同批 `run_command` denied、`add` 成功，`add` 不被跳过。
- 如果人为让先完成的 call index 更大，最终 message 顺序仍按 index。

## Acceptance

- `react.rs` 不再有 hard-deny skipped TODO。
- 同批 tool calls 中一个失败或被拒，不影响其他 call 执行。
- 每个 `tool_call_id` 都有 observation。
- observations 按原始 index 写回。
- `ToolRuntimeResult::Skipped` 当前已删除；不要再为 batch-level hard-deny 重新引入它。

## Tokio Runtime Reference

Tokio 的底层调度、等待、取消和 `join_all` / `JoinSet` 选择不放在本行动 guide 里展开，单独阅读：

- [`11-tokio-runtime-scheduling.md`](11-tokio-runtime-scheduling.md)

对当前 Slice 9 的实现结论：

- 先用 `join_all` 收集同批 tool outcomes，失败作为 `ToolRuntimeResult` 值返回，不使用 fail-fast 的 `try_join!`。
- 如果后续需要真正 spawned task、任务取消、完成即处理，再升级到 `JoinSet + Arc<ToolRuntime>`。
- event 可以按完成顺序发出，但回灌给 model 的 tool observations 应按原始 `index` 稳定排序。

## Stop Rules

- 不在 Slice 9 做 session approval persistence。
- 不在 Slice 9 做真实 OS sandbox。
- 不在 Slice 9 处理多命令 shell parser。
- 不因为“并发执行”就默认所有未来工具都安全并发；Phase 2 前需要补 tool-level `supports_parallel` 或等价策略。
