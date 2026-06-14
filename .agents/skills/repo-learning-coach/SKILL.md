---
name: repo-learning-coach
description: Guides the full 10-stage daedalus code repo learning loop: align goal, select repo, ask roadmap questions, run/debug, analyze architecture, read core code, design mini demo, implement mini demo, transfer to business, and archive knowledge. Use when the user starts, continues, resumes, summarizes, or closes a repository learning task.
---

# Repo Learning Coach

Use this skill when the user wants to learn a code repository deeply through daedalus. The repo can come from GitHub, GitLab, an internal Git service, an archive, or a local filesystem checkout.

## Core Rule

Repo learning is a long-running, filesystem-first coaching process. Prefer prompt files and workspace artifacts over hidden chat memory. Keep `workspaces/02-learning` WIP = 1.

Every repo learning step must start from real production pressure:

```text
production problem -> naive failure -> source-code response -> protected invariant -> trade-off -> transferable pattern
```

Do not read code by directory, function list, or coverage. Read code by industrial failure mode and the invariants the repo protects.

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

Read only the code that serves a real production problem or key architecture question. Extract naive failure modes, source-code responses, protected invariants, design costs, and reusable implementation patterns.

### 7. Design Mini Demo

Load:

- `system/prompts/repo/phase3-practice/07-demo-architecture.md`

Design a focused mini demo that preserves the repo's core architectural decision. Define verification before coding.

### 8. Implement Mini Demo

Load:

- `system/prompts/repo/phase3-practice/08-demo-coder.md`

Implement the mini demo in small verified steps. Run the smallest validation after meaningful changes.

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
- Keep `done`, `doing`, `next`, and `blocked` explicit.
- When a Socratic question round produces user answers, immediately record the full learning trace in notes before moving on: question, user's original answer, Agent calibration/supplement, source or experiment validation path, and validation status.
- Record production framing for every major note: production problem, naive failure, source-code response, protected invariant, trade-off, transferable pattern.

## Rules

- Keep WIP = 1 in `workspaces/02-learning`.
- Prefer filesystem state over chat memory.
- Treat `.daedalus/state.toml` as the only lifecycle fact source; completed and abandoned directories are projections of `task.lifecycle` and `task.workspace_bucket`.
- Ask at most 3 high-value questions at a time.
- Tie every reading step to a future output artifact.
- Do not let chat become the only learning record. If the user answers, corrects, or validates an important point, update the relevant `notes/` file in the same turn.
- Preserve the distinction between user understanding, Agent calibration, and verified source-code conclusions.
- Do not accept a code-reading note that only explains call chains. It must explain production constraints, failure handling, invariants, trade-offs, and migration limits.
- Do not mark the task complete until demo/business transfer/knowledge archival are addressed.
- Do not archive unverified summaries as knowledge.
