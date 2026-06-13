# Closeout Pending Topic 生命周期

> 日期：2026-06-12
> 对应计划：`docs/plan/09-closeout-pending-topic-lifecycle.md`
> 状态：已完成

## 变更摘要

- 新增 topic lifecycle：`awaiting-reflection`。
- 新增 CLI 命令：`daedalus topic await-reflection <topic-slug> --reason <reason>`。
- 将“日常 active 学习主题”和“等待用户主动回顾的 closeout debt”拆成两条状态线。
- 扩展 `workspaces/.daedalus/current.toml`：
  - `current_project`
  - `current_topic`
  - `pending_closeout_project`
  - `pending_closeout_topic`
- 新增人类可点击入口：
  - `workspaces/closeout-topic`
- 顶层 symlink 只作为行动入口：没有 active topic 时不展示 `current-project`；pending closeout 只展示 `closeout-topic`。
- `validate` 会检查 active topic、pending closeout topic、`current.toml` 和 symlink 投影是否一致。
- `TUI` 在无 active topic 时可以回落展示 awaiting-reflection topic，避免 closeout 现场丢失。
- 更新 `AGENTS.md`、`CLAUDE.md`、repo-learning skill、resume prompt、CLI contract 和 topic 模板，明确 active topic 与 pending closeout 的差异。

## 设计决策

- `active`：当前正在消耗日常认知资源推进的学习主题。
- `awaiting-reflection`：主体学习、demo、业务迁移和测试验证已经完成，但用户还没完成 closeout reflection。
- `completed`：用户 closeout、Agent challenge、用户确认和 knowledge-base 归档都完成。

这让 daedalus 可以保持 WIP 专注，同时承认真实学习节奏：完整 closeout 需要大块注意力，不应该长期占用每天 1-2 小时的 active learning slot。

## 当前 Codex 示例

- Project：`workspaces/projects/openai-codex-cli-deep-learning`
- Topic：`tools-permissions`
- Project lifecycle：`idle`
- Topic lifecycle：`awaiting-reflection`
- Active topic：无
- Pending closeout：`workspaces/closeout-topic -> projects/openai-codex-cli-deep-learning/topics/tools-permissions`

下一步不是继续日常实现，而是由用户在大块时间填写 `reflection/closeout.md`，之后再进入 Agent challenge 和知识归档。

## 验证

已通过：

```bash
cargo fmt --manifest-path crates/Cargo.toml --check
cargo test --manifest-path crates/Cargo.toml -p daedalus-cli
cargo run --manifest-path crates/Cargo.toml -p daedalus-cli --bin daedalus -- validate workspaces/projects/openai-codex-cli-deep-learning --all-topics
```

新增测试覆盖：

- topic 进入 `awaiting-reflection` 后释放 `current-topic`。
- pending closeout 不阻塞新的 active topic。
- 未完成主体阶段时不能进入 `awaiting-reflection`。
- 同时只能有一个 pending closeout。
- `validate` 能发现 closeout projection 缺失或不一致。
