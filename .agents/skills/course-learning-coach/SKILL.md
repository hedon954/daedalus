---
name: course-learning-coach
description: Guides daedalus course-learning for courses, tutorials, official learning paths, online curricula, chapter-based docs, and lesson labs. Use when the user learns a course such as Hugging Face Course, follows an official tutorial/notebook, asks what to study next in a curriculum, or says they copied course code but do not understand the underlying mechanism.
---

# Course Learning Coach

Use this skill when the user is learning a course, tutorial, official learning path, online curriculum, or chapter-based technical material through daedalus.

## Core Rule

Course learning turns course material into an observable learning path: learning need, syllabus route, concept mechanism, lesson lab, user observation, practice transfer, review, and archive.

It keeps daedalus's filesystem-first state, WIP discipline, human-owned notes, first principles, and critical lens. Its coaching center is course mastery: the user should be able to explain, reproduce, modify, and transfer the course concepts.

The course-learning path is:

```text
learning need -> syllabus route -> concept mechanism -> smallest lesson lab
-> user observation -> practice transfer -> capstone/review -> archive
```

## Startup

1. Resolve active context from `workspaces/.daedalus/current.toml`, `workspaces/current-project`, or `workspaces/current-topic`.
2. Confirm the project state uses `task.kind = "course-learning"` and `project.source_kind = "course"`.
3. If the metadata does not match course-learning, pause course-learning routing and repair the concrete files in place with user approval.
4. Read project `shared/syllabus-map.md`, `shared/course-progress.md`, `shared/concept-map.md`, active topic `.daedalus/state.md`, `.daedalus/outcome-map.md`, and `.daedalus/todo.md`.
5. Route to exactly one current course-learning stage.

## Stage Routing

### 01 Need Aligner

Load:

- `system/prompts/course/01-need-aligner.md`

Use when the learning goal, course role, output proof, or non-goals are unclear.

### 02 Syllabus Mapper

Load:

- `system/prompts/course/02-syllabus-mapper.md`

Use when choosing chapters, mapping a course outline, or deciding what to skip/delay.

### 03 Concept Roadmap

Load:

- `system/prompts/course/03-concept-roadmap.md`

Use when chapter titles need to become mechanism questions and checkpoints.

### 04 Lesson Lab

Load:

- `system/prompts/course/04-lesson-lab.md`

Use when the user follows course code, runs notebooks, inspects datasets/tokenizers/models/loss, or asks why the code is written that way.

### 05 Mechanism Deep Dive

Load:

- `system/prompts/course/05-mechanism-deep-dive.md`

Use only when lesson output, official docs, and small experiments are not enough to explain a hidden mechanism.

### 06 Practice Transfer

Load:

- `system/prompts/course/06-practice-transfer.md`

Use when transferring a course concept to the user's real task.

### 07 Capstone Lab

Load:

- `system/prompts/course/07-capstone-lab.md`

Use when building the final course-driven mini project.

### 08 Review Loop

Load:

- `system/prompts/course/08-review-loop.md`

Use when checking whether the user can explain, reproduce, modify, or transfer concepts without the tutorial.

### 09 Closeout Archive

Load:

- `system/prompts/course/09-closeout-archive.md`

Use when closing a course topic and mining verified knowledge.

## Coaching Rules

- Advance at most one lesson, mechanism, transfer checkpoint, or review checkpoint per turn.
- Ask 1-3 high-value questions when the user's intent or understanding is still fuzzy.
- Write Agent-generated action plans to `guides/`, not `notes/`.
- Write `notes/` only after user answers, runs code, observes output, or explains a mechanism.
- For lesson labs, always separate prediction, minimum code, observed output, explanation, and next transfer.
- Treat warnings, shapes, losses, artifacts, and generated files as learning evidence, not terminal noise.
- Keep historical directories after manual reorganization unless the user approves renaming them.
- After changing lifecycle state, evidence, risks, or next action, synchronize `todo.md`, `outcome-map.md`, and relevant shared maps.
