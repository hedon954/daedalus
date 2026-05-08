---
name: mini-demo-builder
description: Designs and implements focused mini demos that reproduce a repository's core architectural decision. Use when turning repo learning into a runnable demo, validation script, or implementation exercise.
---

# Mini Demo Builder

Use this skill after the repo's core path and architecture have been understood.

## Design Steps

1. Load `system/prompts/repo/phase3-practice/07-demo-architecture.md`.
2. Identify the repo capability to reproduce.
3. Preserve the key invariant or architectural trade-off.
4. Explicitly list what the demo will not implement.
5. Define a runnable acceptance check before coding.

## Implementation Steps

1. Load `system/prompts/repo/phase3-practice/08-demo-coder.md`.
2. Build the smallest skeleton that can run.
3. Implement the core data structure and main path first.
4. Add only the most instructive boundary case.
5. Run the smallest verification command after each meaningful step.
6. Compare the demo with the original repo: same idea, simplifications, and costs.

## Output

Keep a short note with goal, modules, main path, validation command, and learning takeaway.
