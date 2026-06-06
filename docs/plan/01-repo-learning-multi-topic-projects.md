# Repo Learning 多专题学习项目实现方案

> Date: 2026-05-23
> Status: done (v1 implemented)

## Summary

daedalus 当前的 repo learning 模型把一个 workspace 默认建模成一个学习任务，并让这个任务按 10 个 stage 从目标澄清走到知识归档。这对“学习一个明确主题”有效，但对长期学习一个大型 repo 不够自然。

真实学习场景里，repo 不是一个单一主题，而是一个长期素材库。例如学习 Codex 时，用户可能先学工具系统和权限系统，之后继续学 sub agent 调度、prompt engineering、context engineering、TUI 渲染等。每个专题都需要自己的 North Star、问题路线、源码证据、demo 和业务迁移；同时它们又应该共享源码、运行环境、架构地图、术语表和已验证结论。

本方案建议把 repo learning 从：

```text
一个 repo -> 一个学习任务 -> 一条 10-stage 流水线
```

升级为：

```text
一个 repo/source -> 一个 Learning Project -> 多个 Topic Track
Topic Track 各自走 stage gates
Project Shared Context 沉淀跨 topic 的 verified knowledge
```

核心原则：**学习素材与学习目标分离，专题状态与共享知识分离，探索草稿与可复用证据分离。**

## Architecture Decision

daedalus 是本地项目，不需要长期兼容旧 workspace 结构。多专题改造应采用 **breaking migration**：

```text
新架构成为唯一架构。
旧 workspace 通过一次性迁移脚本升级。
CLI、prompt、validate 不长期保留 legacy 分支。
```

这个选择的收益是：

- 状态模型更简单，不需要同时支持 single-topic 和 multi-topic 两套语义。
- `state` 命令、resume 规则、validate 规则都只面向新结构。
- 后续学习任务不会继续生成旧布局。
- 旧数据仍可保留，但通过迁移脚本进入新布局，而不是由运行时兼容逻辑兜底。

## First Principles

### 1. 学习素材不是学习目标

一个 repo 只是现实系统的载体。用户真正要获得的是围绕某个生产问题的可迁移能力。

```text
repo = source material
topic = learning objective
artifact = proof of learned transfer
```

如果把 repo 等同于学习任务，学习过程就会自然膨胀成“读懂整个 repo”。这会破坏 daedalus 已经建立的 outcome-map 原则。

### 2. 每个专题都需要完整闭环，但不需要重复所有成本

每个 topic 都应该回答：

```text
为什么学？
要解决什么现实问题？
源码里哪些设计保护了什么不变量？
最小 demo 怎么验证？
如何迁移到业务？
学完后沉淀什么知识？
```

但它不应该重复：

```text
重新选择同一个 repo
重新拉源码
重新建立全局 runbook
重新画全局架构图
重新解释已经验证过的术语和通用链路
```

所以 stage gate 应该存在于 topic 内，但某些 stage 可以继承 project-level shared evidence。

### 3. 共享层只能放 verified knowledge

如果 shared context 混入探索草稿，就会污染后续 topic。后续 topic 会把未经验证的猜测当作事实继承，学习质量会快速下降。

因此：

```text
topic notes = 用户回答、假设、源码验证、专题结论
shared context = 多 topic 可复用的已验证结论
```

只有满足来源清楚、边界清楚、迁移限制清楚的结论，才能从 topic 提升到 shared。

### 4. 一次只能 active 一个专题

daedalus 已经有 WIP=1 的工作原则。多 topic 不应该把它打破，而应该细化：

```text
workspaces/02-learning 中最多一个 active Learning Project
每个 Learning Project 中最多一个 active Topic Track
```

这样既允许一个长期 repo 学习项目承载多个专题，也不会让用户同时在多个认知路径里迷路。

### 5. 文件系统必须能恢复学习状态

多专题模型不能依赖聊天记忆。用户应该能打开项目目录后，通过少量文件恢复：

```text
这是哪个长期 repo 学习项目？
当前 active topic 是哪个？
这个 topic 走到哪个 stage？
它继承了哪些 shared evidence？
它完成后会反向更新哪些 shared artifacts？
```

## Current Diagnosis

当前实现有这些稳定点：

- `workspaces/02-learning/<task>` 是 active learning workspace。
- `.daedalus/state.toml` 是机器可读状态源。
- `.daedalus/outcome-map.md` 是当前任务的终点地图。
- `.daedalus/todo.md` 是路径看板。
- `guides/` 和 `notes/` 已经支持阶段目录化。
- CLI 的 stage 状态机已经能处理 `pending / active / blocked / paused / done`。

当前限制是：

- 一个 workspace 只能表达一个 current topic。
- `state.toml` 的 `[[stages]]` 是单条 10-stage 路径，不能表达多个 topic 的独立进度。
- `source/`、`runbook`、`architecture`、`glossary` 等共享信息没有 project-level 位置。
- 已验证结论无法被后续 topic 明确引用、继承、刷新或废弃。
- `daedalus validate` 只知道 task，不知道 topic。

## Target Model

### Concept Model

```text
LearningProject
  表示围绕一个 repo/source 的长期学习计划。
  负责共享源码、共享运行环境、共享证据和 topic board。

SharedContext
  表示跨 topic 可复用的 verified knowledge。
  只接受来源明确、边界明确、迁移限制明确的结论。

TopicTrack
  表示一个具体学习专题。
  每个 topic 有自己的 North Star、stage state、notes、guides、demo 和业务迁移。

EvidenceReference
  表示 topic 对 shared evidence 或其他 topic 结论的显式引用。
  引用时必须说明用途和是否需要重新验证。
```

### Lifecycle Model

```text
Project lifecycle:
active -> completed / abandoned

Topic lifecycle:
planned -> active -> completed / abandoned
planned -> skipped
active -> blocked -> active
```

第一版可以复用现有 stage state：

```text
pending / active / blocked / paused / done
```

但 topic lifecycle 和 stage status 不要混为一谈：

```text
topic.lifecycle 描述专题是否还要继续学。
stage.status 描述专题内部某个阶段是否完成。
```

## Filesystem Design

建议新结构如下：

```text
workspaces/02-learning/openai-codex-cli-deep-learning/
  CLAUDE.md

  .daedalus/
    state.toml
    state.md
    project-map.md
    topic-board.md
    outcome-map.md
    todo.md
    artifact-index.md
    decision-log.md
    validation-log.md
    long-context.md

  source/
    pull_source.sh
    README.md
    codex/

  shared/
    README.md
    runbook.md
    architecture-map.md
    source-index.md
    glossary.md
    evidence-registry.md
    transfer-patterns.md

  topics/
    tools-permissions/
      .daedalus/
        state.toml
        state.md
        outcome-map.md
        todo.md
        artifact-index.md
        decision-log.md
        validation-log.md
        long-context.md
      guides/
      notes/
      demo/

    subagent-scheduling/
      .daedalus/
      guides/
      notes/
      demo/

    prompt-context-engineering/
      .daedalus/
      guides/
      notes/
      demo/
```

### Project Root Responsibilities

Project root 保存长期学习计划和共享材料：

- `source/`：源码缓存和拉取脚本。
- `shared/runbook.md`：基础运行环境和通用调试入口。
- `shared/architecture-map.md`：全局架构地图，只放跨 topic 稳定边界。
- `shared/source-index.md`：源码模块索引，帮助 topic 快速定位。
- `shared/glossary.md`：术语表，例如 `Rollout`、`ToolCall`、`ApprovalRequirement`。
- `shared/evidence-registry.md`：跨 topic verified evidence。
- `.daedalus/project-map.md`：长期学习项目地图。
- `.daedalus/topic-board.md`：topic 列表、状态、继承关系和 active topic。

### Topic Track Responsibilities

每个 topic 保存自己的完整学习闭环：

- `.daedalus/outcome-map.md`：专题终点地图。
- `.daedalus/todo.md`：专题路径看板。
- `guides/`：Agent 引导和问题地图。
- `notes/`：用户回答、源码证据和专题结论。
- `demo/`：专题最小 demo。
- `notes/09-biz-solver/README.md`：专题业务迁移。
- `artifact-index.md`：专题归档索引。

### Shared Evidence Registry

`shared/evidence-registry.md` 建议结构：

```markdown
# Evidence Registry

## E-001: Tool call result 会回灌到模型上下文

- 状态：verified
- 来源 topic：tools-permissions
- 源码证据：
  - `core/src/...`
- 验证方式：
  - 用户运行 / test / source reading
- 结论：
  - ...
- 适用边界：
  - ...
- 迁移限制：
  - ...
- 被引用：
  - subagent-scheduling：用于判断子 agent tool result 是否进入同一 history
```

引用 shared evidence 时，topic note 必须写：

```markdown
## Inherited Evidence

- `E-001`
  - 本专题用途：
  - 是否需要重新验证：
  - 如果不重新验证，理由：
```

## State Model

### Project State

项目根 `.daedalus/state.toml` 只表示 workspace/project 级生命周期，不再承载某个专题的 10-stage 进度：

```toml
[task]
name = "openai-codex-cli-deep-learning"
kind = "repo-learning"
lifecycle = "active"
workspace_bucket = "02-learning"
current_phase = "project"
next_action = "继续 active topic: tools-permissions"

[project]
mode = "multi-topic"
active_topic = "tools-permissions"
source_kind = "repo"
source_name = "openai/codex"

[[topics]]
slug = "tools-permissions"
title = "工具系统与权限系统"
lifecycle = "active"
path = "topics/tools-permissions"
inherits = ["shared/runbook.md", "shared/architecture-map.md"]

[[topics]]
slug = "subagent-scheduling"
title = "Sub Agent 调度系统"
lifecycle = "planned"
path = "topics/subagent-scheduling"
inherits = ["shared/runbook.md"]
```

项目根不再保留 legacy `[[stages]]`。stage 只存在于 topic state 中。这样可以避免“project 当前阶段”和“topic 当前阶段”同时存在造成语义冲突。

### Topic State

每个 topic 使用自己的 `topics/<slug>/.daedalus/state.toml`：

```toml
[topic]
slug = "tools-permissions"
title = "工具系统与权限系统"
kind = "repo-learning-topic"
parent_project = "../.."
lifecycle = "active"
current_phase = "08-demo-coder"
next_action = "继续 Slice 6 Agent Orchestrator。"

[[inherited_evidence]]
id = "E-001"
source = "../../shared/evidence-registry.md"
usage = "作为本专题 tool result 回灌的前置结论。"
reverify = false

[[stages]]
id = "01-goal-aligner"
title = "对齐专题学习目标"
status = "done"
required_artifacts = [".daedalus/task-card.md", ".daedalus/outcome-map.md"]

[[stages]]
id = "02-repo-scout"
title = "确认专题学习素材"
status = "done"
required_artifacts = ["notes/02-repo-scout/README.md"]
completion_mode = "inherited"
inherited_from = "../../shared/source-index.md"
```

`completion_mode` 是建议新增的轻量字段：

```text
direct：本 topic 直接产出。
inherited：继承 shared 或已有 topic 的证据。
not-applicable：该 stage 对本 topic 不适用，但必须写明原因。
```

这样每个 topic 可以“走 stage gates”，但不用机械重复所有工作。

## CLI Design

### Executable Impact

当前 crate 暴露两个可运行程序：

```text
daedalus      -> 确定性状态流转、初始化、校验、关闭、迁移
daedalus-tui  -> 只读学习驾驶舱，展示当前 workspace / task 状态
```

多 topic 改造后，两个程序都必须调整，但职责不同：

```text
daedalus 是 source of truth operator。
daedalus-tui 是 project/topic 状态的只读投影。
```

也就是说，所有创建 topic、激活 topic、迁移旧 workspace、完成 stage、关闭 topic/project 的写操作仍必须走 `daedalus`。`daedalus-tui` 不直接改状态，只负责把 project -> active topic -> topic stage 的位置展示清楚。

### New Commands

建议新增 `daedalus topic` 命令组：

```text
daedalus topic new <slug> --title <title> --project-dir <path>
daedalus topic list --project-dir <path>
daedalus topic activate <slug> --project-dir <path>
daedalus topic complete <slug> --project-dir <path> --reason <reason>
daedalus topic abandon <slug> --project-dir <path> --reason <reason>
daedalus topic validate <slug> --project-dir <path>
```

第一版行为：

- `topic new`：创建 `topics/<slug>`，复制 topic 模板，写入 topic state。
- `topic activate`：更新 project state 的 `project.active_topic`，并确保同 project 内只有一个 active topic。
- `topic complete`：校验 topic required artifacts，标记 topic lifecycle completed，但不移动 topic 目录。
- `topic validate`：校验 topic 文件结构、state、required artifacts、inherited evidence 链接。

### Command Model After Migration

迁移完成后，现有 stage 命令默认操作 active topic，而不是 project root：

```text
daedalus state enter ...
daedalus state complete ...
daedalus task complete ...
```

语义调整为：

- `daedalus init repo-learning <project-name> --topic <topic-slug> --title <topic-title>`：创建一个 Learning Project，并自动创建第一个 topic。
- `daedalus state enter/complete/block/resume`：解析 active topic，并操作 topic state。
- `daedalus task complete/abandon`：关闭整个 Learning Project，要求所有 active topic 已完成、放弃或显式暂停。
- `daedalus topic complete/abandon`：关闭单个 topic。

如需指定 topic：

```text
daedalus state enter 06-code-reader --topic tools-permissions
daedalus state complete 06-code-reader --topic tools-permissions
```

当 cwd 位于 `topics/<slug>` 内时，默认操作该 topic state。

### `daedalus` Required Changes

`daedalus` 需要承担新模型的确定性写入：

- `init repo-learning`：创建 project + initial topic，而不是单个 task stage tree。
- `state enter/complete/block/resume/rollback`：解析 topic state，再执行 stage transition。
- `task complete/abandon`：关闭 project，要求 topic 状态满足关闭条件。
- `topic new/list/activate/complete/abandon/validate`：管理 topic lifecycle。
- `migrate repo-learning-multi-topic`：一次性迁移旧 workspace。
- `validate`：先校验 project，再校验 active topic；必要时支持 `--all-topics`。
- `state render`：需要区分 project state render 和 topic state render。

命令行为建议：

```text
daedalus validate <project>              -> 校验 project + active topic
daedalus validate <project> --all-topics -> 校验 project + 所有 topics
daedalus state render                    -> 渲染 active topic state.md
daedalus state render --project          -> 渲染 project state.md
daedalus topic activate <slug>           -> 切换 active topic，并更新 topic-board
```

### `daedalus-tui` Required Changes

`daedalus-tui` 当前的心智模型是：

```text
workspace bucket -> task -> stages -> current_phase -> missing artifacts / todo / transitions
```

新模型应该改成：

```text
workspace bucket -> project -> topics -> active topic -> topic stages
```

因此 TUI 的数据层需要从 `TuiTaskSummary / TuiOverview` 演进为：

```text
TuiProjectSummary
  - project name
  - lifecycle
  - bucket
  - active topic
  - topic count
  - active topic progress

TuiTopicSummary
  - slug / title
  - lifecycle
  - current_phase
  - current_status
  - done_stage_count / total_stage_count

TuiProjectOverview
  - project map summary
  - active topic overview
  - shared evidence count / stale warnings
  - topic board

TuiTopicOverview
  - topic outcome-map position
  - current stage progress
  - missing topic artifacts
  - topic todo focus
  - recent topic transitions
```

TUI 页面建议从两层改成三层：

```text
ProjectSelector
  -> ProjectOverview
     -> TopicOverview
```

第一版也可以保持两层，但在 ProjectOverview 中默认展示 active topic：

```text
ProjectSelector
  -> ProjectOverview(active topic embedded)
```

这比直接展示 project root stages 更准确，因为 project root 不再有 10-stage 进度。

TUI 需要改的具体点：

- `enclosing_task_dir` 改为能识别 project dir 和 topic dir。
- `scan_task_summaries` 改为 scan project summaries。
- `load_overview(project_dir)` 读取 project state + active topic state。
- progress gauge 使用 active topic 的 stage progress。
- missing artifacts 使用 active topic 的 required artifacts。
- todo focus 读取 active topic 的 `.daedalus/todo.md`。
- recent transitions 默认显示 active topic transitions，另加 project-level migration / topic switch history。
- selector 列表增加 active topic 列，避免只看到 project 名称。
- 页面文案从 `Task` / `Phase` 调整为 `Project` / `Topic` / `Stage`。

### Workspace Resolution

当前 `workspace_fs::bucket_from_task_dir` 假设 task 目录的父目录就是 `02-learning`。topic 目录不满足这个假设。

需要新增两个解析函数：

```rust
default_project_dir(explicit: Option<PathBuf>) -> Result<PathBuf>
default_topic_dir(explicit_topic: Option<String>, explicit_project: Option<PathBuf>) -> Result<PathBuf>
```

解析优先级：

```text
1. 显式 --topic-dir
2. 当前目录向上查找 topic .daedalus/state.toml，且 kind = repo-learning-topic
3. 当前目录向上查找 project .daedalus/state.toml，读取 active_topic
4. daedalus repo 中唯一 active project 的 active_topic
```

### Validation Rules

Project validate:

- 根目录仍在 `workspaces/02-learning`、`03-completed` 或 `04-abandoned`。
- project state lifecycle 与 bucket 一致。
- `project.active_topic` 指向存在的 topic，除非没有 active topic。
- `shared/` 核心文件存在。
- topic board 中的 topic 路径存在。
- 同一 project 最多一个 topic lifecycle = active。

Topic validate:

- topic state 可解析。
- topic parent project 存在。
- stage active 数量最多 1。
- done stage 的 required artifacts 存在。
- inherited evidence 指向的 `E-xxx` 存在。
- inherited stage 必须说明 inherited source。

## Template Changes

### Project Template

`system/templates/repo` 需要新增：

```text
shared/README.md
shared/runbook.md
shared/architecture-map.md
shared/source-index.md
shared/glossary.md
shared/evidence-registry.md
shared/transfer-patterns.md
topics/.gitkeep
.daedalus/project-map.md
.daedalus/topic-board.md
```

`CLAUDE.md` 需要增加：

```text
处理 multi-topic project 前，先读 project-map、topic-board 和 active topic 的 outcome-map。
共享层只放 verified knowledge。
一次只 active 一个 topic。
topic notes 不得伪装成 shared evidence。
```

### Topic Template

新增：

```text
system/templates/repo-topic/
  .daedalus/state.toml
  .daedalus/state.md
  .daedalus/task-card.md
  .daedalus/outcome-map.md
  .daedalus/todo.md
  .daedalus/artifact-index.md
  .daedalus/decision-log.md
  .daedalus/validation-log.md
  .daedalus/long-context.md
  guides/.gitkeep
  notes/.gitkeep
  demo/.gitkeep
```

topic 的 `outcome-map.md` 要多两个区块：

```markdown
## Inherited Context

- Shared runbook:
- Shared architecture:
- Shared evidence:

## Contribution Back To Project

- 本 topic 完成后预计新增或更新哪些 shared evidence：
- 哪些旧 shared evidence 可能被修正：
```

## Prompt And Skill Changes

CLI 语义变化后，prompt 和 skill 必须同步迁移。否则 AI 会继续按旧模型调用命令，把 project state 当 topic state 写，或者把 topic stage 写到 project root。

这部分应视为一等公民改动，不是文案收尾。

### Agent Instruction Surface

需要全面调整的指令面：

```text
.claude/skills/repo-learning-coach/SKILL.md
system/templates/repo/CLAUDE.md
system/prompts/common/resume.md
system/prompts/common/gatekeeper.md
system/prompts/common/compress-context.md
system/prompts/common/export-knowledge.md
system/prompts/repo/phase1-exploration/01-goal-aligner.md
system/prompts/repo/phase1-exploration/02-repo-scout.md
system/prompts/repo/phase1-exploration/03-socratic-coach.md
system/prompts/repo/phase2-learning/04-debugger-guide.md
system/prompts/repo/phase2-learning/05-arch-analyzer.md
system/prompts/repo/phase2-learning/06-code-reader.md
system/prompts/repo/phase3-practice/07-demo-architecture.md
system/prompts/repo/phase3-practice/08-demo-coder.md
system/prompts/repo/phase3-practice/09-biz-solver.md
system/prompts/repo/phase4-closing/10-archivist.md
system/templates/README.md
crates/docs/repo-learning-stage-state-flow.md
```

迁移原则：

```text
所有“task stage”表述改为“project + active topic + topic stage”。
所有“读取 task outcome-map/todo”改为“先读 project-map/topic-board，再读 active topic outcome-map/todo”。
所有“state command 更新当前任务阶段”改为“state command 更新 active topic 阶段”。
所有“task complete 代表完成 10-archivist”改为“topic complete 完成专题；task complete 关闭 project”。
```

### CLI Command Contract For Agents

迁移后，prompt 和 skill 只能教 AI 使用这套命令契约：

```text
创建长期 repo 学习项目：
daedalus init repo-learning <project-name> --topic <topic-slug> --title <topic-title>

创建新专题：
daedalus topic new <topic-slug> --title <topic-title>

切换 active topic：
daedalus topic activate <topic-slug>

推进当前 active topic 的 stage：
daedalus state enter <stage-id>
daedalus state complete <stage-id>
daedalus state block <stage-id> --reason <reason>
daedalus state resume <stage-id> --reason <reason>

显式推进某个 topic：
daedalus state enter <stage-id> --topic <topic-slug>
daedalus state complete <stage-id> --topic <topic-slug>

渲染状态：
daedalus state render             # active topic
daedalus state render --project   # project root

校验：
daedalus validate                 # project + active topic
daedalus validate --all-topics    # project + all topics
daedalus topic validate <topic-slug>

关闭：
daedalus topic complete <topic-slug> --reason <reason>
daedalus task complete --reason <reason>
```

旧命令不是全部消失，但旧解释必须消失。例如：

```text
daedalus state complete 06-code-reader
```

新解释是：

```text
完成 active topic 的 06-code-reader。
```

不是：

```text
完成 project root 的 06-code-reader。
```

### Stale Instruction Audit

实现时必须做一次全文审计，避免残留旧命令语义。

建议检查关键词：

```text
active task
current task
task stage
current_phase
task-card.md
outcome-map.md
todo.md
state complete
state enter
task complete
10-stage
10 个 stage
workspaces/02-learning
```

不是所有命中都要删除，但每个命中都要判断它描述的是：

```text
project-level 状态
topic-level 状态
旧模型遗留描述
历史文档
```

推荐新增一个脚本或测试：

```text
system/bin/audit-daedalus-agent-instructions
```

最小行为：

```text
rg 旧命令/旧语义关键词 .claude system docs crates/docs
输出需要人工确认的文件和行号
```

更进一步可以做 allowlist，确保旧命令只出现在 migration 文档或历史记录中。

### repo-learning-coach

新增 Multi-Topic Rule：

```text
If a repo learning workspace is a multi-topic project, every resume must locate:
1. project north star
2. active topic
3. topic stage
4. inherited shared evidence
5. contribution expected after topic completion
```

新增 Topic Boundary Gate：

```text
Before starting a source reading or demo design step, declare whether the result belongs to:
- shared context
- current topic notes
- current topic demo
- future topic parking lot
```

### Stage Prompts

`01-goal-aligner`：

- 增加“这是 project 目标还是 topic 目标”的区分。
- topic 目标必须说明继承哪些 project context。

`02-repo-scout`：

- 对 topic 来说，大多数情况下是 `inherited`。
- 只在 topic 需要补充新 repo 或新源码入口时重新执行。

`04-debugger-guide`：

- shared runbook 记录通用启动方式。
- topic runbook 记录专题 trace 入口。

`05-arch-analyzer`：

- shared architecture-map 只放跨 topic 稳定边界。
- topic architecture note 放局部架构和专题路径。

`06-code-reader`：

- 每轮阅读必须声明它补 topic artifact，还是提升 shared evidence。
- 若要提升 shared evidence，必须写明跨 topic 适用边界。

`10-archivist`：

- topic 归档时必须反向检查 shared context：
  - 新增哪些 evidence？
  - 哪些 glossary 需要补？
  - 哪些 architecture-map 需要更新？
  - 哪些 future topics 被发现？

## Migration Strategy

### One-Time Breaking Migration

因为不保留旧逻辑，迁移必须是显式、一次性、可回滚的。建议提供本地脚本：

```text
system/bin/migrate-repo-learning-to-multi-topic
```

使用方式：

```text
system/bin/migrate-repo-learning-to-multi-topic \
  workspaces/02-learning/openai-codex-cli-deep-learning \
  --topic tools-permissions \
  --title "工具系统与权限系统"
```

脚本原则：

- 默认 dry-run，先打印将要移动和生成的文件。
- 正式执行前创建备份目录。
- 只迁移 daedalus 自己的 workspace 文件，不碰 `source/codex` 这类外部源码内容。
- 迁移后运行 `daedalus validate`。
- 迁移失败时保留备份和 migration log，便于手动恢复。

### Migration File Mapping

旧结构到新结构的映射：

```text
source/                  -> source/
notes/                   -> topics/<slug>/notes/
guides/                  -> topics/<slug>/guides/
demo/                    -> topics/<slug>/demo/
.daedalus/task-card.md   -> topics/<slug>/.daedalus/task-card.md
.daedalus/outcome-map.md -> topics/<slug>/.daedalus/outcome-map.md
.daedalus/todo.md        -> topics/<slug>/.daedalus/todo.md
.daedalus/artifact-index.md -> topics/<slug>/.daedalus/artifact-index.md
.daedalus/decision-log.md   -> topics/<slug>/.daedalus/decision-log.md
.daedalus/validation-log.md -> topics/<slug>/.daedalus/validation-log.md
.daedalus/long-context.md   -> topics/<slug>/.daedalus/long-context.md
```

迁移后 project root 新增：

```text
shared/
topics/<slug>/
.daedalus/project-map.md
.daedalus/topic-board.md
.daedalus/state.toml
.daedalus/state.md
```

project root 的旧 `.daedalus/*.md` 不再作为长期状态文件保留；它们会被移动到 topic 内。project root 只保留 project-level state、project-map、topic-board 和 project-level logs。

### Migration Algorithm

脚本流程：

```text
1. 校验目标 workspace 是旧结构：
   - 存在 root .daedalus/state.toml
   - 不存在 topics/
   - 不存在 [project].mode = "multi-topic"

2. 读取旧 state.toml：
   - task.name
   - task.lifecycle
   - task.workspace_bucket
   - task.current_phase
   - [[stages]]
   - [[transitions]]

3. 创建备份：
   - .daedalus-migration-backup/<timestamp>/
   - 保存原始 .daedalus、notes、guides、demo 的副本

4. 创建新目录：
   - shared/
   - topics/<slug>/
   - topics/<slug>/.daedalus/

5. 移动专题产物：
   - notes/guides/demo 移入 topic
   - root .daedalus 的 topic-level 文件移入 topic .daedalus

6. 生成 project-level 文件：
   - .daedalus/state.toml：只包含 project lifecycle 和 topics 列表
   - .daedalus/project-map.md
   - .daedalus/topic-board.md
   - shared/README.md
   - shared/evidence-registry.md
   - shared/source-index.md

7. 生成 topic state：
   - current_phase 继承旧 task.current_phase
   - [[stages]] 继承旧 stages
   - transitions 可以继承旧 transitions，并标记 migrated_from_root = true

8. 渲染 state.md：
   - project state.md
   - topic state.md

9. 运行校验：
   - daedalus validate <project>
   - daedalus topic validate <slug>
```

### Migration Script Shape

第一版可以用 Rust CLI 实现，也可以先用 shell 脚本编排 CLI。为了保持确定性，最终建议落入 Rust CLI：

```text
daedalus migrate repo-learning-multi-topic \
  <old-task-dir> \
  --topic <slug> \
  --title <title> \
  --execute
```

`system/bin/migrate-repo-learning-to-multi-topic` 可以作为薄封装，方便本地直接运行。

## Implementation Phases

### Phase 1: Breaking Filesystem Template

目标：新建 repo learning workspace 只生成新架构，不再生成旧 single-topic 布局。

改动：

- `system/templates/repo` 新增 `shared/`、`topics/`、`project-map.md`、`topic-board.md`。
- `system/templates/repo` 的 root `.daedalus/state.toml` 改为 project state，不再包含 10 个 topic stage。
- 新增 `system/templates/repo-topic`，topic 内部才包含 10-stage state。
- `init repo-learning` 创建 project 后自动创建第一个 topic。
- `validate_workspace` 检查这些 project-level 文件存在。
- `CLAUDE.md` 加入 active topic 恢复规则。

验收：

- `daedalus init repo-learning test --topic first-topic` 后生成 project + topic。
- root state 不包含 topic stages。
- topic state 包含 10-stage learning flow。
- 新测试断言模板包含 `shared/evidence-registry.md`、`.daedalus/topic-board.md` 和 `topics/first-topic/.daedalus/state.toml`。

### Phase 2: `daedalus` Core Command Rewrite

目标：让 CLI 的确定性写入全部面向 project/topic 新结构。

改动：

- `init repo-learning` 支持 initial topic。
- `state` 命令默认解析 active topic。
- `validate` 校验 project + active topic。
- `state render` 支持 project / topic 两种 state.md。
- `task complete/abandon` 改为关闭 project。

验收：

- 在 project root 执行 `daedalus state complete 01-goal-aligner` 会更新 active topic state。
- 在 topic 目录执行同命令会更新当前 topic state。
- project root state 不会出现 topic stage transition。
- `daedalus validate` 能报告 project 问题和 active topic 问题。

### Phase 3: Agent Instruction Migration

目标：让所有 skill、prompt、模板和说明文档使用新 CLI 契约，避免 AI 按旧命令语义操作。

改动：

- 更新 `repo-learning-coach` 的描述和规则。
- 更新 `system/templates/repo/CLAUDE.md`，明确 project/topic state 分工。
- 更新 `resume.md`，恢复时必须定位 project、active topic 和 topic stage。
- 更新所有 repo phase prompts，把 stage 操作绑定到 active topic。
- 更新 `gatekeeper.md`，将 WIP=1 表达为一个 active project + 一个 active topic。
- 更新 `crates/docs/repo-learning-stage-state-flow.md`，从 task-stage flow 改为 project-topic-stage flow。
- 新增 `system/bin/audit-daedalus-agent-instructions` 或等价测试，扫描旧命令语义。

验收：

- 搜索 `active task`、`task stage`、`task complete`、`state complete` 等关键词时，没有未解释的旧模型描述。
- prompt 中不再要求 AI 直接读 root `.daedalus/outcome-map.md` 作为专题状态。
- 所有阶段 prompt 都知道 notes/guides/demo 属于 active topic。
- `repo-learning-coach` 明确禁止在 project root 手写 topic stage 状态。

### Phase 4: Migration Script MVP

目标：把现有旧 workspace 一次性迁移到新结构。

改动：

- 新增 `daedalus migrate repo-learning-multi-topic`。
- 新增 `system/bin/migrate-repo-learning-to-multi-topic` 薄封装。
- 迁移前自动备份。
- 迁移后自动 validate。

验收：

- 当前 Codex learning workspace 可以迁移成 project + `tools-permissions` topic。
- 原 notes/guides/demo 都进入 topic。
- project root 只保留 project-level 文件。
- 迁移后 `daedalus validate` 和 `daedalus topic validate` 通过。

### Phase 5: Topic CLI MVP

目标：CLI 可以创建、激活、校验 topic。

改动：

- 新增 `crates/daedalus-cli/src/domain/topic.rs`。
- 新增 `application/init_topic.rs`、`validate_topic.rs`、`activate_topic.rs`。
- 新增 `interfaces/agent_cli/commands/topic.rs`。
- 新增 `system/templates/repo-topic`。

验收：

- `daedalus topic new subagent-scheduling --title ...` 创建 topic。
- `daedalus topic activate subagent-scheduling` 更新 project active topic。
- 同一 project 不能同时有两个 active topic。
- `daedalus topic validate subagent-scheduling` 能检查 topic artifacts。

### Phase 6: `daedalus-tui` Project / Topic View

目标：TUI 能正确展示 project/topic/stage 三层位置，不再读取 project root stages。

改动：

- 将 `TuiTaskSummary` 改为 `TuiProjectSummary`。
- 将 `TuiOverview` 拆为 project overview + active topic overview。
- selector 列表显示 project、bucket、lifecycle、active topic、topic progress。
- overview 顶部显示 Project / Active Topic / Stage / Status。
- missing artifacts、todo、transitions 默认来自 active topic。
- 增加 shared evidence / topic board 摘要面板。

验收：

- 从 daedalus 根目录启动 `daedalus-tui` 可以看到 project 列表和 active topic。
- 从 topic 目录启动 `daedalus-tui` 直接打开该 topic overview。
- active topic progress 与 topic state 中的 stages 一致。
- TUI 不再把 project root state 当作 10-stage 学习进度。

### Phase 7: Stage Commands Topic-Aware Hardening

目标：现有 `state enter/complete/block/resume` 可以作用于 topic。

改动：

- `state` 命令增加 `--topic` 或 `--topic-dir`。
- cwd 在 topic 目录下时，默认操作 topic state。
- project root 下运行时，默认操作 active topic。

验收：

- 在 topic 目录运行 `daedalus state enter 03-socratic-coach` 操作 topic state。
- 在 project root 运行同命令时，操作 active topic。

### Phase 8: Evidence Registry Automation

目标：让 shared evidence 引用可校验。

改动：

- 增加 evidence id parser。
- `topic validate` 校验 inherited evidence 是否存在。
- `topic complete` 提示需要更新 shared context。

验收：

- topic 引用不存在的 `E-999` 时 validate 失败。
- topic completed 但 `Contribution Back To Project` 为空时给 warning。

## Test Plan

### Unit Tests

- `TopicLifecycle::parse` / `as_str`。
- topic slug sanitize。
- active topic uniqueness。
- inherited evidence id parsing。

### CLI Workflow Tests

- `init_repo_learning_creates_multi_topic_project_files`。
- `init_repo_learning_creates_initial_topic_state`。
- `topic_new_creates_topic_workspace`。
- `topic_activate_updates_project_state_and_blocks_previous`。
- `topic_validate_rejects_missing_required_topic_artifact`。
- `migrate_single_topic_workspace_creates_project_and_topic`。
- `migrate_single_topic_workspace_moves_artifacts_to_topic`。
- `migrate_single_topic_workspace_preserves_stage_state`。
- `state_command_from_project_root_uses_active_topic_state`。
- `state_command_inside_topic_uses_topic_state`。
- `task_complete_rejects_project_with_active_unfinished_topic`。

### TUI Tests

- `tui_scan_project_summaries_reads_active_topic`。
- `tui_load_overview_uses_active_topic_stage_progress`。
- `tui_load_overview_reads_topic_todo_and_missing_artifacts`。
- `tui_from_topic_directory_opens_topic_overview`。
- `tui_selector_displays_project_and_active_topic`。

### Prompt Behavior Tests

用人工验收即可：

- 用户说“继续学习”：Agent 输出 project + active topic + stage + current gap。
- 用户说“新开 Codex prompt engineering 专题”：Agent 创建或建议 topic，而不是污染当前工具权限 topic。
- 用户要求跨 topic 复用结论：Agent 引用 `shared/evidence-registry.md`，并说明是否需要重新验证。
- Agent 要完成阶段时：使用 `daedalus state complete <stage-id>`，并明确这是 active topic stage。
- Agent 要关闭专题时：使用 `daedalus topic complete <topic-slug>`，而不是 `daedalus task complete`。
- Agent 要关闭整个项目时：先确认 topic 状态，再使用 `daedalus task complete`。

### Agent Instruction Audit Tests

- `audit_daedalus_agent_instructions_flags_old_task_stage_language`。
- `audit_daedalus_agent_instructions_allows_migration_docs_to_mention_legacy`。
- `repo_learning_skill_mentions_project_topic_stage_contract`。
- `resume_prompt_reads_project_map_then_active_topic_map`。

## Risks And Trade-Offs

### Risk 1: 目录结构变复杂

代价是真实存在的。解决方式：

- 所有 repo learning 都使用同一套 project/topic 结构。
- 简单学习任务也只是一个 project + 一个 topic，不走旧结构。
- project-map 和 topic-board 必须承担导航责任。

### Risk 2: shared context 被滥用

如果所有东西都放 shared，会变成第二个大杂烩 notes。

控制规则：

- shared 只放 verified knowledge。
- 每条 evidence 必须有 source、boundary、transfer limit。
- topic 草稿不得直接写入 shared。

### Risk 3: 一次性迁移有破坏性

不保留 legacy runtime 兼容会让迁移变成关键路径。

控制方式：

```text
迁移脚本默认 dry-run
迁移前自动备份
迁移日志写入 .daedalus/migration-log.md
迁移后自动 validate
失败时不删除 backup
```

### Risk 4: Topic stage 继承导致偷懒

继承不是跳过学习，而是复用已经验证过的学习证据。

每个 inherited stage 必须说明：

```text
继承了什么？
为什么足够？
本 topic 是否需要补充验证？
如果不验证，风险是什么？
```

## Open Questions

- topic completed 后是否需要移动到 `topics/.completed/`，还是保留在 `topics/<slug>` 仅改 lifecycle？
- `knowledge-base` 应该按 project 归档，还是按 topic 归档，再由 project entry 汇总？
- shared architecture-map 是否应该允许 Mermaid/图片等非 markdown 资产？
- topic 之间是否允许依赖 DAG，还是第一版只支持轻量 `inherits = []`？
- 迁移脚本的备份目录是否默认保留在 workspace 内，还是放到 `/tmp` 并提示路径？

## Recommended First Slice

第一步应该直接收敛文件契约和迁移路径，不保留 legacy 分支：

```text
1. 修改 repo 模板：root 只表示 project，topic 目录才有 10-stage state。
2. 新增 repo-topic 模板。
3. 修改 init repo-learning：创建 project + initial topic。
4. 修改 daedalus state/validate/render：默认操作 active topic。
5. 更新 repo-learning-coach、resume 和所有 repo phase prompts，统一新 CLI 命令语义。
6. 新增指令审计脚本，防止旧命令语义残留。
7. 新增 migrate repo-learning-multi-topic：把现有 Codex workspace 迁移过去。
8. 修改 daedalus-tui：展示 project + active topic + topic stage。
```

这个 slice 的好处是：

- 架构一次性变干净。
- CLI 不需要长期支持两套 workspace 语义。
- 当前 Codex 学习项目可以通过迁移脚本保留数据。
- 后续 topic CLI 和 evidence registry 都建立在唯一结构上。
