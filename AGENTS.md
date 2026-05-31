# AGENTS.md

This repository is a filesystem-first deep learning coach named daedalus. The product goal is to guide a user from a real learning/business problem to repo selection, deep code reading, mini-demo implementation, business transfer, and knowledge-base archival.

## Operating Principles

- Use Chinese for project-facing learning artifacts unless the user asks otherwise.
- Treat output as the driver of input. Every reading step should support a concrete artifact: a question roadmap, runbook, architecture note, code reading note, mini demo, business solution, or knowledge-base entry.
- Keep WIP strict. `workspaces/02-learning` should contain at most one active learning task.
- Completed learning tasks must move to `workspaces/03-completed`; abandoned tasks must move to `workspaces/04-abandoned`. Use the CLI lifecycle commands instead of leaving closed tasks in `workspaces/02-learning`.
- Treat `.daedalus/state.toml` as the only source of truth for task lifecycle. The workspace directory bucket is a filesystem projection and must match `task.lifecycle` and `task.workspace_bucket`.
- Prefer filesystem artifacts over hidden chat memory. Long-running learning state must be recoverable from files.
- Do not turn summaries into chat logs. Preserve goals, decisions, open questions, todo state, and verified conclusions.
- After a review or validation changes completion status, risks, or the next step, update the relevant learning artifacts before ending the turn, even when source code is otherwise review-only.
- Emphasize first principles and trade-offs: reality needs X, constraints force Y, the repo chooses Z, and the choice has costs.
- When validating templates or generated workspace artifacts, judge them against daedalus's purpose: filesystem-first, recoverable, teachable, and reviewable learning loops. Do not blindly align one file to another if the result weakens that purpose.

## Project Structure

- `crates/`: deterministic Rust workspace for MCP servers, CLI tools, agent utilities, and other code-backed capabilities.
- `knowledge-base/`: durable knowledge distilled from verified learning output.
- `system/bin/`: scripts for local setup, build, and execution.
- `system/config/`: user preferences and future runtime configuration.
- `system/prompts/common/`: reusable prompts shared by repo, book, course, and paper learning.
- `system/prompts/repo/`: staged prompts for code repository learning.
- `system/templates/`: future templates for task cards, long context, todo state, and learning boards.
- `workspaces/`: backlog, active learning, completed, and abandoned tasks.

## Repo Learning Flow

1. Clarify the learning goal and real-world problem.
2. Introduce the goal from first principles and connect prior knowledge when available.
3. Recommend and narrow code repositories.
4. Generate a progressive question roadmap.
5. Run the repo locally and debug the core path from an entry point.
6. Analyze architecture, modules, patterns, algorithms, and trade-offs.
7. Read core code deeply and line by line where needed.
8. Design and implement a mini demo that preserves the repo's core architectural decision.
9. Apply the learned pattern to the user's original business problem.
10. Summarize and archive verified knowledge.

## Editing Guidelines

- Keep prompt files concise and composable.
- Avoid adding broad abstractions before a concrete learning workflow requires them.
- When adding deterministic logic, prefer Rust code under `crates/` and keep generated or runtime state out of source control.
- When adding or moving a Rust crate, update `crates/Cargo.toml`, `Makefile`, `.pre-commit-config.yaml`, and `.github/workflows/ci.yml` as needed so `make ci` continues to cover formatting, check, clippy, and tests for the whole Rust workspace.
- Tests should assert stable behavior, not incidental wording: for errors, prefer checking the error variant/category or presence unless the exact message is part of the public contract.
- Before marking a learning task completed, verify that it has at least: goal, core questions, run/debug notes or justified skip, architecture/code notes, demo or explicit reason for no demo, business transfer, and knowledge export.

## Knowledge Base Rules

- Archive only verified knowledge.
- Prefer reusable patterns over repo-specific trivia.
- Each knowledge entry should include reality constraints, core approach, trade-off, transferable pattern, and source.
- Revisit taxonomy as the knowledge base grows; do not over-design categories early.
