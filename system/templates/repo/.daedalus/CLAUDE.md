# 学习任务上下文

本目录是学习任务 `{{TASK_NAME}}` 的大脑和状态中心。

处理当前学习任务前，先阅读这些上下文文件：

@state.md
@task-card.md
@todo.md
@long-context.md
@artifact-index.md
@decision-log.md

## 操作规则

- 将 [`.daedalus/state.toml`](state.toml) 视为机器可读的状态源。
- 不要手动编辑 [`.daedalus/state.md`](state.md)；需要更新时运行 `daedalus state render` 重新生成。
- 当学习证据、阶段状态或任务范围变化时，及时更新 [`todo.md`](todo.md)、[`artifact-index.md`](artifact-index.md)、[`decision-log.md`](decision-log.md) 和 [`long-context.md`](long-context.md)。
- 使用 `system/prompts/repo` 下的分阶段 repo 学习提示词来引导教学过程。
- 除非用户明确批准，或已有可追溯的等价证据，否则不要使用 `--force` 这类强制绕过选项。
