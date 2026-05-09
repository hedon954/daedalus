# 学习任务上下文

本目录是学习任务 `{{TASK_NAME}}` 的工作区。

处理当前学习任务前，先阅读这些上下文文件：

@.daedalus/state.md
@.daedalus/task-card.md
@.daedalus/todo.md
@.daedalus/long-context.md
@.daedalus/artifact-index.md
@.daedalus/decision-log.md
@.daedalus/validation-log.md

## 操作规则

- 将 [`.daedalus/state.toml`](.daedalus/state.toml) 视为机器可读的状态源，也是任务 lifecycle 的唯一事实源；目录 bucket 只是投影，必须与 `task.lifecycle` 和 `task.workspace_bucket` 一致。
- 创建新的 repo learning 任务必须使用 `daedalus init repo-learning <task-name>`；不要手写 `.daedalus`、`state.toml`、`todo.md` 等模板文件。CLI 不可用时先报告阻塞原因，不要自动 fallback 到手写模板。
- 不要手动编辑 [`.daedalus/state.md`](.daedalus/state.md)；需要更新时运行 `daedalus state render` 重新生成。
- 当学习证据、阶段状态或任务范围变化时，及时更新 [`.daedalus/todo.md`](.daedalus/todo.md)、[`.daedalus/artifact-index.md`](.daedalus/artifact-index.md)、[`.daedalus/decision-log.md`](.daedalus/decision-log.md)、[`.daedalus/validation-log.md`](.daedalus/validation-log.md) 和 [`.daedalus/long-context.md`](.daedalus/long-context.md)。
- 使用 `system/prompts/repo` 下的分阶段 repo 学习提示词来引导教学过程。
- 除非用户明确批准，或已有可追溯的等价证据，否则不要使用 `--force` 这类强制绕过选项。
- `guides/` 用于保存 Agent 生成的行动指南、问题引导、运行说明和验收清单；`notes/` 用于保存用户亲自观察、回答、实践和总结后的学习笔记。
- 不要默认代替用户完成关键学习实践。Agent 应该说明要做什么、为什么做、用户等待时可以思考什么，并在用户完成实践后协助验收和排障。
- Notes Ownership Rule：用户没有回答、观察或实践前，不要把结论写入 `notes/`。Agent 可以写 `guides/`，也可以在 `notes/` 中创建待用户填写的轻量模板；整理用户口述内容时，必须保留用户原始回答和待验证假设。
- 外部源码放在 `source/`，默认被忽略。优先维护 [`source/pull_source.sh`](source/pull_source.sh) 让用户按需拉取；如果 Agent 代为拉取，必须避免提交外部源码和嵌套 `.git`。
- 学习外部 repo 时，建议用户单独用 Cursor 打开该 repo 或其实际 workspace 根目录。运行、断点和 `launch.json` 示例默认以被学习 repo 的 workspace 为基准；daedalus 任务目录只保存学习状态、指南、笔记和复盘。
