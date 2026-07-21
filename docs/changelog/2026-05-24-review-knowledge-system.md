# 2026-05-24 Review 与 Knowledge System 落地

## Summary

实现 daedalus 的复习计划与知识体系萃取第一版完整能力。复习现在是挂载在 project/topic 上的独立生命周期，不会重新打开 learning stage；知识萃取通过 topic candidate -> shared verified candidate -> knowledge-base candidate 的 promotion pipeline 推进。

## Changes

- 新增 `review` CLI：
  - `start`
  - `list`
  - `show`
  - `session start`
  - `session complete`
  - `complete`
  - `abandon`
  - `render`
  - `validate`
- 新增 `knowledge` CLI：
  - `extract`
  - `promote --to shared`
  - `export --to knowledge-base`
  - `list`
  - `validate`
- 新增 `validate --reviews --knowledge`。
- 新增 review / knowledge-system domain、application 和 presenter。
- 新增 review / knowledge-system templates。
- 更新 repo-learning skill、resume、reflection、export-knowledge，使复习与萃取都遵守第一性原理链路。
- TUI 增加 read-only Review Focus 和 Knowledge Focus。
- 当前 Codex learning workspace 已手动补齐 review 与 knowledge-system scaffolding。
- 当前 Codex learning workspace 已补齐 review 与 knowledge-system scaffolding。

## First-Principles Gate

复习和知识萃取必须从这条链路出发：

```text
业务目标 / 现实任务
  -> 现实制约
  -> naive solution 为什么失败
  -> 核心抽象 / 不变量
  -> 实现机制
  -> trade-off
  -> 对比最佳实践
  -> 可迁移模式
  -> 复习题 / 应用题
```

## Validation

- `cargo fmt --manifest-path crates/Cargo.toml --check`
- `cargo clippy --manifest-path crates/Cargo.toml --workspace --all-targets --all-features --tests --benches -- -D warnings`
- `cargo test --manifest-path crates/Cargo.toml -p daedalus-cli`
- `system/bin/audit-daedalus-agent-instructions .`
- `daedalus validate workspaces/02-learning/openai-codex-cli-deep-learning --all-topics --reviews --knowledge`
