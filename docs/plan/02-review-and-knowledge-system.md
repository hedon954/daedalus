# Review Plan 与 Knowledge System 萃取能力实现方案

> Date: 2026-05-23
> Status: done (v1 implemented)

## Summary

daedalus 当前已经支持 repo learning 的主学习闭环：

```text
Learning Project -> Topic Track -> 01..10 stages -> demo / biz transfer / archive
```

但它还没有把“学完以后如何复习”和“如何从学习证据中萃取知识体系”建模成一等能力。现有 `10-reflection` 与 `export-knowledge` 能在关闭阶段做知识归档，但它们更像尾声动作：

- 只在任务收尾时触发。
- 偏向总结和归档，不负责长期掌握。
- 没有复习计划、复习 session、掌握度地图。
- 没有 topic/project 完成后再次启动复习的独立生命周期。
- 没有明确规定知识如何从 topic notes 晋升到 shared，再晋升到 knowledge-base。

本方案建议新增两套横切能力：

```text
Review System
  对已学习 topic 或 project 启动复习计划，持续跟踪掌握度。

Knowledge System Extraction
  从学习产物和复习证据中萃取 concept / invariant / failure mode / trade-off / pattern / relation。
```

核心判断：**复习不是 repo-learning 的第 11 阶段，而是挂载在 completed 或 active 学习对象上的独立生命周期。知识萃取也不是简单归档，而是有证据门槛的 promotion pipeline。**

## Implementation Status

状态：已完成第一版完整落地。

已实现范围：

- Review filesystem templates：`system/templates/review/`。
- Knowledge system templates：`system/templates/knowledge-system/`。
- Project/topic review 入口模板：`.daedalus/reviews/README.md`。
- Shared knowledge-system 入口：`shared/knowledge-system/`。
- Review domain / application / CLI：
  - `daedalus review start`
  - `daedalus review list`
  - `daedalus review show`
  - `daedalus review session start`
  - `daedalus review session complete`
  - `daedalus review complete`
  - `daedalus review abandon`
  - `daedalus review render`
  - `daedalus review validate`
- Knowledge domain / application / CLI：
  - `daedalus knowledge extract`
  - `daedalus knowledge promote --to shared`
  - `daedalus knowledge export --to knowledge-base`
  - `daedalus knowledge list`
  - `daedalus knowledge validate`
- `daedalus validate --reviews --knowledge`。
- TUI read-only Review Focus / Knowledge Focus。
- 当前 Codex learning project 已补充 review / knowledge-system scaffolding。
- 当前 Codex learning project 已补充 review / knowledge-system scaffolding。

仍然刻意不做的范围：

- 不自动生成已验证知识正文。
- 不自动评分用户掌握度。
- 不做后台 scheduler。
- 不自动重构 knowledge-base taxonomy。

这些边界不是未完成，而是产品约束：确定性 CLI 只负责结构、状态、索引和校验；知识内容仍由 Agent coaching + 用户校准产生。

## First Principles

### 1. 学完不等于掌握

一次 topic 完成只能证明用户在当时能沿着材料走完整条路径。长期掌握还需要回答：

```text
不看笔记能不能复述？
能不能重新推导设计？
能不能迁移到新场景？
能不能识别相似但不等价的概念？
能不能解释 trade-off，而不是背结论？
```

所以 daedalus 需要把 completed topic/project 重新作为复习对象，而不是把 learning 目录归档后彻底静态化。

### 2. 复习对象必须是 artifact，不是聊天记忆

daedalus 的基本承诺是 filesystem-first。复习计划不能依赖聊天历史，它必须从这些文件恢复：

```text
project/.daedalus/project-map.md
project/.daedalus/topic-board.md
topic/.daedalus/outcome-map.md
topic/.daedalus/artifact-index.md
topic/notes/
topic/demo/
project/shared/
knowledge-base/
```

复习系统不是“重新总结聊天”，而是“基于已落盘学习证据生成复习路线”。

### 3. 复习不是重新打开学习任务

一个 topic 已经 completed 后，复习不应该把它改回 active，也不应该重走 01..10。否则生命周期会混乱：

```text
topic.lifecycle = completed
review.lifecycle = active
```

这两条生命周期必须分离。学习对象保持已完成，复习计划作为附属对象继续推进。

### 4. 知识萃取需要 evidence promotion

topic notes 里会混合：

- 用户原始假设。
- Agent 校准。
- 源码证据。
- demo 设计。
- review 过程中的错误和修正。

这些不应该直接进入 knowledge-base。知识萃取必须经过 promotion：

```text
topic evidence
  -> shared evidence candidate
  -> shared verified knowledge
  -> knowledge-base entry
```

每次晋升都要说明：

```text
证据来自哪里？
解决什么现实问题？
保护什么不变量？
适用边界是什么？
下次遇到什么场景应复用它？
```

### 5. 复习和萃取互相增强

复习暴露用户的真实掌握缺口，知识萃取把这些缺口转化为可复用结构。

```text
review session 中答错的点
  -> weakness map
  -> 更新 mastery map
  -> 形成更精确的 concept / relation / anti-pattern
  -> 反哺下一次 review plan
```

所以 Review System 和 Knowledge System Extraction 应该一起设计，而不是两个互不相干的功能。

## Target Capabilities

### Review System

支持用户随时对已学习对象启动复习计划：

```text
daedalus review start --topic tools-permissions --mode rebuild
daedalus review start --project . --mode application
```

复习计划可以挂载在：

- 一个 completed topic。
- 一个 active topic 的阶段性 checkpoint。
- 一个 completed project。
- 一个 active project 的 shared context。

第一版推荐主要支持 completed topic 和 completed/active project。active topic 的随堂复习可以由 prompt gate 先做，不必进入 CLI 一等对象。

复习模式至少包括：

| Mode | 目的 | 典型问题 |
| --- | --- | --- |
| `recall` | 检查不看笔记能否复述 | “从第一性原理解释 approval 和 sandbox 的关系。” |
| `rebuild` | 从空白重建设计 | “重新画出 command execution state machine。” |
| `application` | 迁移到新场景 | “如果给浏览器插件加本地命令执行，权限模型怎么设计？” |
| `weakness-repair` | 针对薄弱点补洞 | “解释 complex parsing fallback 和 multi-segment command 的区别。” |
| `mixed` | 综合复习 | 自动混合 recall / rebuild / application。 |

### Knowledge System Extraction

支持从 topic/project 中萃取知识体系：

```text
daedalus knowledge extract --topic tools-permissions
daedalus knowledge promote --topic tools-permissions --to shared
daedalus knowledge export --project . --to knowledge-base
```

萃取结果不是普通摘要，而是结构化知识图谱草案：

```text
Concept
Invariant
FailureMode
TradeOff
Pattern
AntiPattern
Evidence
Relation
ReviewWeakness
```

这些结构最终可以落到 markdown，但需要保持机器可验证的索引。

## Filesystem Design

### Project-level reviews

整个 repo/project 的复习计划挂在 project 根目录：

```text
workspaces/03-completed/<project>/
  .daedalus/
    reviews/
      README.md
      <review-id>/
        review-plan.md
        state.toml
        mastery-map.md
        question-bank.md
        sessions/
          <session-id>.md
```

如果 project 还在 `02-learning`，也允许创建 project-level review，但默认只针对 shared context，不影响 active topic。

### Topic-level reviews

专题复习计划挂在 topic 内：

```text
topics/<slug>/
  .daedalus/
    reviews/
      README.md
      <review-id>/
        review-plan.md
        state.toml
        mastery-map.md
        question-bank.md
        sessions/
          <session-id>.md
```

理由：

- topic 的复习证据应跟 topic 学习证据靠近。
- project 级复习只聚合跨 topic 能力。
- 不污染原 topic stage state。

### Knowledge extraction outputs

建议新增 project shared 下的知识体系目录：

```text
shared/
  knowledge-system/
    README.md
    concept-map.md
    invariant-map.md
    failure-mode-map.md
    pattern-catalog.md
    relation-map.md
    promotion-log.md
```

topic 内可有候选知识：

```text
topics/<slug>/notes/knowledge-system/
  extraction.md
  concept-candidates.md
  promotion-candidates.md
```

最终进入全局知识库：

```text
knowledge-base/<category>/<entry>.md
```

## State Model

### ReviewPlan

新增领域模型：

```rust
pub struct ReviewPlan {
    pub id: ReviewId,
    pub target: ReviewTarget,
    pub mode: ReviewMode,
    pub lifecycle: ReviewLifecycle,
    pub goal: String,
    pub source_artifacts: Vec<ArtifactRef>,
    pub cadence: Option<ReviewCadence>,
    pub created_at: DateTime,
    pub next_session_at: Option<DateTime>,
}

pub enum ReviewTarget {
    Topic { project_dir: PathBuf, slug: String },
    Project { project_dir: PathBuf },
}

pub enum ReviewMode {
    Recall,
    Rebuild,
    Application,
    WeaknessRepair,
    Mixed,
}

pub enum ReviewLifecycle {
    Planned,
    Active,
    Paused,
    Completed,
    Abandoned,
}
```

### ReviewSession

```rust
pub struct ReviewSession {
    pub id: ReviewSessionId,
    pub review_id: ReviewId,
    pub started_at: DateTime,
    pub completed_at: Option<DateTime>,
    pub status: ReviewSessionStatus,
    pub questions: Vec<ReviewQuestion>,
    pub user_answers: Vec<UserAnswer>,
    pub calibration: Vec<ReviewCalibration>,
    pub mastery_delta: Vec<MasteryDelta>,
}

pub enum ReviewSessionStatus {
    Active,
    Completed,
    Abandoned,
}
```

### MasteryMap

掌握度不是主观评分，而是证据状态：

```rust
pub struct MasteryItem {
    pub id: String,
    pub title: String,
    pub kind: MasteryKind,
    pub level: MasteryLevel,
    pub evidence: Vec<ArtifactRef>,
    pub last_reviewed_at: Option<DateTime>,
    pub next_review_hint: Option<String>,
    pub weakness_notes: Vec<String>,
}

pub enum MasteryKind {
    Concept,
    Invariant,
    Flow,
    TradeOff,
    Pattern,
    ImplementationDetail,
}

pub enum MasteryLevel {
    Unknown,
    Weak,
    Medium,
    Strong,
    Transferable,
}
```

推荐在 markdown 中呈现：

```markdown
| Knowledge Item | Kind | Level | Evidence | Next Review |
| --- | --- | --- | --- | --- |
| ApprovalRequirement vs bypass_sandbox | invariant | strong | demo/design.md, review session 1 | 30d |
```

### KnowledgeItem

```rust
pub struct KnowledgeItem {
    pub id: String,
    pub title: String,
    pub kind: KnowledgeKind,
    pub reality_constraint: String,
    pub core_approach: String,
    pub trade_off: String,
    pub transferable_pattern: String,
    pub boundaries: Vec<String>,
    pub evidence: Vec<ArtifactRef>,
    pub related: Vec<KnowledgeRelation>,
    pub promotion_status: PromotionStatus,
}

pub enum KnowledgeKind {
    Concept,
    Invariant,
    FailureMode,
    TradeOff,
    Pattern,
    AntiPattern,
}

pub enum PromotionStatus {
    Candidate,
    SharedVerified,
    ExportedToKnowledgeBase,
}
```

## TOML Layout

### Review state

`reviews/<review-id>/state.toml`：

```toml
[review]
id = "2026-05-23-tools-permissions-rebuild"
target_type = "topic"
target = "tools-permissions"
mode = "rebuild"
lifecycle = "active"
goal = "不看原笔记重建 Codex 工具权限系统的核心状态机。"
created_at = "2026-05-23 20:00:00"
next_action = "开始第一轮 rebuild session。"

[[source_artifacts]]
path = "../../outcome-map.md"
kind = "outcome-map"

[[source_artifacts]]
path = "../../../demo/design.md"
kind = "demo-design"

[[sessions]]
id = "2026-05-23-session-1"
status = "completed"
path = "sessions/2026-05-23-session-1.md"

[[transitions]]
action = "start"
timestamp = "2026-05-23 20:00:00"
actor = "daedalus-cli"
reason = "用户启动 topic rebuild review。"
```

### Knowledge promotion log

`shared/knowledge-system/promotion-log.md` 或后续 `promotion.toml`：

```toml
[[promotions]]
item_id = "agent-cli-command-approval-scope"
from = "topics/tools-permissions/notes/knowledge-system/extraction.md"
to = "shared/knowledge-system/invariant-map.md"
status = "shared-verified"
evidence = [
  "topics/tools-permissions/demo/design.md",
  "topics/tools-permissions/notes/06-code-reader/runtime-request-assembly.md",
  "topics/tools-permissions/.daedalus/reviews/2026-05-23/sessions/2026-05-23-session-1.md",
]
timestamp = "2026-05-23 20:30:00"
```

第一版可以只用 markdown 表格，等操作变多再引入 TOML。

## CLI Design

### Review commands

新增 command group：

```text
daedalus review start
daedalus review list
daedalus review show
daedalus review session start
daedalus review session complete
daedalus review complete
daedalus review abandon
daedalus review render
daedalus review validate
```

建议接口：

```bash
daedalus review start --topic tools-permissions --mode rebuild --goal "重建工具权限状态机"
daedalus review start --project . --mode application --goal "把 Codex 权限模式迁移到自有 Agent CLI"

daedalus review list
daedalus review show <review-id>

daedalus review session start <review-id>
daedalus review session complete <review-id> --reason "用户回答与校准已写入 session 文件"

daedalus review complete <review-id> --reason "至少一个 session 已完成且包含用户回答和校准"
daedalus review abandon <review-id> --reason "不再需要"
daedalus review render <review-id>
daedalus review validate <review-id>
```

解析规则：

- 在 project root 下，`--topic <slug>` 定位 `topics/<slug>`。
- 在 topic 目录下，省略 `--topic` 时默认当前 topic。
- `--project` 显式创建 project-level review。
- completed topic/project 可启动 review。
- active project 可启动 project review，但 active topic 不应被 review 命令改状态。

### Knowledge commands

新增 command group：

```text
daedalus knowledge extract
daedalus knowledge promote
daedalus knowledge export
daedalus knowledge list
```

建议接口：

```bash
daedalus knowledge extract --topic tools-permissions
daedalus knowledge promote --topic tools-permissions --to shared
daedalus knowledge export --project . --to knowledge-base
daedalus knowledge list --project .
```

第一版可以先不做自动抽取逻辑，只生成结构化模板和 prompt navigation：

- `extract` 创建 `notes/knowledge-system/extraction.md`。
- `promote` 创建或更新 `shared/knowledge-system/*`。
- `export` 创建候选 knowledge-base entry，必须人工确认后才能标记 exported。

不要第一版就让 CLI 用 LLM 自动写知识正文。daedalus 的确定性 CLI 应负责文件结构、状态、校验和索引；知识内容仍由 Agent prompt + 用户校准完成。

## Prompt Design

### 新增 common prompt

```text
system/prompts/common/review-guidance.md
system/prompts/common/knowledge-system-extraction.md
```

`review-guidance.md` 负责：

- 从 target artifacts 生成 3-7 个复习问题。
- 强制先让用户回答，不先讲解。
- 强制问题从业务目标、现实制约、naive 失败、核心不变量、实现机制、trade-off、最佳实践对比和迁移模式出发。
- 按 recall / rebuild / application / weakness-repair 选择问题形态。
- 根据用户回答更新 mastery-map。
- 把错误记录为 weakness，不羞辱用户、不直接替用户完成。

`knowledge-system-extraction.md` 负责：

- 从已验证 evidence 里抽取 concept / invariant / failure mode / trade-off / pattern。
- 每条知识必须补齐业务目标、现实制约、naive 失败、核心抽象/不变量、实现机制、trade-off、最佳实践对比、迁移模式和复习题。
- 标记每个知识点的 evidence 和 boundary。
- 判断能否 promotion。
- 只把已验证内容推进到 shared 或 knowledge-base。

### 更新 existing prompts

需要更新：

```text
.claude/skills/repo-learning-coach/SKILL.md
system/prompts/common/resume.md
system/prompts/common/export-knowledge.md
system/prompts/repo/phase4-closing/10-reflection.md
```

关键规则：

- 用户说“复习一下”“帮我回顾”“启动复习计划”时，不要进入 repo-learning 10-stage，而是进入 Review System。
- 复习 completed topic/project 时，不修改原 topic/project lifecycle。
- `10-reflection` 完成 topic 时要提示是否创建 review seed，但不自动创建长期计划。
- `export-knowledge` 不再只输出单篇知识条目，还要能输出 relation map 和 promotion candidate。

### Skill description 更新

`repo-learning-coach` 的 description 当前只覆盖 10-stage。需要扩展为：

```text
Guides repo learning projects, topic tracks, post-learning review plans, and verified knowledge extraction.
```

并新增触发：

```text
Use when the user asks to review, revisit, test memory, rebuild understanding, extract a knowledge system, or promote verified learning to shared/knowledge-base.
```

## Template Design

### Review templates

新增：

```text
system/templates/review/
  README.md
  review-plan.md
  state.toml
  mastery-map.md
  question-bank.md
  sessions/.gitkeep
```

`review-plan.md`：

```markdown
# Review Plan

## Target
- Type:
- Path:
- Source artifacts:

## Review Goal

## Mode

## What To Rebuild Without Notes

## Weakness Hypotheses

## Session Plan

## Stop Rules
```

`mastery-map.md`：

```markdown
# Mastery Map

| Item | Kind | Level | Evidence | Weakness | Next Review |
| --- | --- | --- | --- | --- | --- |
```

`question-bank.md`：

```markdown
# Question Bank

## Recall

## Rebuild

## Application

## Weakness Repair
```

`sessions/<session-id>.md`：

```markdown
# Review Session

## Navigation
- Target:
- Mode:
- Goal:

## Questions

## User Answers

## Calibration

## Mastery Delta

## Next Review
```

### Knowledge system templates

新增：

```text
system/templates/knowledge-system/
  extraction.md
  concept-map.md
  invariant-map.md
  failure-mode-map.md
  pattern-catalog.md
  relation-map.md
  promotion-log.md
```

这些模板既可用于 `shared/knowledge-system/`，也可用于 topic 的 `notes/knowledge-system/`。

## Validation Rules

### Review validation

已实现 `daedalus review validate <review-id>` 和 `daedalus validate --reviews`。

第一版校验：

- `state.toml` 存在且 `target` 可解析。
- `review-plan.md`、`mastery-map.md`、`question-bank.md` 存在。
- `sessions` 中引用的 session 文件存在。
- completed review 至少有一个 completed session。
- completed session 必须包含 `User Answers` 和 `Calibration`。

不做内容语义校验，避免 deterministic CLI 越界。

### Knowledge validation

已实现 `daedalus knowledge validate` 和 `daedalus validate --knowledge`。

第一版校验：

- promotion candidate 必须有 evidence path。
- shared verified item 的 evidence path 必须存在。
- exported item 必须有 knowledge-base target path。
- knowledge-base entry 必须包含 reality constraint / trade-off / transferable pattern / source。

## TUI Design

当前 TUI 已显示 project/topic。新增最小支持：

```text
Review Focus
  active review id
  target
  mode
  next session
  weak items

Knowledge Focus
  promotion candidates
  shared verified items
  pending exports
```

不要第一版做完整交互。TUI 只读展示即可，操作继续由 CLI 驱动。

## Interaction Protocol

### 用户说“复习 tools-permissions”

Agent 应：

1. 定位 project 和 topic。
2. 检查是否已有 active review。
3. 如果没有，建议 mode，或默认 `mixed`。
4. 运行或建议运行 `daedalus review start --topic tools-permissions --mode mixed`。
5. 读取 `review-plan.md`。
6. 先问 1-3 个问题，不直接总结。

### 用户说“复习整个 Codex 项目”

Agent 应：

1. 定位 project-level artifacts。
2. 读取 topic-board 和 shared context。
3. 创建 project-level review。
4. 问跨 topic 的 relation / application 问题。

### 用户说“萃取知识体系”

Agent 应：

1. 读取 topic/project artifact-index。
2. 读取 source evidence、demo validation、review sessions。
3. 输出 knowledge candidates，而不是直接写 knowledge-base 正文。
4. 要求用户确认或校准。
5. 再 promotion 到 shared / knowledge-base。

## Implementation Plan

### Phase 0: 方案与边界确认

产出：

- 本技术方案。
- 明确 Review System 是独立生命周期，不是第 11 阶段。
- 明确 CLI 只管理结构和状态，不自动生成知识正文。

验收：

- 用户确认 review target、review mode、knowledge promotion pipeline 的方向。

### Phase 1: Review filesystem templates

实现：

- 新增 `system/templates/review/`。
- 新增 review plan / mastery map / question bank / session 模板。
- 在 repo topic/project 模板中补充 reviews 入口说明。

验收：

- 模板可复制生成一个完整 review skeleton。
- markdown 入口能让 Agent 恢复复习状态。

### Phase 2: Review domain + CLI

实现：

- 新增 `domain/review.rs`。
- 新增 `application/review.rs`。
- 新增 `interfaces/agent_cli/commands/review.rs`。
- 支持 `review start/list/show/session start/session complete/abandon/render`。

验收：

- completed topic 可以创建 review。
- project 可以创建 review。
- review 不修改 topic/project lifecycle。
- review session complete 会更新 review state。

### Phase 3: Review prompt integration

实现：

- 新增 `system/prompts/common/review-guidance.md`。
- 更新 `repo-learning-coach` trigger 和 workflow。
- 更新 `resume.md`：如果当前请求是复习，不输出 stage navigation，而输出 review navigation。

验收：

- 用户说“复习一下 tools-permissions”，Agent 不进入 08-demo-coder，而是启动/恢复 review plan。
- Agent 先问问题，不直接总结。

### Phase 4: Knowledge system templates

实现：

- 新增 `system/templates/knowledge-system/`。
- 给 project `shared/` 模板增加 `knowledge-system/` 入口。
- 给 topic notes 增加 `notes/knowledge-system/` 约定。

验收：

- 一个 topic 能生成 extraction candidates。
- shared 能保存 concept/invariant/failure-mode/pattern/relation map。

### Phase 5: Knowledge CLI

实现：

- 新增 `domain/knowledge.rs`。
- 新增 `application/knowledge.rs`。
- 新增 `interfaces/agent_cli/commands/knowledge.rs`。
- 支持 `extract/promote/export/list/validate`。

验收：

- `knowledge extract --topic` 创建 topic-level candidate 文件。
- `knowledge promote --to shared` 创建或更新 shared knowledge-system。
- `knowledge export --to knowledge-base` 创建候选知识库条目，但不自动标记 verified，除非 evidence 完整。

### Phase 6: TUI read-only surface

实现：

- TUI 显示 active review 和 pending knowledge promotions。
- 不做交互式编辑。

验收：

- `daedalus-tui` 能看到 project/topic/review/knowledge 四个方向的当前状态。

### Phase 7: Current Codex project dogfood

用当前 Codex learning project 验证：

```bash
daedalus review start --topic tools-permissions --mode rebuild --goal "重建安全本地命令执行 loop"
daedalus review session start <review-id>
daedalus knowledge extract --topic tools-permissions
```

验收：

- 能对 `tools-permissions` 启动复习。
- 能生成 mastery-map。
- 能从 Slice 0-6 的学习证据中萃取 approval/sandbox/retry 知识候选。
- 不影响当前 topic 的 `08-demo-coder` 状态。

## Test Plan

### CLI tests

新增 `cli_workflow.rs` 覆盖：

- `review_start_for_topic_creates_review_workspace`
- `review_start_for_project_creates_project_review`
- `review_does_not_reopen_completed_topic`
- `review_session_complete_records_session`
- `review_validate_rejects_missing_target`
- `knowledge_extract_creates_topic_candidates`
- `knowledge_promote_requires_existing_evidence`
- `knowledge_export_creates_knowledge_base_candidate`

### Validation tests

- project validate 默认不要求 reviews 存在。
- `validate --reviews` 校验 review state 和 session files。
- `validate --knowledge` 校验 promotion evidence。

### Prompt audit tests

扩展 `system/bin/audit-daedalus-agent-instructions`：

- 不允许 prompt 说“复习时重新打开 topic”。
- 不允许 prompt 把 review 称为第 11 阶段。
- 不允许 knowledge export 归档未验证假设。
- 不允许 Agent 在 review guidance 中先给答案再提问。

### Manual dogfood tests

对当前 Codex project：

- 启动 topic review。
- 完成一次 rebuild session。
- 更新 mastery-map。
- 生成 knowledge extraction candidates。
- 确认 `daedalus validate --all-topics` 不受 review 影响。

## Existing Workspace Update Strategy

review system 是新增能力，不需要移动旧 review 数据。

需要做的迁移：

- 给已有 project 增加 `.daedalus/reviews/README.md` 可选入口。
- 给已有 topic 增加 `.daedalus/reviews/README.md` 可选入口。
- 给已有 project `shared/` 增加 `knowledge-system/` 可选入口。

这些属于一次性手动整理：只新增缺失目录和 README，不移动原学习产物。

## Product Boundaries

第一版不做：

- 自动间隔提醒或后台通知。可以先记录 `next_session_at`，不做 scheduler。
- LLM 自动评分。Agent 可以校准，但必须保留用户回答和证据。
- 大规模 knowledge-base taxonomy 重构。
- 图数据库或复杂可视化。
- 多人协作复习。

第一版必须做到：

- 复习计划可恢复。
- 复习不污染 learning lifecycle。
- 掌握度变化有证据。
- 知识萃取有 promotion gate。
- completed topic/project 可以再次被激活为 review target。

## Open Questions

1. `review start` 是否允许 active topic？

   建议：第一版允许，但默认只作为 checkpoint review，不允许改变 stage state。

2. `mastery level` 是否需要分数？

   建议：不要用数字分数。使用 `unknown / weak / medium / strong / transferable`，并要求 evidence。

3. knowledge extraction 是否应该自动写入 knowledge-base？

   建议：不自动。学习过程中滚动维护 candidate，再由用户确认 promotion。

4. review 是否应该支持 scheduler？

   建议：第一版只记录 `next_session_at`。自动提醒可等 automation 能力稳定后再接。

5. review system 是否只服务 repo-learning？

   建议：领域模型保持通用，第一版只接 repo-learning。未来可以扩展到 book/course/paper learning。

## Success Criteria

该方案实现后，daedalus 应能回答这些用户请求：

```text
复习一下 tools-permissions 这个 topic。
不要总结，先考我几个问题。
我想重新从零设计一遍 Codex 权限系统。
把这个 topic 的知识体系萃取出来。
哪些知识已经能晋升到 shared？
哪些知识可以进入 knowledge-base？
我现在对这个 repo 的掌握薄弱点在哪里？
```

并且所有回答都能落回文件系统：

```text
review-plan.md
mastery-map.md
review sessions
knowledge-system maps
promotion log
knowledge-base candidate
```

最终目标：daedalus 不只是“带你学完一次”，而是能管理一个学习对象的长期掌握、复习、知识晋升和跨专题复用。
