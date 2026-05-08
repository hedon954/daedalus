---
name: pre-stage-one
overview: 当前已经完成 repo 学习任务的 CLI、模板、状态机和 prompt 分层基线。启动第 01 阶段前，重点不再是补齐基础设施，而是用一次受控 dry run 验证这些基线能否稳定支撑真实教学流程。
todos:
  - id: baseline-inventory
    content: Record the implemented CLI, templates, state machine, prompt layering, and quality-check baseline.
    status: completed
  - id: workspace-init-smoke-test
    content: Smoke test `daedalus init repo-learning` from the repo root and a subdirectory, including WIP behavior and generated paths.
    status: completed
  - id: template-navigation-audit
    content: Audit generated `.daedalus` templates for Chinese content, relative links, source-of-truth clarity, and Agent-readable loading order.
    status: completed
  - id: state-machine-consistency-audit
    content: Check that the 10-stage state machine, required artifacts, enum constraints, and generated state.md remain consistent.
    status: completed
  - id: cli-transition-smoke-test
    content: Smoke test `state render`, `state block`, `state resume`, `state complete`, `validate`, and constrained force behavior.
    status: completed
  - id: prompt-layering-audit
    content: Audit common/repo prompt layering and ensure `01-goal-aligner` uses common clarification logic without duplicating it.
    status: completed
  - id: stage01-artifact-dry-run
    content: Run a dry `01-goal-aligner` simulation and fill the Stage 01 artifacts as if it were a real learning task.
    status: completed
  - id: dry-run-friction-backlog
    content: Record observed friction and decide whether each issue should update prompts, templates, CLI behavior, tests, or docs.
    status: completed
  - id: stage01-readiness-decision
    content: Decide whether the system is ready for a real Stage 01 run, or whether another dry run is needed.
    status: completed
isProject: false
---

# Before Stage 01 Setup

## Goal

在真正开启第 01 阶段前，先把已经实现的 CLI、模板、状态机、prompt 分层和校验规则当作一个整体做验收。现在的风险已经不是“缺少基础结构”，而是这些结构在真实 Agent 教学流程中是否足够顺手、稳定、可恢复。

本计划因此从“建设前置能力”调整为“确认当前基线、执行 Stage 01 dry run、记录摩擦并反向改进 prompt/template/CLI”。

## Current Baseline

当前已经具备以下基础能力：

- `crates/daedalus-cli` 提供 Agent-friendly CLI `daedalus` 和 human-friendly TUI `daedalus-tui`，共享应用层和领域层，只在输出层分化。
- `daedalus` 已改成 trait + `enum_dispatch` 的命令执行模型，`main` 只负责 tracing 初始化、参数解析、上下文发现和 `command.execute(ctx).await`。
- CLI 已支持 `init repo-learning`、`state enter`、`state complete`、`state block`、`state resume`、`state render`、`validate`。
- CLI 只能在 daedalus 项目根目录或其子目录下运行；合法目录会动态推导 repo root、active task 和输出路径。
- `state.toml` 是机器可读事实源，CLI 使用 `toml_edit` 更新并保留注释；`state.md` 是派生摘要，不应手动编辑。
- `system/templates/repo/.daedalus` 已提供中文模板：`CLAUDE.md`、`task-card.md`、`state.toml`、`state.md`、`todo.md`、`long-context.md`、`artifact-index.md`、`decision-log.md`。
- 模板和生成的 Markdown 已使用相对链接，便于在 IDE 内跳转。
- `state.toml` 和 `state.md` 已显式列出 `stage.status`、`transition.action`、`transition.approval_source` 枚举约束。
- `artifact-index.md` 已用 blockquote 隐式说明 `状态` 列允许值。
- `--force` 类绕过行为已被约束：只有用户明确批准或已有可追溯等价证据时才允许使用，并且要记录 reason 与 approval source。
- `Makefile` 已提供 `fmt`、`fmt-check`、`check`、`clippy`、`test`、`build`、`ci`，其中 `build` 生成 release binary 并输出 `daedalus` 与 `daedalus-tui` 路径。

## What To Verify Before Stage 01

### 1. Workspace Init Smoke Test

学习任务工作区契约已经落在 [`system/templates/repo`](system/templates/repo) 和 `daedalus init repo-learning` 中。Stage 01 前不再重新设计结构，而是用 smoke test 验证 CLI 初始化出来的目录是否能让 Agent 稳定理解当前任务。

验收时重点检查：

- 在 daedalus repo 根目录执行 `daedalus init repo-learning <dry-run-name>` 是否能生成预期 workspace。
- 在 daedalus repo 任意子目录执行同一类命令时，是否能动态推导 repo root 和输出路径。
- 在非法目录执行时，是否拒绝服务并给出可行动的错误信息。
- 已有 active task 时，是否默认阻止创建新任务，并只在用户明确批准时才允许 `--allow-existing-active --reason ...`。
- 根目录 `CLAUDE.md` 是否优先引用 `@.daedalus/state.md`、`@.daedalus/task-card.md`、`@.daedalus/todo.md`、`@.daedalus/long-context.md`、`@.daedalus/artifact-index.md`、`@.daedalus/decision-log.md`。
- Agent 是否能从 `.daedalus/state.md` 直接判断当前阶段、下一步、缺失产物和最近 transition。
- `todo.md` 是否足够层次化，能表达 Doing、Next、Blocked、Done、Cancelled，而不是一次性静态清单。
- `artifact-index.md` 是否用相对链接索引产物，并使用约束内的中文状态值。
- `decision-log.md` 与 `long-context.md` 是否能支撑暂停和恢复，而不依赖聊天记忆。
- WIP = 1 的约束是否通过 CLI 初始化和校验体现出来。

当前约定结构：

```text
workspaces/02-learning/learning-xxx/
  CLAUDE.md              # Agent 恢复当前学习任务的入口
  .daedalus/
    task-card.md
    state.toml
    state.md
    todo.md
    long-context.md
    artifact-index.md
    decision-log.md
  source/
    .gitkeep             # 可选：学习对象的本地 checkout 或解压内容
  demo/
    .gitkeep             # 可选：mini demo 实作
  notes/
    .gitkeep             # 可选：阶段笔记、图、阅读记录
```

完成标准：只打开一个 learning workspace，不看聊天记录，也能知道当前学什么、处于第几阶段、下一步做什么、已有产物在哪里；CLI 输出路径和错误信息也能支撑 Agent 下一步行动。

### 2. Template Navigation Audit

模板层已经初始化完成，当前 source of truth 是 [`system/templates/repo`](system/templates/repo)。Stage 01 前要确认模板不是“看起来完整”，而是真的能被 Agent 按预期消费。

验收时重点检查：

- 模板内容保持中文，避免初始化后还需要人工翻译。
- 所有文件路径使用 Markdown 相对链接，便于 IDE 跳转。
- `state.toml` 注释明确枚举约束，防止 Agent 发明新状态。
- `state.md` 渲染结果包含枚举约束、当前状态、阶段进度、缺失产物和最近 transition。
- `artifact-index.md` 用 blockquote 描述 `状态` 列允许值，不再新增显眼二级标题。
- 模板示例不应再复制到本计划中维护；以实际模板文件和 CLI integration tests 为准，避免双写漂移。

完成标准：`daedalus init repo-learning <name>` 生成的根目录 `CLAUDE.md`、`.daedalus` 内容和 `demo/notes/source/.gitkeep` 可以直接进入 Stage 01 dry run，不需要再人工补结构。

#### Template Source Of Truth

模板样例不再复制到本计划中维护，避免和实际模板双写漂移。后续需要看 demo 时，直接以这些文件为准：

- [`system/templates/repo/CLAUDE.md`](system/templates/repo/CLAUDE.md)
- [`system/templates/repo/.daedalus/task-card.md`](system/templates/repo/.daedalus/task-card.md)
- [`system/templates/repo/.daedalus/state.toml`](system/templates/repo/.daedalus/state.toml)
- [`system/templates/repo/.daedalus/state.md`](system/templates/repo/.daedalus/state.md)
- [`system/templates/repo/.daedalus/todo.md`](system/templates/repo/.daedalus/todo.md)
- [`system/templates/repo/.daedalus/long-context.md`](system/templates/repo/.daedalus/long-context.md)
- [`system/templates/repo/.daedalus/artifact-index.md`](system/templates/repo/.daedalus/artifact-index.md)
- [`system/templates/repo/.daedalus/decision-log.md`](system/templates/repo/.daedalus/decision-log.md)
- [`system/templates/repo/demo/.gitkeep`](system/templates/repo/demo/.gitkeep)
- [`system/templates/repo/notes/.gitkeep`](system/templates/repo/notes/.gitkeep)
- [`system/templates/repo/source/.gitkeep`](system/templates/repo/source/.gitkeep)

### 3. State Machine Consistency Audit

[`repo-learning-coach`](.claude/skills/repo-learning-coach/SKILL.md) 描述的 10 个阶段已经落入 [`system/templates/repo/.daedalus/state.toml`](system/templates/repo/.daedalus/state.toml)，并由 CLI 维护。Stage 01 前要验证状态机的阶段命名、必需产物和 prompt 命名是否一致。

当前阶段序列为：

- `01-goal-aligner`
- `02-repo-scout`
- `03-socratic-coach`
- `04-debugger-guide`
- `05-arch-analyzer`
- `06-code-reader`
- `07-demo-architecture`
- `08-demo-coder`
- `09-biz-solver`
- `10-archivist`

验收时重点检查：

- `current_phase` 是否与 active stage 保持一致。
- 任意时刻是否只有一个 active stage。
- `required_artifacts` 是否覆盖该阶段真正需要的最小产物。
- `state complete` 是否能阻止缺失产物的阶段完成。
- `state block` 和 `state resume` 是否能表达真实学习阻塞。
- `state render` 生成的 `state.md` 是否比直接读 `state.toml` 更适合 Agent 快速消费。

现阶段不引入更复杂的 telemetry 或评分系统。先用 transition、artifact-index、decision-log 和 long-context 记录足够复盘的信息。

完成标准：CLI 可以根据 `state.toml` 校验状态一致性；Agent 可以通过 `state.md` 快速判断当前应该加载哪个阶段 prompt、能不能进入下一阶段、缺失哪些产物。

### 4. CLI Transition Smoke Test

`.daedalus/state.toml` 的关键状态字段已经由 Rust CLI 确定性维护。Stage 01 前要验证 CLI 行为能被 Agent 安全调用，并且输出足够稳定。

需要实际走一遍这些命令：

- `daedalus init repo-learning <name>`：初始化一个 dry-run 学习任务。
- `daedalus validate <task-dir>`：确认必需文件、active stage、当前阶段和派生状态一致。
- `daedalus state render <task-dir>`：重新生成 `.daedalus/state.md`。
- `daedalus state block 01-goal-aligner --reason ...`：模拟用户目标不清晰时的阻塞。
- `daedalus state resume 01-goal-aligner --reason ...`：模拟用户补充信息后的恢复。
- `daedalus state complete 01-goal-aligner`：验证缺少必需产物时会拒绝完成。
- `daedalus state complete 01-goal-aligner --force --reason ... --approval-source ...`：只在 dry run 中验证约束提示，不把它作为常规路径。

CLI 仍不替 Agent 做教学判断。比如“用户是否真的完成目标澄清”由 Agent 和用户判断；CLI 只记录阶段状态、原因、产物是否存在，以及状态转换是否满足结构约束。

完成标准：Agent 不直接手改 `.daedalus/state.toml` 的关键状态字段，也不直接编辑 `.daedalus/state.md`；阶段流转由 CLI 更新 TOML 并重新生成 Markdown 摘要。手写注释和人工补充说明在 CLI 更新后仍然保留。

### 5. Prompt Layering Audit

repo prompt 已经通过 `@system/prompts/common/...` 引用 common prompt。Stage 01 前要验证分层协议是否真的减少了重复内容，并且不会让 Agent 在执行阶段 prompt 时漏掉通用能力。

验收时重点检查：

- `common` 只放跨材料通用能力，例如目标澄清、第一性原理、问题路线图、总结、知识归档。
- `repo` 只写 code repo 特有的 delta，例如运行、调试、架构、代码阅读、mini demo。
- 阶段 prompt 顶部使用 `@` 引用需要的 common prompt，而不是让 Agent 再主动读取。
- 如果 common prompt 改了，repo/book/paper/course 自动继承新规则，不重复维护。
- 所有涉及 todo 的 prompt 都应明确：todo list 是动态教学工具，不是一次性计划。Agent 要能创建 todo、拆分 todo、更新优先级、完成 todo、取消过时 todo，并解释变化原因。
- `01-goal-aligner` 是否只保留 repo 学习特有的目标对齐要求，并把通用澄清逻辑交给 common prompt。
- `gatekeeper` 是否能引导 Agent 在目标不清晰时拒绝推进，而不是为了完成阶段而补虚假内容。

完成标准：任何一个阶段 prompt 打开后，都能看出它依赖哪些 common 能力，以及它自己只负责什么差异化职责。

### 6. Stage 01 Artifact Dry Run

第 01 阶段会产出最关键的学习边界。如果这些结果只留在聊天里，后续选 repo、提问、运行和 demo 设计都会失去稳定锚点。

当前约定：

- `.daedalus/task-card.md` 写入学习目标：现实问题、当前基础、最小输出物、验收标准。
- 通过 Rust CLI 更新 `.daedalus/state.toml`，记录当前阶段为 `01-goal-aligner`，追加 transition，并重新生成 `.daedalus/state.md`。
- `.daedalus/decision-log.md` 记录为什么接受、延后或拒绝该学习任务。
- `.daedalus/artifact-index.md` 记录 01 阶段生成的任务卡和后续需要补的材料。
- 如果用户目标还不够清晰，在层次化 `.daedalus/todo.md` 写入需要用户回答的问题，并在用户回答后及时完成或拆分 todo。

完成标准：完成 01 阶段后，不看聊天记录也能知道这个学习任务为什么存在、要解决什么问题、下一步是否可以进入 repo 选择。

### 7. Dry Run Friction Backlog

你后续真正要优化的是 Agent 的教学引导能力，因此每次实践都要留下可复盘数据。否则只能凭感觉改 prompt，很难知道哪一类问题、哪一个阶段真正有效。

dry run 和后续真实阶段都应记录：

- Agent 提出的问题是否让用户产生了新的理解。
- 用户在哪些概念、权衡或代码点上卡住。
- 哪些追问有效，哪些追问让用户更困惑。
- 阶段是否按预期产出了文件。
- prompt 是否需要变短、变具体、增加例子或移动到 common。

可以先用轻量人工记录，不急着做 telemetry 自动化。等流程跑过几次后，再决定哪些指标值得进入确定性逻辑。

每个摩擦点都要分类到一个后续处理位置：

- prompt 问题：更新 `system/prompts/common` 或 `system/prompts/repo`。
- template 问题：更新 [`system/templates/repo/.daedalus`](system/templates/repo/.daedalus)。
- CLI 行为问题：更新 `crates/daedalus-cli` 和对应 integration tests。
- 文档问题：更新 `CLAUDE.md`、`crates/docs/instructions.md` 或本计划。
- 暂不处理的问题：写入 `.daedalus/decision-log.md`，说明为什么不马上处理。

完成标准：每跑完一个阶段，都能回答“这次教学哪里有效、哪里无效、下一版 prompt/template/CLI/test 应该改什么”。

### 8. Stage 01 Readiness Decision

dry run 结束后，不应自动进入真实 Stage 01，而是做一次明确的 readiness decision。

可以进入真实 Stage 01 的条件：

- `daedalus validate <task-dir>` 能通过。
- `01-goal-aligner` 的必需产物可以在不使用 `--force` 的情况下完成。
- Agent 能从根目录 `CLAUDE.md` 和 `@.daedalus/state.md` 正确理解当前学习任务。
- 用户能看懂 `task-card.md` 中的现实问题、验收标准和暂不学习范围。
- 所有高优先级摩擦点都已经修复，或已明确记录为可接受风险。

完成标准：在 `.daedalus/decision-log.md` 或本计划中记录结论：`ready-for-real-stage-01`、`needs-another-dry-run` 或 `blocked`。

## Recommended Dry Run Before Real Stage 01

1. 在 daedalus repo 根目录或子目录中运行 `daedalus init repo-learning <dry-run-name>`，创建一个假想学习任务；如果已有 active task，优先复用或关闭现有任务，只有用户明确批准 dry run 并给出原因时才使用 `--allow-existing-active --reason ...`。
2. 打开生成的根目录 `CLAUDE.md`，确认 `@.daedalus/state.md`、`@.daedalus/task-card.md`、`@.daedalus/todo.md`、`@.daedalus/long-context.md`、`@.daedalus/artifact-index.md`、`@.daedalus/decision-log.md` 的加载顺序适合 Agent。
3. 用一个假想 code repo 学习目标只跑 `01-goal-aligner`，不要进入 `02-repo-scout`。
4. 在 `.daedalus/task-card.md` 中填写现实问题、当前基础、最小输出物、验收标准、repo 选择约束、暂不学习范围。
5. 在 `.daedalus/todo.md` 中记录并更新 Stage 01 的动态 todo，至少覆盖 Doing、Blocked、Done 三类变化。
6. 在 `.daedalus/decision-log.md` 中记录接受、延后或拒绝该学习任务的原因。
7. 在 `.daedalus/artifact-index.md` 中用相对链接登记 01 阶段产物，并使用允许的 `状态` 值。
8. 运行 `daedalus validate <task-dir>`，确认 workspace 结构和状态一致。
9. 运行 `daedalus state complete 01-goal-aligner --task-dir <task-dir>`，验证在必需产物存在时能完成阶段；如果不能完成，优先修正产物或模板，不使用 `--force`。
10. 根据 dry run 摩擦调整 `clarify-goal`、`gatekeeper`、`01-goal-aligner`、模板或 CLI 测试，再开启真实 Stage 01。

## Why This Matters

你真正要迭代的不是单个 prompt，而是一个长周期学习系统。现在基础设施已经具备，01 阶段之前最重要的是验证“状态、产物、恢复、评估”这套闭环是否能在真实 Agent 教学中自然运转。dry run 产生的摩擦应该沉淀回 prompt、template、CLI 或测试，而不是继续散落在聊天记录里。
