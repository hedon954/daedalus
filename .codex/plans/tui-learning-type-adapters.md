---
name: tui-learning-type-adapters
overview: 追踪 daedalus TUI 对 repo-learning / course-learning 的解耦适配；详细方案以 docs/plan/16-tui-learning-type-adapters.md 为准。
status: completed
todos:
  - id: write-tui-adapter-plan
    content: 编写 TUI learning type adapter 正式计划。
    status: completed
  - id: implement-learning-type-adapter
    content: 在 TUI app 层引入 learning-type adapter，并接入 repo/course。
    status: completed
  - id: update-tui-screens
    content: 让 TUI screens 展示 learning type、source reference 和类型化进度文案。
    status: completed
  - id: add-course-tui-tests
    content: 补充 course-learning TUI fixture 和 guide 查找测试。
    status: completed
  - id: write-changelog
    content: 实现完成后写 changelog 总结。
    status: completed
isProject: false
---

# Codex Tracking Card

详细计划不在此处展开。本文件只用于 Codex `status` / `todos` 追踪。

## Canonical Plan

- [TUI Learning Type Adapter 实现方案](../../docs/plan/16-tui-learning-type-adapters.md)

## Tracking Rule

- `docs/plan/`：正式详细计划，作为 daedalus 项目的长期设计资料。
- `.codex/plans/`：Codex 执行追踪卡，只保留 `status`、`todos` 和正式计划链接。
