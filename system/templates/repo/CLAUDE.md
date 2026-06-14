# Repo Learning Project Context

本目录是长期 repo learning project `{{TASK_NAME}}` 的工作区。

处理当前 project 前，先阅读 project 层上下文：

@.daedalus/state.md
@.daedalus/project-map.md
@.daedalus/topic-board.md
@shared/evidence-registry.md

然后读取 active topic：

@topics/{{TOPIC_SLUG}}/.daedalus/state.md
@topics/{{TOPIC_SLUG}}/.daedalus/task-card.md
@topics/{{TOPIC_SLUG}}/.daedalus/outcome-map.md
@topics/{{TOPIC_SLUG}}/.daedalus/todo.md
@topics/{{TOPIC_SLUG}}/.daedalus/long-context.md

## 操作规则

- Project root 的 `.daedalus/state.toml` 只描述 project lifecycle、workspace bucket、active topic 和 topic 列表。
- Topic 的 `topics/<slug>/.daedalus/state.toml` 才描述 10-stage 学习进度。
- 创建新的 repo learning project 必须使用 `daedalus init repo-learning <project-name> --topic <topic-slug> --title <topic-title>`。
- 创建新专题必须使用 `daedalus topic new <topic-slug> --title <topic-title>`。
- 切换专题必须使用 `daedalus topic activate <topic-slug>`。
- 推进阶段时，`daedalus state enter/complete/block/resume <stage-id>` 默认操作 active topic，不操作 project root。
- 如需指定专题，使用 `daedalus state complete <stage-id> --topic <topic-slug>`。
- 关闭专题使用 `daedalus topic complete <topic-slug> --reason <reason>`；关闭整个 project 才使用 `daedalus task complete --reason <reason>`。
- 不要手动编辑 `state.md`；需要更新时运行 `daedalus state render` 渲染 active topic，或 `daedalus state render --project` 渲染 project。
- `daedalus validate` 校验 project + active topic；`daedalus validate --all-topics` 校验 project + 所有 topics。
- 不要在 project root 手写 topic stage 状态，不要把 topic notes 写到 project root。
- `shared/` 只保存跨 topic 可复用的 verified knowledge；topic 草稿和用户回答留在 `topics/<slug>/notes/`。
- 每次继续学习前，先说明 project、active topic、topic stage、current gap 和完成后解锁什么。
- 使用 `system/prompts/repo` 下的分阶段 repo 学习提示词来引导 active topic 的学习过程。
- 除非用户明确批准，或已有可追溯的等价证据，否则不要使用 `--force` 这类强制绕过选项。
- 不要默认代替用户完成关键学习实践。Agent 应该说明要做什么、为什么做、用户等待时可以思考什么，并在用户完成实践后协助验收和排障。
- Notes Ownership Rule：用户没有回答、观察或实践前，不要把结论写入 topic `notes/`。Agent 可以写 topic `guides/`，也可以在 topic `notes/` 中创建待用户填写的轻量模板。
- 外部源码放在 `source/`，默认被忽略。优先维护 [`source/pull_source.sh`](source/pull_source.sh) 让用户按需拉取。
