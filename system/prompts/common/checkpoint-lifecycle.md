# Checkpoint Lifecycle

## Purpose

Checkpoint lifecycle keeps code facts, learning maps, commits, and next actions aligned. It prevents long-running learning from depending on chat memory.

Use this protocol after review, validation, user-practice completion, design acceptance, implementation closeout, or any repo-learning commit.

## Long-Running Turn Handoff

When a repo-learning action is likely to take noticeable time, do not make the learner wait with no path forward.

Use this before long code review, broad file scans, test/debug loops, large doc rewrites, validation/commit sequences, or any task expected to take more than roughly 30-60 seconds.

If unsure whether the work is long, prefer a compact handoff over silence. First send a short handoff card, then continue the Agent work:

```markdown
## While I Work
- What I am doing:
- Why it matters:
- You can read / inspect:
- Question to think about:
- Expected checkpoint:
```

Rules:

- The handoff can be as short as 3 lines: what I am doing, what the learner can inspect, and one question to think about.
- Keep it actionable, not motivational. Give the learner one small reading target, one observation task, or one question.
- Tie the handoff to the current final artifact or blocked decision.
- Do not use handoff for quick commands or tiny edits; unnecessary handoff adds noise.
- If the work changes direction, update the learner with the new focus instead of letting the old handoff become misleading.
- The final response must reconcile what the Agent found with the handoff question, so the learner's parallel thinking can reconnect to the result.

## Evidence First

Before changing stage, slice, or completion status, inspect the strongest available evidence:

1. Current source code, tests, runtime output, and git diff.
2. User-verified terminal output or code they wrote.
3. Active topic artifacts: `.daedalus/outcome-map.md`, `.daedalus/todo.md`, stage README, notes, guides.
4. Prior chat or assumptions.

If code evidence and learning maps disagree, trust code/tests, mark the map stale, and synchronize it.

## Cursor Sync

Use cursor sync when the user has reached a new implementation or reading frontier, but no verified checkpoint exists yet.

- Update only the active topic `.daedalus/todo.md` `Current Cursor` block.
- Update `.daedalus/outcome-map.md` only when the broader position is misleading.
- Mark WIP facts as `WIP / unverified`.
- Record at most: `Code frontier`, `Already wired`, `Current open decision`, `Do not suggest`.
- Do not complete stages, create notes, or commit solely because of cursor sync.

## Review / Validation Sync

If review or validation changes completion status, risks, blockers, test gaps, or next action:

- Update `.daedalus/todo.md`.
- Update `.daedalus/outcome-map.md`.
- Update relevant `notes/` or `guides/` indexes when a conclusion or action map changed.
- Tell the user what is complete, what remains, and what this unlocks.

Review-only is not progress-sync-only. Source code may stay untouched, but learning maps must not stay stale.

## Pre-Commit Sync

Before committing repo-learning implementation, demo, test, or checkpoint changes, check whether code, tests, review, or validation changed:

- current stage, slice, code frontier, or open gap;
- completion status or exit criteria;
- risks, blockers, failed validation, or deferred non-goals;
- verified evidence, tests, or runtime observations;
- next action, next guide, or next coaching question.

If yes, synchronize active topic artifacts before staging:

- `.daedalus/todo.md` for current cursor, next action, and blockers.
- `.daedalus/outcome-map.md` for broader position, artifacts, gaps, and stop rules.
- `.daedalus/long-context.md` only when durable recovery context changed.
- Relevant `notes/` / `guides/` when conclusions or next-step maps changed.

## Commit Boundary

Split commits by meaning:

```text
current checkpoint = proof that the completed work is stable
next-step guide = map for the next session
```

Do not mix completed implementation and future planning in one commit unless they are inseparable.

Use `type(scope): 中文描述` or `type: 中文描述`. Do not put unverified hypotheses in commit messages.

## Post-Commit Orientation

After every repo-learning commit, final response must re-orient the learner:

```markdown
## Post-Commit Orientation
- Commit:
- Current:
- Completed:
- Updated artifacts:
- Validation:
- Next:
```

Keep it short. A commit is a stable learning checkpoint, not the end of the learning path.

## Critical Checkpoint

After a closed loop, stage transition, key design decision, or slice closeout, preserve:

```text
source constraint
faithful imitation
simplified / improved / discarded
transfer risk
```
