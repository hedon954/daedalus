# CLAUDE.md

daedalus is a filesystem-first deep learning coach. It guides a user from a real learning or business problem to source selection, deep reading, mini-demo implementation, business transfer, and verified knowledge archival.

## Must Follow

- Follow `system/prompts/common/agent-operating-contract.md`; the bullets below are the high-priority local contract.
- Use Chinese for project-facing learning artifacts unless the user asks otherwise.
- Prefer filesystem artifacts over hidden chat memory. Long-running learning state must be recoverable from files.
- Keep WIP strict: `workspaces/projects` may contain many stable projects, but at most one topic can be active through `workspaces/.daedalus/current.toml`.
- Resolve active learning context from `workspaces/.daedalus/current.toml` or `workspaces/current-topic` first. Resolve pending closeout context from `pending_closeout_topic` or `workspaces/closeout-topic`. Do not look for `.daedalus/state.toml` at the repository root.
- Treat project/topic `.daedalus/state.toml` and `workspaces/.daedalus/current.toml` as lifecycle sources of truth. `current-project` / `current-topic` are active-learning entry symlinks and should only appear when an active topic exists; `closeout-topic` is the pending reflection entry.
- Ground implementation progress in current code, tests, runtime output, and git diff before trusting learning maps.
- After review, validation, or commit changes completion status, risks, evidence, or next action, synchronize the relevant learning artifacts.
- Before repo-learning commits, run a learning-map sync check; after repo-learning commits, end with post-commit orientation.
- When a discussion reveals a reusable learning method, thinking tool, artifact pattern, or Agent failure mode, promote it to the right durable layer instead of burying it only in the current topic.
- Treat topic closeout as a push-driven state machine: when a gate is satisfied, synchronize maps and move to the next gate instead of waiting for the user to re-prompt.
- Mine knowledge-base candidates from closeout plus guides, notes, demo, and validation evidence; do not rely on closeout alone.
- Keep closeout focused on core goals, demo decisions, trade-offs, architecture, transfer boundaries, and important weak foundations; make knowledge extraction greedy.
- Treat every study material as a constrained design case, not an authority. Preserve first principles, trade-offs, critical lens, faithful imitation choices, and not-to-copy boundaries.
- When developing daedalus Rust code, prioritize feature correctness and code simplicity over minimizing refactor size or implementation time.
- Write daedalus implementation plans as target outcomes, not "first version" compromises, unless the user explicitly asks for phased delivery.
- Git commit messages must follow `type(scope): 中文描述` or `type: 中文描述`.
- Tests should assert stable behavior, not incidental wording.

## Project Map

- `crates/`: deterministic Rust workspace for CLI, TUI, MCP, and code-backed capabilities.
- `system/prompts/`: composable prompt protocols.
- `system/templates/`: generated workspace templates.
- `knowledge-base/`: verified reusable knowledge.
- `workspaces/`: stable projects, backlog, current symlinks, and workspace-level indexes.

## Repo Learning

- Use `.claude/skills/repo-learning-coach/SKILL.md` for repo learning, review, human-owned notes, and verified knowledge archival.
- Before marking a learning task completed, verify goal, core questions, run/debug evidence or justified skip, architecture/code notes, demo or explicit no-demo reason, business transfer, user closeout retrospective, and verified knowledge archival.
