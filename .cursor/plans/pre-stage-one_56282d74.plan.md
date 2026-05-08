---
name: pre-stage-one
overview: 在启动第 01 阶段前，先补齐 repo 学习任务的运行边界：工作区结构、状态机、产物模板、提示词加载规则和反馈闭环。这样后续实践中才能稳定迭代 10 个阶段的教学能力。
todos:
  - id: workspace-contract
    content: Design the active learning workspace contract for one repo learning task.
    status: pending
  - id: templates
    content: Create minimal templates and demo samples for CLAUDE.md, task card, state.toml, generated state.md, todo, long context, artifact index, and decision log.
    status: pending
  - id: stage-state
    content: Define the 10-stage state machine in state.toml and generated state.md with entry and exit criteria.
    status: pending
  - id: state-cli
    content: Plan a Rust CLI that deterministically updates state.toml using toml_edit and regenerates state.md.
    status: pending
  - id: prompt-loading
    content: Document the `@`-based prompt layering protocol for common and repo prompts.
    status: pending
  - id: stage01-dry-run
    content: Run a dry 01-stage simulation and refine the first-stage prompts from observed friction.
    status: pending
isProject: false
---

# Before Stage 01 Setup

## Goal

在真正开启第 01 阶段前，不急着开始找 repo 或澄清目标，而是先把“一个 repo 学习任务如何在文件系统里存在、推进、暂停、恢复、评价”定义清楚。否则 01 阶段会产出信息，但后续 02-10 阶段很难稳定接住。

## What To Prepare

### 1. 定义学习任务工作区契约

先规定一个 active repo learning task 在文件系统里应该长什么样。建议每个任务在 [`workspaces/02-learning`](workspaces/02-learning) 下有一个独立目录，目录名包含学习主题和日期，避免多个任务混在一起。每个学习任务的“大脑和状态中心”统一放在任务目录下的 `.daedalus/` 中。

这个契约至少要回答：

- 哪些文件是必需的，例如 `.daedalus/CLAUDE.md`、任务卡、阶段状态、todo、长期上下文、产物索引、决策日志。
- 哪些文件由 Agent 更新，哪些文件主要由用户手写或确认。
- 任务暂停、恢复、完成、放弃时，目录如何迁移到 `03-completed` 或 `04-abandoned`。
- 如果 WIP = 1，新的任务进入前如何检查当前 active 任务是否已经关闭或暂停。

推荐结构：

```text
workspaces/02-learning/learning-xxx/
  .daedalus/
    CLAUDE.md
    task-card.md
    state.toml
    state.md
    todo.md
    long-context.md
    artifact-index.md
    decision-log.md
  source/                # 可选：学习对象的本地 checkout 或解压内容
  demo/                  # 可选：mini demo 实作
  notes/                 # 可选：阶段笔记、图、阅读记录
```

完成标准：看到一个 workspace 目录，就能知道当前学什么、处于第几阶段、下一步做什么、已有产物在哪里。

### 2. 初始化模板层

模板层负责把“学习过程中的状态和产物”标准化，避免每次都临时发明文件结构。模板应放在 [`system/templates`](system/templates)，repo 专用模板可放在 `system/templates/repo`。这些模板在初始化学习任务时会生成到任务目录的 `.daedalus/` 下。

建议先补齐最小模板：

- `.daedalus/CLAUDE.md`：当前学习任务的上下文入口，低频更新，通过 `@` 引用任务状态文件。
- `.daedalus/task-card.md`：记录学习目标、现实问题、验收标准、材料来源、输出物。
- `.daedalus/state.toml`：结构化状态事实源，记录当前阶段、当前步骤、状态、阶段转换记录。选择 TOML 是为了可读、可注释，并由后续 Rust CLI 使用 `toml_edit` 做确定性更新且保留注释。
- `.daedalus/state.md`：由 CLI 从 `state.toml` 生成的 Agent 友好状态摘要。它是只读派生产物，供 `.daedalus/CLAUDE.md` 使用 `@state.md` 引用。
- `.daedalus/todo.md`：记录层次化任务树，而不是平铺清单。它要支持按阶段、主题、阻塞项组织，并允许 Agent 在学习推进中创建、拆分、重排、完成和取消 todo。
- `.daedalus/long-context.md`：记录可恢复上下文，承接 `compress-context`。
- `.daedalus/artifact-index.md`：索引所有产物，包括图、runbook、阅读笔记、demo、业务方案。
- `.daedalus/decision-log.md`：记录关键选择和原因，类似轻量 ADR。

完成标准：新开一个学习任务时，可以复制模板生成 workspace，而不是从空目录开始。

#### Template Demo Samples

`.daedalus/CLAUDE.md` demo：

```markdown
# Daedalus Learning Context

You are coaching this active repo learning task.

## Stable Rules

- Keep WIP = 1.
- Use filesystem state over chat memory.
- Do not mark a stage complete without required artifacts.

## Task Brain

@task-card.md
@state.md
@artifact-index.md
@decision-log.md

## Volatile Context

@todo.md
@long-context.md
```

`.daedalus/task-card.md` demo：

```markdown
# Task Card

## Goal
理解某个代码仓库如何实现插件化任务调度，并能迁移到自己的 Agent 工具链。

## Real Problem
当前业务需要让多个工具按上下文动态组合执行，但不希望每次新增工具都修改主流程。

## Learning Material
- type: code-repo
- source: https://example.com/org/repo
- local_path: ../source

## Current Baseline
- 已熟悉 Rust trait 和 async 基础。
- 不熟悉该 repo 的模块边界和调度模型。

## Minimum Output
- 一份架构图。
- 一个复现插件注册和调度的 mini demo。
- 一份业务迁移方案。

## Acceptance Criteria
- 能解释核心链路。
- 能运行 demo。
- 能说清楚该 repo 的 trade-off 和不适用场景。
```

`.daedalus/state.toml` demo：

```toml
# Human-readable state for one active repo learning task.
# Updated by both Agent and deterministic CLI. Keep comments.

current_phase = "01-clarify-goal"
current_step = "01-align-repo-learning-goal"
status = "active" # pending | active | blocked | done | skipped

[wip]
limit = 1
active_task = "2026-05-plugin-scheduler"

[[stages]]
id = "01-clarify-goal"
status = "active"
entry_criteria = ["user_has_initial_learning_intent"]
exit_criteria = ["task_card_completed", "repo_selection_constraints_defined"]
artifacts = [".daedalus/task-card.md", ".daedalus/decision-log.md", ".daedalus/todo.md"]

[[stages]]
id = "02-first-principles"
status = "pending"
entry_criteria = ["task_card_completed"]
exit_criteria = ["first_principles_note_created"]
artifacts = ["notes/first-principles.md"]

[telemetry]
started_at = "2026-05-01T09:00:00Z"
total_active_days = 0
transition_count = 0
adr_count = 0

[[transitions]]
stage = "01-clarify-goal"
action = "enter"
timestamp = "2026-05-01T09:00:00Z"
reason = "new repo learning task accepted"
```

`.daedalus/state.md` demo：

```markdown
# Learning State

> Generated from `.daedalus/state.toml`. Do not edit manually.

## Current

- phase: 01-clarify-goal
- step: 01-align-repo-learning-goal
- status: active
- next_action: finish task-card.md and define repo selection constraints

## Stage Progress

- 01 Clarify Goal: active
- 02 First Principles: pending
- 03 Select Repo: pending
- 04 Build Question Roadmap: pending
- 05 Run And Debug Repo: pending
- 06 Analyze Architecture: pending
- 07 Read Core Code: pending
- 08 Build Mini Demo: pending
- 09 Transfer To Business: pending
- 10 Archive And Close: pending

## Missing Artifacts

- `.daedalus/task-card.md`: incomplete
- `.daedalus/decision-log.md`: missing accept/defer/reject decision

## Recent Transitions

- 2026-05-01T09:00:00Z: entered 01-clarify-goal because new repo learning task was accepted
```

`.daedalus/todo.md` demo：

```markdown
# Todo

## Doing

- [ ] 01 Clarify Goal
  - [ ] Ask user to state the real business problem.
  - [ ] Convert vague learning intent into acceptance criteria.
  - [ ] Define repo selection constraints.

## Next

- [ ] 02 Explain First Principles
  - [ ] Identify core concepts behind the target capability.
  - [ ] Explain why production constraints make this problem hard.

- [ ] 03 Select Repo
  - [ ] Collect up to 3 candidate repos.
  - [ ] Compare learning density, runnability, and demo potential.

## Blocked

- [ ] Need user to confirm available weekly learning time.

## Done

- [x] Initialize workspace from repo learning templates.

## Cancelled

- [x] Skip broad survey of unrelated repos.
```

`.daedalus/long-context.md` demo：

```markdown
# Long Context

## Goal
学习插件化任务调度，并迁移到自己的 Agent 工具链。

## Current Stage
01-clarify-goal

## Done
- 用户确认现实问题：工具组合逻辑难以扩展。
- 已确定最小输出物：架构图、mini demo、业务方案。

## Doing
- 澄清 repo 选择约束。

## Next
- 进入 first principles 解释。

## Key Decisions
- mini demo 只复现插件注册和调度，不做完整 UI 或持久化。

## Open Questions
- 用户每周可投入时间未确认。

## Files And Artifacts
- .daedalus/task-card.md
- .daedalus/todo.md
- .daedalus/decision-log.md
```

`.daedalus/artifact-index.md` demo：

```markdown
# Artifact Index

## Goal And Planning

- `.daedalus/task-card.md`: 学习任务卡。
- `.daedalus/decision-log.md`: 接受/延后/拒绝任务的关键判断。

## Stage Notes

- `notes/first-principles.md`: 目标能力背后的第一性原理。
- `notes/architecture.md`: 架构分析笔记。
- `notes/code-reading.md`: 核心代码阅读笔记。

## Diagrams

- `diagrams/core-flow.md`: 核心链路 mermaid 图。

## Demo

- `demo/`: mini demo 代码。
- `demo/README.md`: demo 运行方式和验收用例。

## Business Transfer

- `business-application.md`: 业务迁移方案。
```

`.daedalus/decision-log.md` demo：

```markdown
# Decision Log

## 2026-05-01 Accept Learning Task

### Context
用户需要解决 Agent 工具组合扩展困难的问题。

### Decision
接受该 repo learning task，先聚焦插件化任务调度。

### Rationale
- 现实问题明确。
- 能产出 mini demo。
- 与长期 Agent 工程能力主线一致。

### Consequences
- 暂不学习 UI、部署和多租户权限。
- 需要选择一个模块边界清晰、可本地运行的 repo。
```

### 3. 定义 10 阶段状态机

[`repo-learning-coach`](.claude/skills/repo-learning-coach/SKILL.md) 已经描述了 10 个阶段，但还需要落成可记录、可检查、可恢复的状态机。否则 Agent 很容易在长对话中跳阶段、重复阶段或忘记退出条件。

状态机使用 `state.toml` 维护，而不是 JSON。原因是学习状态需要人类长期阅读和手工修正，TOML 的注释、分段和 diff 可读性更适合这个场景。

同时生成 `.daedalus/state.md`。`state.toml` 是事实源，`state.md` 是给 Agent 看的摘要。`.daedalus/CLAUDE.md` 应引用 `@state.md`，而不是直接引用 `@state.toml`。这样 Agent 不需要每次解析完整 TOML，也能快速知道当前阶段、下一步、缺失产物和最近状态转换。

每个阶段建议在 `state.toml` 中定义：

- `stage_id`：例如 `01-clarify-goal`。
- `status`：例如 `pending`、`active`、`blocked`、`done`、`skipped`。
- `entry_criteria`：进入该阶段前必须满足什么。
- `exit_criteria`：完成该阶段必须产出什么。
- `artifacts`：该阶段应该生成或更新哪些文件。
- `quality_signals`：如何判断这一阶段的教学引导是否有效。

状态机还应记录 transitions，例如 `enter`、`block`、`complete`、`skip`、`resume`。每一次转换都应记录时间、原因、操作者和必要 metadata。你贴出的 `telemetry` 思路可以保留，但建议先保持轻量：只记录对学习流程有帮助的指标，不要过早做复杂评分系统。

完成标准：CLI 可以根据 `state.toml` 校验状态一致性；Agent 可以通过 `state.md` 快速判断当前应该加载哪个阶段 prompt、能不能进入下一阶段、缺失哪些产物。

### 4. 规划 Rust CLI 确定性更新 state.toml

`.daedalus/state.toml` 不应该长期依赖 Agent 用文本替换维护。状态转换属于确定性逻辑，适合放进 `crates` 里的 Rust CLI。CLI 使用 `toml_edit` 更新 TOML，确保修改字段时尽量保留注释、顺序和人工编辑过的结构。每次更新 `state.toml` 后，CLI 都应重新生成 `.daedalus/state.md`。

建议 CLI 的第一版只做几件小而确定的事：

- `state init <task-dir>`：从 `system/templates/repo/state.toml` 初始化 `<task-dir>/.daedalus/state.toml`。
- `state enter <stage-id>`：进入某阶段，更新 `current_phase`、stage status，并追加 transition。
- `state complete <stage-id>`：完成某阶段，检查 exit criteria 是否被标记满足。
- `state block <stage-id> --reason ...`：标记阻塞并记录原因。
- `state resume <stage-id>`：从 paused/blocked 状态恢复。
- `state render <task-dir>`：从 `.daedalus/state.toml` 生成 `.daedalus/state.md`。
- `state validate <task-dir>`：检查 WIP、当前阶段、必需字段、阶段状态是否一致。

CLI 不应该替 Agent 做教学判断，只负责维护状态结构的一致性。比如“是否真的理解架构”仍由 Agent 和用户判断；CLI 只记录该阶段是否完成、由谁完成、产物是否存在。

第一版可以先放在 `crates` 下，例如未来的 `crates/daedalus-cli`。实现上用 `toml_edit::DocumentMut` 读取并修改 `state.toml`，避免 serde 反序列化再序列化导致注释丢失。

完成标准：Agent 不直接手改 `.daedalus/state.toml` 的关键状态字段，也不直接编辑 `.daedalus/state.md`；阶段流转由 CLI 更新 TOML 并重新生成 Markdown 摘要。手写注释和人工补充说明在 CLI 更新后仍然保留。

### 5. 定义 Prompt 加载协议

现在 repo prompt 已经通过 `@system/prompts/common/...` 引用 common prompt，这个分层协议需要正式写清楚。否则后续新增 book、paper、course prompt 时，容易复制 common 内容，导致多处漂移。

协议应明确：

- `common` 只放跨材料通用能力，例如目标澄清、第一性原理、问题路线图、总结、知识归档。
- `repo` 只写 code repo 特有的 delta，例如运行、调试、架构、代码阅读、mini demo。
- 阶段 prompt 顶部使用 `@` 引用需要的 common prompt，而不是让 Agent 再主动读取。
- 如果 common prompt 改了，repo/book/paper/course 自动继承新规则，不重复维护。
- 所有涉及 todo 的 prompt 都应明确：todo list 是动态教学工具，不是一次性计划。Agent 要能创建 todo、拆分 todo、更新优先级、完成 todo、取消过时 todo，并解释变化原因。

完成标准：任何一个阶段 prompt 打开后，都能看出它依赖哪些 common 能力，以及它自己只负责什么差异化职责。

### 6. 定义 01 阶段的输入/输出落盘位置

第 01 阶段会产出最关键的学习边界。如果这些结果只留在聊天里，后续选 repo、提问、运行和 demo 设计都会失去稳定锚点。

建议明确：

- `.daedalus/task-card.md` 写入通用学习目标：现实问题、当前基础、最小输出物、验收标准。
- 通过 Rust CLI 更新 `.daedalus/state.toml`，记录当前阶段为 `01-clarify-goal`，追加 transition，并重新生成 `.daedalus/state.md`。
- `.daedalus/decision-log.md` 记录为什么接受、延后或拒绝该学习任务。
- `.daedalus/artifact-index.md` 记录 01 阶段生成的任务卡和后续需要补的材料。
- 如果用户目标还不够清晰，在层次化 `.daedalus/todo.md` 写入需要用户回答的问题，并在用户回答后及时完成或拆分 todo。

完成标准：完成 01 阶段后，不看聊天记录也能知道这个学习任务为什么存在、要解决什么问题、下一步是否可以进入 repo 选择。

### 7. 定义评估与复盘机制

你后续真正要优化的是 Agent 的教学引导能力，因此每次实践都要留下可复盘数据。否则只能凭感觉改 prompt，很难知道哪一类问题、哪一个阶段真正有效。

建议每个阶段记录：

- Agent 提出的问题是否让用户产生了新的理解。
- 用户在哪些概念、权衡或代码点上卡住。
- 哪些追问有效，哪些追问让用户更困惑。
- 阶段是否按预期产出了文件。
- prompt 是否需要变短、变具体、增加例子或移动到 common。

可以先用轻量人工记录，不急着做 telemetry 自动化。等流程跑过几次后，再决定哪些指标值得进入确定性逻辑。

完成标准：每跑完一个阶段，都能回答“这次教学哪里有效、哪里无效、下一版 prompt 应该改什么”。

## Recommended Minimal Flow Before 01

1. 先创建一个空的学习任务模板，而不是马上开始真实任务。
2. 用一个假想 repo 学习目标做 dry run，只跑 01 阶段，不进入选 repo。
3. 检查 01 阶段是否能稳定产出：现实问题、当前基础、验收标准、repo 选择约束、mini demo 方向、暂不学习范围。
4. 根据 dry run 结果调整 `clarify-goal`、`gatekeeper`、`01-goal-aligner` 三个 prompt。
5. 再开启真实 01 阶段。

## Why This Matters

你真正要迭代的不是单个 prompt，而是一个长周期学习系统。01 阶段之前最重要的是把“状态、产物、恢复、评估”固定住；这样每次实践都能把经验沉淀回 prompt、template 或未来的 deterministic logic，而不是散落在聊天记录里。
