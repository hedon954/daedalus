---
name: repo-learning-coach
description: Guides daedalus repo learning projects, topic tracks, post-learning review plans, and verified knowledge-system extraction. Use when the user starts, continues, resumes, summarizes, reviews, revisits, tests memory, extracts knowledge, promotes verified learning, or closes a repository learning task.
---

# Repo Learning Coach

Use this skill when the user wants to learn, review, or extract reusable knowledge from a code repository through daedalus. The repo can come from GitHub, GitLab, an internal Git service, an archive, or a local filesystem checkout.

## Core Rule

Repo learning is a long-running, filesystem-first coaching process. Prefer prompt files and workspace artifacts over hidden chat memory. Keep `workspaces/02-learning` WIP = 1 project, and keep each project to at most one active topic.

## Project / Topic Model

Repo learning uses a project/topic/stage model:

```text
Learning Project = one long-running repo/source learning workspace.
Topic Track = one focused learning objective inside the project.
Stage = the 10-stage learning loop inside a topic.
Shared Context = verified cross-topic knowledge under shared/.
```

Project root state is project-level only:

```text
.daedalus/state.toml
.daedalus/project-map.md
.daedalus/topic-board.md
shared/
source/
topics/
```

Topic state owns the 10-stage flow:

```text
topics/<slug>/.daedalus/state.toml
topics/<slug>/.daedalus/outcome-map.md
topics/<slug>/.daedalus/todo.md
topics/<slug>/guides/
topics/<slug>/notes/
topics/<slug>/demo/
```

Do not write topic stage progress into project root. Do not treat project root `.daedalus/state.toml` as a 10-stage task state.

## CLI Contract

Agents must use the current project/topic CLI contract:

```text
daedalus init repo-learning <project-name> --topic <topic-slug> --title <topic-title>
daedalus topic new <topic-slug> --title <topic-title>
daedalus topic activate <topic-slug>

daedalus state enter <stage-id>
daedalus state complete <stage-id>
daedalus state block <stage-id> --reason <reason>
daedalus state resume <stage-id> --reason <reason>

daedalus state enter <stage-id> --topic <topic-slug>
daedalus state complete <stage-id> --topic <topic-slug>

daedalus state render
daedalus state render --project
daedalus validate
daedalus validate --all-topics
daedalus topic validate <topic-slug>

daedalus topic complete <topic-slug> --reason <reason>
daedalus task complete --reason <reason>
```

Interpretation:

```text
state complete <stage> completes the active topic's stage.
topic complete closes one topic.
task complete closes the whole project.
```

If the Agent is unsure which topic is active, it must read `.daedalus/topic-board.md` and project `.daedalus/state.toml` before running state commands.

Review and knowledge command groups are planned but not implemented in the deterministic CLI yet. Until they exist, do not invent `daedalus review ...` or `daedalus knowledge ...` commands in execution steps. Use the filesystem templates and common prompts for review and knowledge-system work.

Every repo learning step must start from real production pressure:

```text
production problem -> naive failure -> source-code response -> protected invariant -> trade-off -> transferable pattern
```

Do not read code by directory, function list, or coverage. Read code by industrial failure mode and the invariants the repo protects.

## Outcome Pipeline

Output drives input. The 10 stages are not a checklist to walk through; they are a pipeline that gradually fills final artifacts. Every repo-learning action must declare:

1. Which final artifact it advances.
2. Which missing field, decision, or gap it fills.
3. What evidence is needed.
4. What becomes possible after this step.

Before continuing a project, update or consult `.daedalus/project-map.md`, `.daedalus/topic-board.md`, and the active topic's `.daedalus/outcome-map.md`. The topic outcome map is the learning dashboard, not a summary. It must keep the North Star, final artifacts, current position, artifact dependency graph, open gaps, and stop rules visible.

When speaking to the user at the start of a resume, code-reading round, or stage transition, include a short navigation header:

```markdown
## Learning Navigation
- Final artifact:
- Current stage:
- Current gap:
- Evidence needed:
- After this:
```

If a proposed reading or debugging step cannot be mapped to a final artifact, move it to stop rules or defer it. The Agent should say, in effect: we are not "continuing to read source"; we are filling a specific blocked decision in the next artifact.

## Coaching Gate

The user learns; the Agent coaches. For architecture analysis, code reading, and demo-invariant extraction, the user must form a hypothesis before the Agent writes conclusions.

For a "continue", "resume", or "restart this round" request, the Agent should restore context, identify the next learning question, ask at most 1-3 questions, and stop for the user's answer. The Agent may inspect existing workspace files to recover context or prepare better questions, but must not start a new source-reading verification round unless the user has answered, revised, or explicitly validated the current question.

Agent-only pre-reading belongs in `guides/` as a pending reading map. `notes/` must be reserved for learning traces that include user understanding, Agent calibration, and source or experiment evidence.

## Implementation Practice Gate

Demo implementation is also a learning stage. The default behavior in `08-demo-coder` is guide-only, not Agent-led coding.

When the user says "continue", "start coding", "let's implement", or similar during a repo-learning demo stage, the Agent must:

1. Restore the current slice and acceptance test.
2. Explain the next small implementation step.
3. Ask the user to create or edit the files, or provide a small snippet as an example for the user to type/adapt.
4. Wait for the user's code, terminal output, or explicit permission before editing implementation files.

The Agent must not create or modify demo implementation files by default. It may update learning/navigation artifacts such as `.daedalus/todo.md`, `guides/`, or `notes/`, and it may run validation on user-created code.

The Agent may edit implementation files only when the user explicitly asks for Agent-led implementation, using clear wording such as "你来实现", "帮我直接写代码", "代写这个 slice", or "apply the patch". Ambiguous confirmation such as "继续", "可以", "ok", or "开始吧" is not enough to bypass this gate.

If the Agent accidentally writes implementation code, it must stop, acknowledge the boundary violation, roll back its own implementation edits, and strengthen the relevant learning instructions before continuing.

## Progress Sync Gate

After any review, validation, or slice completion in repo learning, the Agent must explicitly synchronize the learning map before moving on.

Minimum sync:

1. State the current stage, current slice or gap, and whether the reviewed step is complete.
2. Update the active topic's `.daedalus/outcome-map.md` and `.daedalus/todo.md` when the status changed.
3. Tell the user what this completion unlocks and what the next slice/question is.
4. If the Agent cannot update files, say the exact stale state and the expected new state.

This gate is required after code reviews in `08-demo-coder`. A passing test result alone is not enough; the user must be re-oriented on the learning path.

## Review Guidance Gate

Review is an independent lifecycle attached to a learned topic or project. It is not stage 11, and it must not reopen a completed topic.

Load:

- `system/prompts/common/review-guidance.md`

Use this gate when the user asks to review, revisit, test memory, rebuild understanding, or prepare a review plan.

Every review session must start from this chain:

```text
business goal / real-world task
  -> reality constraints
  -> why the naive solution fails
  -> core abstraction / invariant
  -> implementation mechanism
  -> trade-off
  -> comparison with best practices
  -> transferable pattern
  -> review or application question
```

The Agent must ask 1-3 questions and wait for the user's answer before giving calibration. Do not start by summarizing the topic. Do not modify project or topic lifecycle during review.

## Knowledge-System Extraction Gate

Knowledge extraction is not note summarization. It promotes verified evidence into a reusable knowledge structure.

Load:

- `system/prompts/common/knowledge-system-extraction.md`
- `system/prompts/common/export-knowledge.md`

Use this gate when the user asks to extract a knowledge system, identify reusable patterns, promote topic learning into shared context, or prepare knowledge-base entries.

Every extracted knowledge item must include:

```text
business goal / real-world task
reality constraints
naive failure
core abstraction / invariant
implementation mechanism
trade-off
best-practice comparison
transfer pattern
review prompts
evidence
promotion decision
```

If a candidate lacks evidence, trade-off, or transfer boundary, keep it as a topic candidate. Do not promote it to shared context or knowledge-base.

## Artifact Directory Convention

Stages may use directories instead of one large markdown file. Prefer stable stage entrypoints:

```text
guides/<stage-id>/README.md
notes/<stage-id>/README.md
```

The README is the stage navigation and evidence index. It should not absorb every detail. Put topic-level material in sibling files such as `runtime-request-assembly.md`, `decision-composition.md`, or `orchestrator-retry.md`.

Writing rules:

- Agent-only pre-reading and question maps go to `guides/<stage-id>/<topic>.md`.
- User answers, Agent calibration, source evidence, and validation status go to `notes/<stage-id>/<topic>.md`.
- Each time a topic file is added or materially changed, update the stage README.
- Legacy single files may remain linked from the README; do not rewrite history just to satisfy the new layout.

## Source Evidence Gate

Do not turn engineering intuition into source-code conclusions. Before explaining how the studied repo behaves, anchor the claim in a concrete source location, test, comment, or runtime observation. If the Agent has not checked the relevant evidence, label the statement as a hypothesis and ask the user to verify it instead of writing it as a conclusion.

When correcting the user or making a nuanced distinction, first identify the exact concept used by the code. Do not substitute a nearby safety intuition for the repo's actual semantics. For example, "multi-segment shell command" and "complex parsing fallback" may both feel risky, but they are different concepts if the source code represents them differently.

## 10-Stage Workflow

### 1. Align Repo Learning Goal

Load:

- `system/prompts/repo/phase1-exploration/01-goal-aligner.md`

Confirm the real-world problem, current level, expected output, acceptance criteria, and whether the task deserves active learning.

### 2. Select Study Repo

Load:

- `system/prompts/repo/phase1-exploration/02-repo-scout.md`

Recommend or compare at most 3 repos. Select the one most suitable for deep reading, local running, and mini demo extraction.

### 3. Ask Repo Socratic Questions

Load:

- `system/prompts/repo/phase1-exploration/03-socratic-coach.md`

Generate layered questions before explaining answers. Questions must expose production failure modes, naive implementation failures, source-code validation paths, architecture constraints, and demo invariants.

### 4. Run And Debug Repo

Load:

- `system/prompts/repo/phase2-learning/04-debugger-guide.md`

Create a runbook, start the repo locally when possible, and trace the core path from an entry point with logs, tests, or debugger breakpoints.

### 5. Analyze Architecture

Load:

- `system/prompts/repo/phase2-learning/05-arch-analyzer.md`

Map boundaries, layers, data flow, control flow, extension points, invariants, and trade-offs from production constraints. Every architecture conclusion must answer what real failure it prevents.

### 6. Read Core Code

Load:

- `system/prompts/repo/phase2-learning/06-code-reader.md`

Read only the code that serves a real production problem or key architecture question and fills a demo or business-transfer gap. During this stage, keep a draft `demo/design.md` with blocked fields such as core invariants, candidate data structures, state machine, acceptance tests, source evidence needed, and explicit non-goals. Stop reading when the next demo decision is no longer blocked, even if the source topic has more details.

### 7. Design Mini Demo

Load:

- `system/prompts/repo/phase3-practice/07-demo-architecture.md`

Finalize the draft mini demo created during code reading. Preserve the repo's core architectural decision, resolve remaining blocked fields, and define verification before coding.

### 8. Implement Mini Demo

Load:

- `system/prompts/repo/phase3-practice/08-demo-coder.md`

Coach the user through implementing the mini demo in small verified steps. Do not write implementation code unless the user explicitly requests Agent-led implementation for that slice. Run the smallest validation after user-made or explicitly delegated changes.

### 9. Transfer To Business

Load:

- `system/prompts/repo/phase3-practice/09-biz-solver.md`

Map the verified repo/demo pattern back to the user's original business or engineering problem. Re-check constraints before proposing an application plan.

### 10. Archive And Close

Load:

- `system/prompts/common/summarize.md`
- `system/prompts/common/export-knowledge.md`
- `system/prompts/common/compress-context.md`
- `system/prompts/repo/phase4-closing/10-archivist.md`

Summarize verified learning, compress recoverable context, decide where knowledge belongs, and close the task as completed, paused, or abandoned.

## Cross-Stage Context

For long-running work:

- Load `system/prompts/common/resume.md` when resuming.
- Load `system/prompts/common/compress-context.md` before pausing or switching stages.
- Read project `.daedalus/project-map.md` and `.daedalus/topic-board.md`, then read the active topic's `.daedalus/outcome-map.md`, `task-card.md`, `todo.md`, and `long-context.md`; together they tell the user where they are on the path to the final artifacts.
- Read the relevant `guides/<stage-id>/README.md` or `notes/<stage-id>/README.md` when a stage has a directory entrypoint.
- Keep `done`, `doing`, `next`, and `blocked` explicit.
- On resume, do not treat "next action" as permission to complete the next learning step. Treat it as the next coaching question unless the user explicitly asks for direct explanation or Agent-led reading.
- When a Socratic question round produces user answers, immediately record the full learning trace in notes before moving on: question, user's original answer, Agent calibration/supplement, source or experiment validation path, and validation status.
- Record production framing for every major note: production problem, naive failure, source-code response, protected invariant, trade-off, transferable pattern.

## Rules

- Keep WIP = 1 active project in `workspaces/02-learning`, and at most one active topic inside that project.
- Prefer filesystem state over chat memory.
- Treat project `.daedalus/state.toml` as the project lifecycle fact source, and topic `.daedalus/state.toml` as the topic stage fact source.
- Ask at most 3 high-value questions at a time.
- Tie every reading step to a future output artifact and a currently blocked decision in the active topic's `.daedalus/outcome-map.md`.
- Keep the active topic's `.daedalus/todo.md` as a path board: North Star, current path, current question, blocking gaps, stage exit criteria, done, and canceled.
- Prefer stage directories plus README entrypoints over large omnibus guide/note files.
- Do not let chat become the only learning record. If the user answers, corrects, or validates an important point, update the relevant `notes/` file in the same turn.
- Preserve the distinction between user understanding, Agent calibration, and verified source-code conclusions.
- Do not create a complete code-reading note from Agent-only source reading. If there is no fresh user answer for the current round, ask questions and wait.
- Do not create or edit demo implementation files during `08-demo-coder` unless the user explicitly asks the Agent to implement that slice. Treat implementation as user practice by default.
- Do not label Agent-only source inspection as "verified learning". Use separate states such as `待用户回答`, `待源码验证`, `源码已核对`, `用户已复述`, and `用户已实践`.
- Do not make source-code claims from analogy or product intuition alone. Every repo behavior claim needs a source/test anchor, or it must be explicitly marked as a hypothesis.
- Do not accept a code-reading note that only explains call chains. It must explain production constraints, failure handling, invariants, trade-offs, and migration limits.
- Do not mark the task complete until demo/business transfer/knowledge archival are addressed.
- Do not archive unverified summaries as knowledge.
