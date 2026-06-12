# AGENTS.md

daedalus is a filesystem-first deep learning coach. It guides a user from a real learning or business problem to source selection, deep reading, mini-demo implementation, business transfer, and verified knowledge archival.

## Must Follow

- Follow `system/prompts/common/agent-operating-contract.md`; the bullets below are the high-priority local contract.
- Use Chinese for project-facing learning artifacts unless the user asks otherwise.
- Prefer filesystem artifacts over hidden chat memory. Long-running learning state must be recoverable from files.
- Keep WIP strict: `workspaces/projects` may contain many stable projects, but at most one project and one topic can be active through `workspaces/.daedalus/current.toml`.
- Resolve active learning context from `workspaces/.daedalus/current.toml` or `workspaces/current-topic` first. Do not look for `.daedalus/state.toml` at the repository root.
- Treat project/topic `.daedalus/state.toml` and `workspaces/.daedalus/current.toml` as lifecycle sources of truth. `current-project` and `current-topic` are human-facing symlink projections.
- Ground implementation progress in current code, tests, runtime output, and git diff before trusting learning maps.
- After review, validation, or commit changes completion status, risks, evidence, or next action, synchronize the relevant learning artifacts.
- Before repo-learning commits, run a learning-map sync check; after repo-learning commits, end with post-commit orientation.
- Treat every study material as a constrained design case, not an authority. Preserve first principles, trade-offs, critical lens, faithful imitation choices, and not-to-copy boundaries.
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
