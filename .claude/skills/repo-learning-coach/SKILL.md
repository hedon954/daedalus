---
name: repo-learning-coach
description: Guides GitHub repository learning from goal clarification to repo selection, question roadmap, local running, architecture analysis, code reading, demo planning, business transfer, and archival. Use when the user starts or continues a repo learning task.
---

# Repo Learning Coach

Use this skill when the user wants to learn a GitHub repo deeply.

## Workflow

1. Load `system/prompts/common/clarify-goal.md` and `system/prompts/repo/phase1-exploration/01-goal-aligner.md`.
2. Confirm the learning goal, real-world problem, current level, output artifact, and acceptance criteria.
3. If no repo is selected, use `system/prompts/repo/phase1-exploration/02-repo-scout.md`.
4. After a repo is selected, use `03-socratic-coach.md` to build the question roadmap.
5. Move through run/debug, architecture analysis, code reading, mini demo, business transfer, and archival in order.

## Rules

- Keep WIP = 1 in `workspaces/02-learning`.
- Prefer filesystem state over chat memory.
- Ask at most 3 high-value questions at a time.
- Tie every reading step to a future output artifact.
- Do not mark the task complete until demo/business transfer/knowledge archival are addressed.
