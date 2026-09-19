# TUI Learning Type Adapter

> 日期：2026-06-23
> 状态：已完成

## 背景

course-learning 已经成为 daedalus 的一等学习类型，但 TUI 仍主要依赖 repo-learning 时代的阅读入口习惯：slice guide、Action Card、`10-reflection`。这会让 course-learning 虽然能显示 stage，却没有自己的课程语义。

本次改造把 TUI 的学习类型差异收敛到 adapter 层。

## 变更

- 新增正式计划：[`docs/plan/16-tui-learning-type-adapters.md`](../plan/16-tui-learning-type-adapters.md)。
- 新增 Codex tracking card：[`/.codex/plans/tui-learning-type-adapters.md`](../../.codex/plans/tui-learning-type-adapters.md)。
- 新增 `crates/daedalus-cli/src/interfaces/tui/learning_types.rs`，内聚 `LearningTypeTuiAdapter`、guide 查找、next action 构造和 source reference 策略：
  - `repo-learning` adapter 保留 slice guide / Action Card 阅读策略。
  - `course-learning` adapter 使用 course stage README 阅读策略。
  - fallback adapter 支持未知学习类型先按 stage README 工作。
- `app.rs` 只负责从 state / filesystem 装配 `TuiOverview`，不再承载 repo/course 的具体 guide 解析细节。
- `TuiOverview` 增加 learning type、source kind、source reference、类型化进度单位和当前阅读标签。
- TUI overview 现在能显示：
  - learning type。
  - source kind / course URL。
  - `Course Stage` / `course stages` 这类类型化文案。
  - course guide 入口。
- closeout 逻辑不再硬编码 `10-reflection`：
  - repo-learning 使用 `10-reflection`。
  - course-learning 使用 `09-closeout-archive`。

## 测试

- 保留 repo-learning action guide 行为测试。
- 新增 course-learning overview fixture，验证：
  - 识别 `course-learning`。
  - 展示 course URL。
  - 当前阅读入口使用 `guides/04-lesson-lab/README.md`。
  - next action 使用 `Course Stage` 和 `Course Guide` 文案。

已验证：

- `cargo fmt --manifest-path crates/Cargo.toml --check`
- `cargo check --manifest-path crates/Cargo.toml -p daedalus-cli`
- `cargo test --manifest-path crates/Cargo.toml -p daedalus-cli interfaces::tui::learning_types::tests`
- `cargo test --manifest-path crates/Cargo.toml -p daedalus-cli interfaces::tui::app::tests`
- `cargo test --manifest-path crates/Cargo.toml -p daedalus-cli --test cli_workflow`
- `daedalus validate`
- `git diff --check`

## 当前边界

- 没有重做 TUI 视觉设计。
- 没有实现动态插件系统。
- 新增学习类型如果需要专属语义，应新增 adapter；如果只遵守 stage README 约定，可以先走 fallback adapter。
