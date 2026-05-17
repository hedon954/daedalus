# 学习任务上下文

本目录是学习任务 `openai-codex-cli-deep-learning` 的工作区。

处理当前学习任务前，先阅读这些上下文文件：

@.daedalus/state.md
@.daedalus/task-card.md
@.daedalus/outcome-map.md
@.daedalus/todo.md
@.daedalus/long-context.md
@.daedalus/artifact-index.md
@.daedalus/decision-log.md
@.daedalus/validation-log.md

## 操作规则

- 将 [`.daedalus/state.toml`](.daedalus/state.toml) 视为机器可读的状态源，也是任务 lifecycle 的唯一事实源；目录 bucket 只是投影，必须与 `task.lifecycle` 和 `task.workspace_bucket` 一致。
- 不要手动编辑 [`.daedalus/state.md`](.daedalus/state.md)；需要更新时运行 `daedalus state render` 重新生成。
- 当学习证据、阶段状态或任务范围变化时，及时更新 [`.daedalus/outcome-map.md`](.daedalus/outcome-map.md)、[`.daedalus/todo.md`](.daedalus/todo.md)、[`.daedalus/artifact-index.md`](.daedalus/artifact-index.md)、[`.daedalus/decision-log.md`](.daedalus/decision-log.md)、[`.daedalus/validation-log.md`](.daedalus/validation-log.md) 和 [`.daedalus/long-context.md`](.daedalus/long-context.md)。
- 使用 `system/prompts/repo` 下的分阶段 repo 学习提示词来引导教学过程。
- 除非用户明确批准，或已有可追溯的等价证据，否则不要使用 `--force` 这类强制绕过选项。
- `.daedalus/outcome-map.md` 是全程导航仪表盘。每次继续学习、深读源码或阶段切换前，先说明当前动作服务哪个最终产物、补哪个缺口、需要什么证据、完成后解锁什么。
- `.daedalus/todo.md` 是路径看板，不只是任务列表；它应该显示 North Star、Current Path、Now、Gaps Blocking Next Stage 和 Stage Exit Criteria。
- `guides/` 用于保存 Agent 生成的行动指南、问题引导、运行说明和验收清单；`notes/` 用于保存用户亲自观察、回答、实践和总结后的学习笔记。
- 阶段产物优先使用文件夹入口：`guides/<stage-id>/README.md` 和 `notes/<stage-id>/README.md`。README 只做阶段导航、状态和索引；专题内容写入同目录下的问题文件。
- 不要默认代替用户完成关键学习实践。Agent 应该说明要做什么、为什么做、用户等待时可以思考什么，并在用户完成实践后协助验收和排障。
- 外部源码放在 `source/`，默认被忽略。优先维护 [`source/pull_source.sh`](source/pull_source.sh) 让用户按需拉取；如果 Agent 代为拉取，必须避免提交外部源码和嵌套 `.git`。
