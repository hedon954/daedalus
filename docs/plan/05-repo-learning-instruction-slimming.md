# Repo Learning 指令瘦身与模块化重构方案

> Date: 2026-06-06
> Status: done (v1 implemented)

## Summary

当前 daedalus 的 repo-learning 指令已经接近“规则过载”边界。问题不是单个规则错误，而是规则逐步补丁化后出现了三个风险：

- `AGENTS.md` 与 `CLAUDE.md` 基本重复，且已经出现轻微差异，后续容易漂移。
- `.claude/skills/repo-learning-coach/SKILL.md` 已超过 500 行，承担了 skill router、CLI 手册、阶段总览、gate 规则、review、knowledge extraction 等多种职责。
- 多个 sync / commit / checkpoint gate 语义重叠，虽然更严格，但更难被 Agent 稳定执行。

本计划目标是把入口规则变短、把细节下沉、把重复规则合并，让 Agent 更容易遵循，而不是继续堆叠补丁。

核心原则：

```text
入口文件只负责高频硬约束
SKILL.md 只负责路由和关键门禁
细节规则下沉到可按场景加载的 prompt
重复 gate 合并成同一条 checkpoint lifecycle
```

## First Principles

### 1. 指令越长，不等于遵循越好

Agent 的失败经常不是“不知道规则存在”，而是规则太分散，当前任务触发时没有被正确激活。

因此高频、跨场景、不可违反的规则应该靠近入口；低频、阶段性、操作型规则应该下沉到 prompt，在触发场景时再加载。

### 2. 同一事实不能多处维护

`AGENTS.md` 和 `CLAUDE.md` 现在大段重复。重复本身不是问题，重复但不完全一致才是问题。

更好的结构是：

```text
shared operating contract
  -> AGENTS.md thin adapter
  -> CLAUDE.md thin adapter
```

这样不同 Agent 入口可以存在，但核心规则只有一个事实源。

### 3. Gate 应该表达生命周期，而不是补丁历史

目前规则里有：

- Progress Sync Gate
- Learning Map Commit Gate
- Post-Commit Orientation Gate
- Current Cursor Sync Gate
- Micro Checkpoint Gate

这些都在解决同一件事：

```text
学习现场不能漂移，代码事实、学习地图、提交和下一步必须保持一致
```

应该重组成一个 `Checkpoint Lifecycle`，内部区分 cursor、review、commit、post-commit，而不是把每次事故都新增一个 gate。

## Current Diagnosis

### AGENTS.md / CLAUDE.md

当前状态：

- `AGENTS.md`: 约 60 行，长度健康。
- `CLAUDE.md`: 约 60 行，长度健康。
- 二者高度重复，但 `AGENTS.md` 比 `CLAUDE.md` 多了若干规则，例如实现进度必须以代码和测试为准。

判断：

- 不需要大删，但需要消除漂移风险。
- 根文件应该保留最硬的跨 Agent 规则，不应承载完整 repo-learning protocol。

建议保留：

- 项目目标。
- 文件系统优先、状态可恢复。
- WIP = 1。
- `.daedalus/state.toml` 是生命周期事实源。
- 代码/测试优先于 markdown 地图。
- repo-learning commit 前 sync，commit 后 orientation。
- first principles / trade-off / critical lens。
- commit message / test assertion 规范。

建议下沉：

- 完整 10 阶段说明。
- 详细 artifact 要求。
- review / knowledge extraction 细节。
- CLI 命令清单。

### repo-learning-coach/SKILL.md

当前状态：

- 超过 500 行。
- 同时承担：
  - skill 使用说明。
  - project/topic 模型。
  - CLI contract。
  - outcome pipeline。
  - critical lens。
  - evidence gate。
  - implementation gate。
  - 多个 checkpoint gate。
  - artifact directory convention。
  - 10-stage workflow。
  - review / knowledge extraction。

判断：

- 作为 skill 入口偏长。
- 读完成本高，触发场景越复杂，越容易漏掉关键门禁。
- 应该变成“路由表 + 不可违反规则 + prompt loading strategy”。

## Target Shape

### 1. 新增共享根规则

新增：

```text
system/prompts/common/agent-operating-contract.md
```

内容包括：

- daedalus 产品目标。
- filesystem-first。
- WIP / lifecycle / state source。
- evidence grounding。
- checkpoint orientation。
- critical learning。
- commit message。
- stable tests。

`AGENTS.md` / `CLAUDE.md` 变成薄入口：

```markdown
# AGENTS.md

This repository is daedalus...

## Must Follow

- Read and follow `system/prompts/common/agent-operating-contract.md`.
- Keep project-facing learning artifacts in Chinese.
- ...
```

注意：部分 Agent 不一定会自动读取引用文件，所以根文件仍保留 8-10 条最硬规则，不能只写“见某文件”。

### 2. 拆分 CLI Contract

新增：

```text
system/prompts/repo/repo-learning-cli-contract.md
```

从 `SKILL.md` 下沉：

- `daedalus init repo-learning`
- `daedalus topic new / activate`
- `daedalus state enter / complete / block / resume`
- `daedalus review ...`
- `daedalus knowledge ...`
- project state 与 topic state 的解释。

`SKILL.md` 只保留：

```text
When using CLI lifecycle commands, load repo-learning-cli-contract.md first.
```

### 3. 合并 Checkpoint Gates

新增：

```text
system/prompts/common/checkpoint-lifecycle.md
```

合并以下内容：

- Current Cursor Sync
- Progress Sync
- Learning Map Commit
- Post-Commit Orientation
- Micro Checkpoint

目标结构：

```markdown
# Checkpoint Lifecycle

## Cursor Sync
WIP 光标变化，只更新最小定位，不提交。

## Review / Validation Sync
review 或验证改变完成度、风险、下一步，则更新学习地图。

## Pre-Commit Sync
提交前检查代码事实是否改变地图。

## Commit Boundary
当前事实和下一步规划必要时拆 commit。

## Post-Commit Orientation
提交后必须输出 Current / Completed / Validation / Next。
```

`micro-checkpoint.md` 可以保留为薄包装，或被 `checkpoint-lifecycle.md` 替代。

### 4. 精简 repo-learning-coach/SKILL.md

目标长度：150-220 行。

建议结构：

```markdown
# Repo Learning Coach

## When To Use
...

## Non-Negotiables
- filesystem-first
- user learns, Agent coaches
- output drives input
- evidence before status
- critical lens always
- checkpoint lifecycle

## Project / Topic Model
简版，只说明 project/topic/stage/shared context。

## Prompt Loading Strategy
- resume -> common/resume.md
- CLI -> repo-learning-cli-contract.md
- checkpoint -> common/checkpoint-lifecycle.md
- review -> common/review-guidance.md
- knowledge -> common/knowledge-system-extraction.md
- stage -> 对应 phase prompt

## Stage Index
10 个阶段只列 prompt path 和一句话目标。

## Implementation Practice Boundary
08 阶段用户动手，Agent 默认不代写。

## Artifact Rules
简版：guides / notes / outcome-map / todo 的职责。
```

删除或下沉：

- 完整 CLI 命令清单。
- 详细 checkpoint gate 文案。
- review / knowledge extraction 长模板。
- 10-stage 的长说明。
- 和 `AGENTS.md` 重复的根规则。

### 5. 保留阶段 prompt 的专业细节

现有阶段 prompt 不需要大改，只需要把重复的 checkpoint 文案替换成引用：

```markdown
@system/prompts/common/checkpoint-lifecycle.md
```

重点影响文件：

- `system/prompts/common/micro-checkpoint.md`
- `system/prompts/repo/phase3-practice/08-demo-coder.md`
- `system/prompts/common/resume.md`
- `system/prompts/common/review-guidance.md`
- `system/prompts/common/knowledge-system-extraction.md`

## Proposed File Changes

### Add

- `system/prompts/common/agent-operating-contract.md`
- `system/prompts/common/checkpoint-lifecycle.md`
- `system/prompts/repo/repo-learning-cli-contract.md`

### Rewrite / Slim

- `AGENTS.md`
- `CLAUDE.md`
- `.claude/skills/repo-learning-coach/SKILL.md`

### Update References

- `system/prompts/common/micro-checkpoint.md`
- `system/prompts/repo/phase3-practice/08-demo-coder.md`
- any prompt that repeats checkpoint lifecycle details.

## Rollout Plan

### Phase 1: Extract Shared Contracts

1. Create `agent-operating-contract.md`.
2. Move shared root rules from `AGENTS.md` / `CLAUDE.md` into it.
3. Keep root files as thin, high-priority adapters.
4. Verify no important root-only rule is lost.

Exit criteria:

- `AGENTS.md` and `CLAUDE.md` are each under 45 lines.
- Both contain the same hard rules or point to the same shared contract.

### Phase 2: Extract CLI Contract

1. Create `repo-learning-cli-contract.md`.
2. Move full daedalus CLI command list out of `SKILL.md`.
3. Keep only a short pointer in `SKILL.md`.

Exit criteria:

- `SKILL.md` no longer contains long CLI command blocks.
- CLI usage remains discoverable from skill instructions.

### Phase 3: Merge Checkpoint Lifecycle

1. Create `checkpoint-lifecycle.md`.
2. Merge cursor sync、progress sync、pre-commit sync、post-commit orientation、micro checkpoint。
3. Replace duplicate gate text in `SKILL.md` and `08-demo-coder.md` with references and short hard rules.

Exit criteria:

- No more separate long sections for five sync/commit gates in `SKILL.md`.
- Post-commit orientation remains explicit and hard to miss.

### Phase 4: Slim SKILL.md

1. Rewrite `repo-learning-coach/SKILL.md` as a router and core contract.
2. Preserve trigger description and non-negotiables.
3. Keep stage index as prompt paths plus one-line purpose.
4. Remove duplicated long content now owned by prompt files.

Exit criteria:

- `SKILL.md` under 220 lines.
- All detailed behavior remains reachable through prompt references.
- No stage loses its current learning constraints.

### Phase 5: Validate With Current Codex Topic

Use current `tools-permissions` topic as acceptance fixture:

1. Ask “回顾当前学习进度”.
2. Expected: Agent reads code/tests/maps, reports `08-demo-coder / Slice 9`.
3. Ask “提交吧” after a small docs change.
4. Expected: Agent commits and outputs `Post-Commit Orientation`.
5. Ask “继续学习”.
6. Expected: Agent asks next Slice 9 coaching question, not direct implementation.

## Test Plan

- `git diff --check`.
- Search for duplicate long gate sections:

```text
rg -n "Learning Map Commit Gate|Post-Commit Orientation Gate|Current Cursor Sync Gate|Micro Checkpoint Gate" .claude system/prompts
```

Expected after refactor:

- Detailed lifecycle exists in `checkpoint-lifecycle.md`.
- Other files only reference it or keep one-line hard rules.

- Search root drift:

```text
diff AGENTS.md CLAUDE.md
```

Expected:

- Differences are intentional adapter wording only.

- Manual prompt simulation:
  - “提交吧” -> includes post-commit orientation.
  - “继续学习” -> asks next coaching question.
  - “当前进度” -> grounds in code/tests before maps.

## Risks And Trade-Offs

### Risk 1: 根文件太薄，Agent 不读引用文件

Mitigation:

- 根文件仍保留 8-10 条最硬规则。
- 不只写“see contract”。

### Risk 2: 下沉后规则不再被触发

Mitigation:

- `SKILL.md` 的 prompt loading strategy 要明确哪些场景必须 load 哪些 prompt。
- 高频 gate 保留短版在 `SKILL.md`。

### Risk 3: 重构期间丢规则

Mitigation:

- 先 extract，后 slim。
- 每个 phase 做一次 checklist。
- 用当前 Codex topic 做行为验收。

### Risk 4: 过度模块化导致查找成本上升

Mitigation:

- 文件命名必须直观。
- `SKILL.md` 提供一张 prompt loading index。
- 不把一个 gate 拆成多个碎文件。

## Success Criteria

- 根入口更短，但硬规则不丢。
- `SKILL.md` 从手册变回 coach router。
- checkpoint 生命周期从多个补丁 gate 变成一个统一协议。
- Agent 在 commit 后能稳定主动输出当前进度和下一步。
- 用户不需要反复提醒“更新进度、提交、规划下一步”。
