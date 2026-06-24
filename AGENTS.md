# AGENTS.md

daedalus is a filesystem-first deep learning coach. It guides a user from a real learning or business problem to source selection, deep reading, mini-demo implementation, business transfer, and verified knowledge archival.

## Must Follow

- Follow `system/prompts/common/agent-operating-contract.md`; the bullets below are the high-priority local contract.
- Use Chinese for project-facing learning artifacts unless the user asks otherwise.
- Prefer filesystem artifacts over hidden chat memory. Long-running learning state must be recoverable from files.
- Keep WIP strict: `workspaces/projects` may contain many stable projects, but at most one topic can be active through `workspaces/.daedalus/current.toml`.
- Resolve active learning context from `workspaces/.daedalus/current.toml` or `workspaces/current-topic` first. Resolve pending closeout context from `pending_closeout_topic` or `workspaces/closeout-topic`. Do not look for `.daedalus/state.toml` at the repository root.
- Treat project/topic `.daedalus/state.toml` and `workspaces/.daedalus/current.toml` as lifecycle sources of truth. `current-project` / `current-topic` are active-learning entry symlinks and should only appear when an active topic exists; `closeout-topic` is the pending reflection entry.
- Treat `workspaces/discovery` as pre-topic exploration space: use it to preserve unclear motivations, need hypotheses, and selection questions before creating backlog candidates or active topics.
- Ground implementation progress in current code, tests, runtime output, and git diff before trusting learning maps.
- After review, validation, or commit changes completion status, risks, evidence, or next action, synchronize the relevant learning artifacts.
- Before repo-learning commits, run a learning-map sync check; after repo-learning commits, end with post-commit orientation.
- When a discussion reveals a reusable learning method, thinking tool, artifact pattern, or Agent failure mode, promote it to the right durable layer instead of burying it only in the current topic.
- For learning notes, classify the artifact before writing: README files are indexes/timelines; reusable concepts, mechanisms, comparisons, failure modes, and deep explanations get their own focused note and are linked from the README and relevant maps.
- Treat topic closeout as a push-driven state machine: when a gate is satisfied, synchronize maps and move to the next gate instead of waiting for the user to re-prompt.
- Mine knowledge-base candidates from closeout plus guides, notes, demo, and validation evidence; do not rely on closeout alone.
- Keep closeout focused on core goals, demo decisions, trade-offs, architecture, transfer boundaries, and important weak foundations; make knowledge extraction greedy.
- Knowledge-base entries are natural notes in an evolving knowledge tree, not fixed templates or type buckets.
- A knowledge-base entry must be deep enough to stand alone as a technical blog post or lesson; do not archive shallow summaries.
- Knowledge-web pages must be at least as deep as their Markdown sources: use light, readable UI; include mechanism details, diagrams, comparisons, interactions, and source links; never ship a shallow decorative summary.
- When summaries or knowledge archival fill technical details, verify them with external sources when possible; prefer official docs, papers, source repos, or established best-practice references.
- Treat every study material as a constrained design case, not an authority. Preserve first principles, trade-offs, critical lens, faithful imitation choices, and not-to-copy boundaries.
- When developing daedalus Rust code, prioritize feature correctness and code simplicity over minimizing refactor size or implementation time.
- Write daedalus implementation plans as target outcomes, not "first version" compromises, unless the user explicitly asks for phased delivery.
- Put detailed daedalus implementation plans in `docs/plan/`; use `.codex/plans/` only as Codex tracking cards with `status` and `todos` that link to the canonical `docs/plan/` plan.
- Git commit messages must follow `{topic}/{scope}: 中文描述`, for example `lora-feedback-loop/debugger-guide: 跑通 Causal LM 训练闭环` or `daedalus/cli: 修正 topic 校验规则`.
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
