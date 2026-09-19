# 中途搁置专题生命周期

> 日期：2026-09-19
> 对应计划：`docs/plan/17-parked-topic-lifecycle.md`
> 状态：已完成

## 变更摘要

- 新增 topic lifecycle：`parked`。
- 新增 CLI 命令：
  - `daedalus topic park <topic-slug> --reason <reason>`
  - `daedalus topic park-limit <n>`
- `daedalus topic activate` 在已有另一个 active topic 时拒绝，不再把旧题偷偷标成 `blocked`。
- 扩展 `workspaces/.daedalus/current.toml`：
  - `max_parked_topics`（默认 1，合法范围 1-3）
  - `parked_topics`
- 新增人类可点击入口：`workspaces/parked-topic`，指向最近一次搁置的专题。
- `validate` 会检查 project / topic lifecycle 是否一致，以及 parked 列表、上限和 symlink 是否对齐。
- TUI、resume、gatekeeper、CLI contract 和 topic-board 区分 active、parked、awaiting-reflection。

## 设计决策

- `active`：今天正在推进。
- `parked`：学到一半，用户主动搁置，稍后用 `activate` 接回。
- `awaiting-reflection`：主体学完，只欠 closeout。
- parked 个数可配置，但硬顶是 3，避免无限搁置把 WIP 纪律打穿。

默认上限是 1。用户可以把上限调到 2 或 3，但不能调到 0 或 4。

## 验证

新增测试覆盖：

- 已有 active 时 `activate` 另一题被拒绝。
- `park` 释放 `current-topic`，并同时更新 project 与 topic lifecycle。
- 默认只能搁置 1 题；`park-limit 2` 后可以再搁置一题并接回旧题。
- `park-limit 0` / `4` 被拒绝。
