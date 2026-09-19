# Course Learning 一等学习类型

> 日期：2026-06-23
> 状态：已完成
> 提交：`a0877c60 daedalus/course-learning: 实现课程学习类型`

## 背景

`hugging-face-llm-course` 是课程型学习材料：它有 syllabus、章节顺序、练习代码、概念依赖和官方文档。daedalus 需要让这类学习材料拥有自己的 project/topic 类型、阶段、prompt 和验证规则。

course-learning 的核心路径是：

```text
学习诉求 -> syllabus 路线 -> 概念机制 -> lesson lab -> 迁移练习 -> 掌握度验证
```

这次改造的重点是让 Agent 围绕课程掌握度推进：用户不仅要看过章节、跑通过代码，还要能解释机制、复现实验，并把概念迁移到自己的真实任务。

## 变更

- 新增正式计划：[`docs/plan/15-course-learning-system.md`](../plan/15-course-learning-system.md)。
- 新增 Codex tracking card：[`/.codex/plans/course-learning-system.md`](../../.codex/plans/course-learning-system.md)，只保留 status / todos 并链接正式计划。
- 新增 `course-learning` 初始化能力：
  - `daedalus init course-learning <project-name> --topic <topic-slug> --title <topic-title> --course-url <url>`
  - 生成 `kind = "course-learning"`、`source_kind = "course"`、`course_url = "..."`
- 手动整理现有 Hugging Face Course project：
  - 更新 project/topic metadata 为 course-learning。
  - 保留历史 topic、guides、notes、demo、reflection，不强行重命名旧目录。
- 移除长期迁移 API：
  - 不提供 `migrate` 命令族。
  - 删除旧的一次性升级脚本和对应测试。
  - 既有项目调整改为手动整理具体文件，整理后运行 validate。
- 新增 course project/topic templates：
  - `system/templates/course/`
  - `system/templates/course-topic/`
- 新增 course prompt namespace：
  - `system/prompts/course/01-need-aligner.md`
  - `system/prompts/course/02-syllabus-mapper.md`
  - `system/prompts/course/03-concept-roadmap.md`
  - `system/prompts/course/04-lesson-lab.md`
  - `system/prompts/course/05-mechanism-deep-dive.md`
  - `system/prompts/course/06-practice-transfer.md`
  - `system/prompts/course/07-capstone-lab.md`
  - `system/prompts/course/08-review-loop.md`
  - `system/prompts/course/09-closeout-archive.md`
  - `system/prompts/course/course-learning-cli-contract.md`
- 新增 `course-learning-coach` skill：`.agents/skills/course-learning-coach/SKILL.md`，用于 course / tutorial / official learning path / online curriculum。

## 生命周期

course-learning 使用独立阶段：

```text
01-need-aligner
02-syllabus-mapper
03-concept-roadmap
04-lesson-lab
05-mechanism-deep-dive
06-practice-transfer
07-capstone-lab
08-review-loop
09-closeout-archive
```

核心不变量：

```text
课程进度不是掌握度。
跑通教程不是理解机制。
章节顺序不是学习目标。
练习输出必须能回到概念机制。
概念机制必须能迁移到用户真实任务。
```

## Validate / Render

- `daedalus validate` 现在理解 `course-learning` project：
  - 检查 `source_kind = "course"`。
  - 检查 `course_url`。
  - 检查 `shared/syllabus-map.md`、`shared/course-progress.md`、`shared/concept-map.md`。
- course topic 现在有额外校验：
  - topic kind 必须是 `course-learning-topic`。
  - course stage 目录必须存在。
  - `review/` 目录必须存在。
  - `current_phase` 必须能从 `.daedalus/todo.md` 恢复。
- `state.md` 渲染会显示 project 类型、course URL 和 course shared files。

## Hugging Face Course 整理

已将现有 project：

```text
workspaces/projects/hugging-face-llm-course
```

整理前沿用旧 project metadata：

```toml
kind = "repo-learning"
source_kind = "repo"
```

整理为：

```toml
kind = "course-learning"
source_kind = "course"
course_url = "https://huggingface.co/learn/llm-course/en"
```

当前 active topic：

```text
topics/lora-feedback-loop
```

已迁移为 `course-learning-topic`，当前阶段推进到：

```text
04-lesson-lab
```

历史 `guides/04-debugger-guide/` 保留为整理前学习现场。整理后已经新增正式 course guide：

```text
guides/03-concept-roadmap/README.md
```

下一步应新增：

```text
guides/04-lesson-lab/README.md
notes/04-lesson-lab/README.md
```

## 测试与验证

- `cargo test --manifest-path crates/Cargo.toml -p daedalus-cli --test cli_workflow`
  - 34 passed
- `daedalus validate`
  - 当前 `hugging-face-llm-course` workspace valid
- `course-learning-coach` skill validator
  - passed
- commit hook
  - `cargo fmt`
  - `cargo check`
  - `cargo clippy`
  - `cargo test`
  - all passed

## 当前边界

- 本次只实现 `course-learning`，没有同时扩展到 book-learning / paper-learning。
- 没有强行重命名历史目录，避免破坏已有学习证据。
- `04-lesson-lab` 的正式 guide 还未生成，这是下一步学习任务，而不是本次 daedalus 设施实现的一部分。
