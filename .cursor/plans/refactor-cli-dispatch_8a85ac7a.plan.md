---
name: refactor-cli-dispatch
overview: 将 daedalus Agent CLI 从集中式 match 分发重构为 rcli 风格的 trait + enum_dispatch 命令执行模型，并接入 tokio、tracing 和 tracing-subscriber。
todos:
  - id: add-cli-dispatch-deps
    content: 添加 enum_dispatch、tokio、tracing、tracing-subscriber 依赖，并保持 edition 2024。
    status: completed
  - id: introduce-cmd-executor
    content: 新增 CmdExecutor trait 和 Agent CLI 执行上下文，承载输出格式与 repo root。
    status: completed
  - id: split-agent-cli-commands
    content: 将 init/state/validate 执行逻辑从 bin 拆到 commands 模块，并用 enum_dispatch 分发。
    status: completed
  - id: simplify-daedalus-main
    content: 把 daedalus.rs 简化为 tracing 初始化、Cli::parse、command.execute().await 和统一错误输出。
    status: completed
  - id: update-cli-guidance
    content: 更新 crate CLAUDE.md，记录后续新增命令的 trait + enum_dispatch 模板。
    status: completed
  - id: verify-refactor
    content: 运行 cargo fmt、cargo doc 和 make ci，确保命令行为与测试保持稳定。
    status: completed
isProject: false
---

# 重构 CLI 命令分发

## 目标

将当前集中在 [`crates/daedalus-cli/src/bin/daedalus.rs`](crates/daedalus-cli/src/bin/daedalus.rs) 的大块 `match` 分发，重构为类似 `/Users/hedon/rust/hedon-rust-road/rcli` 的模板化命令执行模型：

```rust
let opts = Cli::parse();
opts.command.execute().await?;
```

每个命令参数结构自己实现执行逻辑，减少后续新增命令时需要改动的集中式分支，降低 Agent 生成重复样板代码和遗漏分支的概率。

## 实施方案

1. 更新依赖与入口

- 在 [`crates/daedalus-cli/Cargo.toml`](crates/daedalus-cli/Cargo.toml) 添加最新稳定依赖：`enum_dispatch`、`tokio`、`tracing`、`tracing-subscriber`。
- 将 [`crates/daedalus-cli/src/bin/daedalus.rs`](crates/daedalus-cli/src/bin/daedalus.rs) 改为 async main：初始化 tracing、解析 CLI、执行命令、统一输出错误。
- 保留现有稳定文本/JSON presenter 行为，不改变 CLI 对外命令语义。

2. 引入命令执行 trait 和上下文

- 在 `interfaces/agent_cli` 下新增或整理：
  - `executor.rs`：定义 `CmdExecutor` trait。
  - `context.rs`：封装 `format`、`repo_root` 等执行上下文。
- 避免每个命令重复计算 repo root。顶层入口负责创建上下文，命令实现从上下文读取需要的信息。

3. 拆分命令模块

- 将 [`crates/daedalus-cli/src/interfaces/agent_cli/args.rs`](crates/daedalus-cli/src/interfaces/agent_cli/args.rs) 中的命令结构保留为 clap 参数定义，但将执行逻辑拆到按命令组织的模块，例如：
  - `commands/init.rs`
  - `commands/state.rs`
  - `commands/validate.rs`
- 顶层 `Command` 和必要的嵌套 enum 使用 `#[enum_dispatch(CmdExecutor)]`。
- 对嵌套子命令保留清晰边界：`InitCommand` 委托到 `InitKind`，`StateCommand` 委托到 `StateSubcommand`，但每层都通过 trait execute，而不是在 bin 里集中 match。

4. 迁移现有逻辑

- 将当前 `daedalus.rs` 中的 init/state/validate 逻辑移动到对应命令结构的 `execute` 实现中。
- 保持现有 application use case 不变：
  - `init_repo_learning`
  - `transition_stage`
  - `render_state`
  - `validate_workspace`
- 保持现有错误类型和 presenter 输出不变，确保测试无需大规模改写。

5. 日志策略

- 在 main 中初始化 `tracing_subscriber`，优先使用 `EnvFilter`，默认保持安静，避免污染 Agent-friendly stdout。
- 普通命令结果仍走 presenter 的 stdout/stderr；tracing 用于 debug 级内部诊断。

6. 文档与约束更新

- 更新 [`crates/daedalus-cli/CLAUDE.md`](crates/daedalus-cli/CLAUDE.md)，补充新增命令时的模式：新增 args struct、实现 `CmdExecutor`、挂到 enum_dispatch enum、补测试。
- 若根 [`CLAUDE.md`](CLAUDE.md) 中 Rust crate 维护规则需要提及 CLI 命令模式，则轻量补充。

7. 验证

- 运行 `cargo fmt --manifest-path crates/Cargo.toml --all`。
- 运行 `cargo doc --manifest-path crates/Cargo.toml --workspace --no-deps`，确认 rustdoc 注释仍有效。
- 运行 `make ci`，确保现有 8 个 integration tests 继续通过，并补一个 smoke test 验证新入口仍能执行 `init`。
