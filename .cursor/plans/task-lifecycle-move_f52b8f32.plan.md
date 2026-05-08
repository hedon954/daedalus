---
name: task-lifecycle-move
overview: 为 daedalus CLI 补齐学习任务生命周期闭环：完成或放弃任务时自动从 active workspace 移动到 completed/abandoned，并在 Agent-friendly 输出中明确移动结果。
todos:
  - id: add-workspace-move-helpers
    content: 新增 completed/abandoned workspace 根目录与安全移动 helper。
    status: completed
  - id: add-close-task-usecase
    content: 实现 task complete/abandon 应用逻辑，并返回结构化移动结果。
    status: completed
  - id: add-task-cli-command
    content: 基于 CmdExecutor/enum_dispatch 新增 `daedalus task complete` 与 `daedalus task abandon`。
    status: completed
  - id: auto-close-final-stage
    content: 让成功的 `state complete 10-archivist` 复用任务关闭逻辑并报告移动结果。
    status: completed
  - id: update-output-and-docs
    content: 更新 presenter 输出、生命周期动作说明、模板和 Agent 指引。
    status: completed
  - id: add-lifecycle-tests
    content: 补充完成、放弃、目标目录冲突保护和 WIP 释放的集成测试。
    status: completed
  - id: verify-lifecycle
    content: 运行 fmt、make ci、lints，并在获批后做手工 smoke test。
    status: completed
isProject: false
---

# 学习任务生命周期移动方案

## 目标语义

- 普通阶段的完成语义保持不变，不移动目录。
- 通过 `daedalus state complete 10-archivist --task-dir <task>` 完成最终阶段时，应自动关闭学习任务，并将目录从 `workspaces/02-learning/<task>` 移动到 `workspaces/03-completed/<task>`。
- 新增显式 task-level 命令：
  - `daedalus task complete [task-dir] --reason <reason>`：校验最终闭环条件，更新/刷新状态，然后移动到 `workspaces/03-completed`。
  - `daedalus task abandon [task-dir] --reason <reason>`：要求具体放弃原因，追加生命周期流转记录，然后移动到 `workspaces/04-abandoned`。
- 不为中间阶段移动目录。目录移动是任务生命周期事件，不是普通 stage transition。

## 实现形态

- 在 [`crates/daedalus-cli/src/infrastructure/workspace_fs.rs`](crates/daedalus-cli/src/infrastructure/workspace_fs.rs) 新增目标目录 helper：
  - `completed_root(repo_root)` -> `workspaces/03-completed`
  - `abandoned_root(repo_root)` -> `workspaces/04-abandoned`
  - `move_task(task_dir, destination_root)`：使用 `fs::rename` 移动目录，并保护目标目录冲突。
- 新增应用层 use case，建议放在 [`crates/daedalus-cli/src/application/close_task.rs`](crates/daedalus-cli/src/application/close_task.rs)，返回结构化结果：
  - `action`: `complete` or `abandon`
  - `moved`: `true`
  - `from_task_dir`
  - `to_task_dir`
  - `state_md`
  - `next`
- 扩展 [`crates/daedalus-cli/src/interfaces/agent_cli/commands`](crates/daedalus-cli/src/interfaces/agent_cli/commands) 下的 CLI 命令：
  - 新增 `task` command module，包含 `complete` 和 `abandon` 子命令。
  - 在 `commands/mod.rs` 中沿用现有 `CmdExecutor + enum_dispatch` 模式注册。
- 更新 [`transition_stage.rs`](crates/daedalus-cli/src/application/transition_stage.rs) 中的 `state complete 10-archivist` 流程：最终阶段完成成功后调用同一套任务关闭逻辑，并返回移动元数据。
- 更新 [`presenter.rs`](crates/daedalus-cli/src/interfaces/agent_cli/presenter.rs) 输出，让 text 和 JSON 都清楚说明移动情况：
  - `ok: task completed`
  - `moved: true`
  - `from_task_dir: ...`
  - `to_task_dir: ...`
  - `next: active WIP slot released; start a new task with daedalus init repo-learning <name>`

## 状态与模板更新

- 如果记录 `task-complete` / `abandon` 这类生命周期动作，需要同步扩展 [`system/templates/repo/.daedalus/state.toml`](system/templates/repo/.daedalus/state.toml) 和 [`state.md`](crates/daedalus-cli/src/application/render.rs) 中的允许值说明。
- 不要把 `stage.status` 过度扩展成任务生命周期状态。任务目录所在 bucket 表示 lifecycle；`state.toml` transitions 记录原因和操作者。
- 更新根 [`CLAUDE.md`](CLAUDE.md)，让 Agent 明确：已完成或已放弃的任务应离开 `workspaces/02-learning`，从而释放 WIP。

## 测试

- 在 [`crates/daedalus-cli/tests/cli_workflow.rs`](crates/daedalus-cli/tests/cli_workflow.rs) 增加集成测试：
  - `state complete 10-archivist` 会把任务移动到 `workspaces/03-completed`，输出包含移动前/后的路径。
  - `task complete` 会把任务移动到 `workspaces/03-completed` 并释放 WIP。
  - `task abandon --reason ...` 会把任务移动到 `workspaces/04-abandoned` 并记录原因。
  - 目标目录已存在时拒绝覆盖。
  - 移动后，`daedalus init repo-learning <new>` 可以成功，因为 `02-learning` 已清空。

## 验证

- 运行 `cargo fmt --manifest-path crates/Cargo.toml --all`。
- 运行 `make ci`。
- 只有在明确确认要完成或放弃 `pre-stage-one-dry-run` 后，才对该 dry-run workspace 做手工移动 smoke test。
