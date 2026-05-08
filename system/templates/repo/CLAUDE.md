# 学习任务上下文

本目录是学习任务 `{{TASK_NAME}}` 的工作区。

处理当前学习任务前，先阅读这些上下文文件：

@.daedalus/state.md
@.daedalus/task-card.md
@.daedalus/todo.md
@.daedalus/long-context.md
@.daedalus/artifact-index.md
@.daedalus/decision-log.md

## 操作规则

- 将 [`.daedalus/state.toml`](.daedalus/state.toml) 视为机器可读的状态源，也是任务 lifecycle 的唯一事实源；目录 bucket 只是投影，必须与 `task.lifecycle` 和 `task.workspace_bucket` 一致。
- 不要手动编辑 [`.daedalus/state.md`](.daedalus/state.md)；需要更新时运行 `daedalus state render` 重新生成。
- 当学习证据、阶段状态或任务范围变化时，及时更新 [`.daedalus/todo.md`](.daedalus/todo.md)、[`.daedalus/artifact-index.md`](.daedalus/artifact-index.md)、[`.daedalus/decision-log.md`](.daedalus/decision-log.md) 和 [`.daedalus/long-context.md`](.daedalus/long-context.md)。
- 使用 `system/prompts/repo` 下的分阶段 repo 学习提示词来引导教学过程。
- 除非用户明确批准，或已有可追溯的等价证据，否则不要使用 `--force` 这类强制绕过选项。
