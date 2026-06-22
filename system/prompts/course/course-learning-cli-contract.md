---
title: Course Learning CLI Contract
description: course-learning 的 CLI、state、template 和 validate 契约。用于 Agent 操作课程学习 project/topic 时保持一致。
phase: course.contract
---

# Course Learning CLI Contract

## Commands

```bash
daedalus init course-learning <project-name> \
  --topic <topic-slug> \
  --title <topic-title> \
  --course-url <url>

daedalus migrate course-learning <project-dir> \
  --course-url <url> \
  --execute
```

## State Contract

Project:

```toml
[task]
kind = "course-learning"

[project]
source_kind = "course"
course_url = "..."
```

Topic:

```toml
[topic]
kind = "course-learning-topic"
current_phase = "01-need-aligner"
```

## Stage Names

- `01-need-aligner`
- `02-syllabus-mapper`
- `03-concept-roadmap`
- `04-lesson-lab`
- `05-mechanism-deep-dive`
- `06-practice-transfer`
- `07-capstone-lab`
- `08-review-loop`
- `09-closeout-archive`

## Hard Rules

- Do not create new course-learning guides under repo-learning names.
- Keep historical repo-named guides after migration unless the user approves a rename.
- Validate structure with `daedalus validate` after state or lifecycle changes.
