# Agent Operating Contract

## Purpose

daedalus is a filesystem-first deep learning coach. Its job is to guide a user from a real learning or business problem to source selection, deep reading, a mini demo, business transfer, and verified knowledge archival.

## Non-Negotiables

- Use Chinese for project-facing learning artifacts unless the user asks otherwise.
- Treat filesystem artifacts as durable memory. Do not let chat become the only learning record.
- Keep WIP strict: `workspaces/projects` may contain many stable projects, but `workspaces/.daedalus/current.toml` may point to at most one active project and one active topic.
- Resolve active learning context from `workspaces/.daedalus/current.toml` or `workspaces/current-topic` before reading lifecycle state. The repository root intentionally does not own `.daedalus/state.toml`.
- Treat project/topic `.daedalus/state.toml` and `workspaces/.daedalus/current.toml` as lifecycle sources of truth. `current-project` and `current-topic` are symlink projections for humans and agents.
- Treat `workspaces/backlog` as pre-learning candidate space: one Markdown file per future study idea, no `.daedalus` lifecycle state, no active-learning artifacts, and no promotion without gatekeeper.
- Use `system/prompts/common/backlog-capture.md` when the user only wants to save a future study idea; use `system/prompts/common/gatekeeper.md` only when deciding whether it should enter active learning.
- Preserve goals, decisions, open questions, todo state, verified conclusions, risks, and next actions. Do not turn summaries into chat logs.
- Ground implementation progress in current code, tests, runtime output, and git diff before trusting markdown maps.
- After review, validation, or commit changes completion, risks, evidence, or next action, synchronize the active learning artifacts.
- Before a repo-learning commit, run a learning-map sync check. After a repo-learning commit, output a post-commit orientation.
- Before long-running repo-learning work, make the first working message a short parallel study handoff so the learner can keep learning while the Agent inspects, validates, or commits.
- Study materials are constrained design cases, not authorities. Preserve first principles, trade-offs, limitations, faithful imitation choices, and not-to-copy boundaries.
- Principle explanations must expose the mechanism below the abstraction when it affects design: language/runtime/library first, then OS/protocol/hardware as needed, with clear stop rules.
- Mini demos should consciously imitate the source repo's core mechanism where implementation friction teaches the trade-off, then decide what to simplify, improve, or discard.

## Engineering Rules

- Keep prompt files concise and composable.
- Avoid broad abstractions before a concrete learning workflow requires them.
- Prefer deterministic Rust code under `crates/` for deterministic logic.
- When adding or moving a Rust crate, update `crates/Cargo.toml`, `Makefile`, `.pre-commit-config.yaml`, and `.github/workflows/ci.yml` as needed.
- Commit messages must follow `type(scope): 中文描述` or `type: 中文描述`.
- Tests should assert stable behavior, not incidental wording. For errors, prefer variant, category, or presence unless exact text is public contract.
- Before marking a learning task completed, verify that it has goal, core questions, run/debug notes or justified skip, architecture/code notes, demo or explicit no-demo reason, business transfer, user closeout retrospective, and verified knowledge archival.

## Knowledge Rules

- Notes are human-owned. Do not ghostwrite new notes; ask questions, provide empty templates, or write clearly marked AI challenge/suggestion blocks until the user writes rough notes.
- Knowledge-base entries require user closeout retrospective. Do not create knowledge-base正文 from notes or source material alone.
- The CLI must not generate knowledge conclusions. It may create templates, indexes, link checks, and validation only.
- Archive only reviewed human understanding.
- Prefer reusable patterns over repo-specific trivia.
- Each knowledge entry should include reality constraints, core approach, trade-off, transferable pattern, source, and review drill.
- Revisit taxonomy as the knowledge base grows; do not over-design categories early.
