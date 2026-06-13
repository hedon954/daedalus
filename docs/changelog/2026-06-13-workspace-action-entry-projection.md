# Workspace 行动入口投影收窄

> 日期：2026-06-13
> 状态：已完成

## 背景

当一个 topic 进入 `awaiting-reflection` 后，顶层同时出现 `current-project`、`closeout-project`、`closeout-topic`，会让学习者误以为当前仍然应该进入 project 做日常推进。

顶层 workspace 应该回答“我现在该点哪里继续行动”，而不是展示全部内部状态。

## 变更

- `current.toml` 继续保存完整机器状态：
  - `current_project`
  - `current_topic`
  - `pending_closeout_project`
  - `pending_closeout_topic`
- `current-project` / `current-topic` 只在存在 active topic 时投影。
- pending closeout 只投影 `closeout-topic`。
- 不再投影 `closeout-project`。
- `validate` 会把没有 active topic 时残留的 `current-project` symlink 视为不一致。
- 生命周期命令会自动刷新 VSCode rust-analyzer `linkedProjects`。
- rust-analyzer 同步会纳入 `current-topic/demo/Cargo.toml` 和 `closeout-topic/demo/Cargo.toml`，让学习者从顶层行动入口进入 demo 时仍有语法分析。

## 当前 Codex 示例

当前 `tools-permissions` 处于 `awaiting-reflection`：

```text
workspaces/closeout-topic -> projects/openai-codex-cli-deep-learning/topics/tools-permissions
```

没有 active topic，所以顶层不再展示 `current-project` / `current-topic`。
