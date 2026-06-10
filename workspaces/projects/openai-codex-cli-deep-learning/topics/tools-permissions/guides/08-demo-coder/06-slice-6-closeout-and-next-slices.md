# Slice 6 Closeout And Next Slices

## Learning Navigation

- Final artifact: `demo/README.md` and Phase 1 acceptance evidence
- Current stage: `08-demo-coder`
- Current slice: Slice 6 closeout
- Current gap: runtime command path tests are now filled; remaining work is stage alignment and next-slice reshaping
- Evidence needed: current source, tests, TODOs, and `demo/design.md` acceptance table
- After this: we can close Slice 6 as Agent Orchestrator and move to policy/event hardening instead of prematurely writing README

## Current Implementation Evidence

Current code evidence:

- `agent/react.rs`: ReAct loop streams model output, collects tool calls, runs tools in index order, and feeds observations into the next model turn.
- `tool/runtime.rs`: `ToolRuntime::run` now covers pure function tools and `run_command`.
- `tool/shell/mod.rs`: `run_shell_command` wires approval, sandbox-first execution, retry decision, and no-sandbox retry.
- `tool/shell/execution/`: `ExecutionRunner` and `SimulatedExecutionRunner` replace the older top-level sandbox runner boundary.

Current validation:

```text
cargo test
54 passed, 3 ignored
```

Update after Slice 7:

```text
cargo test
61 passed, 3 ignored
```

New Slice 6 tests added at `tool/runtime.rs`:

- `run_command_safe_read_should_finish_with_output`
- `run_command_network_install_should_finish_after_retry`
- `run_command_command_failure_should_fail_without_pinning_error_text`
- `run_command_dangerous_shell_should_be_denied`
- `run_command_invalid_json_should_fail_without_pinning_error_text`
- `run_command_unmatched_capability_should_fail_without_pinning_error_text`

These tests close the previous gap between shell-level tests and ReAct-level observation tests.

## Slice 6 Exit Classification

### Required Before Exiting Slice 6

1. Sync stale guide wording with the current code: done for the stage README and design; older Slice 6 guides now carry historical status banners.
   - `SimulatedSandboxRunner` -> `SimulatedExecutionRunner`
   - `SandboxRunner` -> `ExecutionRunner`
   - `ToolRuntime WIP / command path pending` -> `ToolRuntime command path tested`
2. Update `.daedalus/todo.md` and `.daedalus/outcome-map.md` so they say Slice 6 is in closeout, not already fully handed off to README: done.
3. Record the new test evidence and validation result: done.
4. Decide the explicit non-goals for Slice 6:
   - no multi-command parser yet
   - no real OS sandbox yet
   - no session approval persistence yet
   - no full approval/sandbox/retry event protocol yet

### Optional Hardening Inside Slice 6

These are useful, but should not block Slice 6 exit:

- Rename or remove stale comments that still say `sandbox` when the abstraction is now `execution`.
- Add a small acceptance matrix that maps current tests to AT items.
- Add one more ReAct test for invalid `run_command` JSON if we want the model-observation layer to cover malformed command arguments.

### Deferred Non-Goals

These should become later slices:

- `RetryPolicy` integration into retry gate: completed in Slice 7.
- Multi-tool independent execution policy and stable observation ordering.
- Explicit `ToolRunDenied` / `ToolRunSkipped` stream events.
- Structured approval, sandbox, retry events.
- Real `OsExecutionRunner`.
- Robust shell parsing and command segment composition.

## Revised Next Slices

The old plan jumped from Slice 6 directly to README. Based on the current implementation, a better path is:

### Slice 7: Retry Policy And Denial Semantics

Goal: make retry behavior respect capability-level policy, not only global `ApprovalPolicy` and `NetworkPolicy`.

Status: completed.

Why it was needed:

- At Slice 6 closeout, `RetryPolicy` existed in the model but was not consumed by `decide_retry`.
- `safe-read` with sandbox denied should not silently become retry-with-approval if its capability says `RetryPolicy::Never`.

Target tests:

- `safe-read` sandbox denied -> `DoNotRetry`.
- `safe-test` sandbox denied -> `RetryWithApproval`.
- `network-install` network denied with prompt -> `RetryWithApproval`.
- retry is still at most once.

Unlocked:

- AT-09 becomes real instead of only documented.
- Retry semantics stop being accidentally global.

### Slice 8: Event Protocol Hardening

Goal: expose the major policy/execution decisions without leaking internal implementation details.

Minimum event additions:

- approval resolved or needs approval
- tool denied
- execution started with `ExecutionAttempt`
- retry evaluated

Decision to make:

- Whether `Denied` and `Skipped` should remain folded into `ToolRunFailed`, or become explicit stream events.

Unlocked:

- AT-01 to AT-04 become observable from the agent stream, not only from unit tests.
- README can explain the demo by showing an event trace.

### Slice 9: Multi-Tool Independent Execution

Goal: decide and implement what happens when one tool call in the same model turn fails or is rejected.

Current code:

- `ToolRuntimeResult::Skipped` has been removed.
- `ToolRuntime::batch_run` owns same-batch tool scheduling.
- `ToolEventEmitter` owns tool-level lifecycle events.
- `react.rs` owns transcript observations.

Target behavior:

- Same-batch tool calls are treated as independent observations.
- A failure or denial in one call does not cancel or skip the remaining calls.
- Execution may be concurrent, but observations are written back in original index order.

Unlocked:

- Complete multi-tool observation feedback in one model repair turn.
- Clearer boundary for Slice 10: approval persistence can be added below command runtime without changing ReAct batch semantics.

### Slice 10: Approval Persistence

Goal: implement session-level approval reuse and scope mismatch rejection.

Target tests:

- same command prefix + cwd + sandbox + network + session approval -> no repeated prompt.
- changed cwd or network policy -> cannot reuse approval.

Unlocked:

- AT-12 and AT-13.

### Slice 11: Phase 1 README And Runbook

Goal: write a README only after the Phase 1 behavior is stable enough to teach and run.

Contents:

- how to run tests
- how to run live LLM tests
- supported tools and commands
- event trace examples
- Phase 1 simplifications
- Phase 2 path to `OsExecutionRunner`

Unlocked:

- `08-demo-coder` can close with a recoverable, runnable artifact.

### Phase 2: OS Execution Runner

Goal: replace the simulated runner with a real execution runner while preserving the existing state machine.

Rule:

The OS runner may replace execution mechanics, but must not rewrite capability, approval, retry, or observation semantics.

## Recommended Next Action

Close Slice 6 with a small documentation sync, then start Slice 7:

```text
Slice 6 closeout
  -> sync guides / todo / outcome-map
  -> cargo test evidence recorded
  -> Slice 7 RetryPolicy integration
```
