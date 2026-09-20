---
name: parked-topic-lifecycle
overview: 追踪中途搁置专题生命周期；详细方案以 docs/plan/17 为准。
status: completed
todos:
  - id: write-formal-plan
    content: 编写 docs/plan/17-parked-topic-lifecycle.md 目标形态方案。
    status: completed
  - id: implement-lifecycle-cli
    content: 实现 parked lifecycle、park / park-limit / activate 拒绝、可配置上限。
    status: completed
  - id: sync-workspace-validate
    content: 同步 current.toml、parked-topic、validate、render、topic-board。
    status: completed
  - id: update-prompts-templates
    content: 更新模板、Agent 合同、gatekeeper / resume / CLI contract。
    status: completed
  - id: tui-and-tests
    content: TUI Parked Debt 与 CLI 测试覆盖。
    status: completed
isProject: false
---

# Codex Tracking Card

详细计划不在此处展开。本文件只用于 Codex `status` / `todos` 追踪。

## Canonical Plan

- [中途搁置专题生命周期方案](../../docs/plan/17-parked-topic-lifecycle.md)
