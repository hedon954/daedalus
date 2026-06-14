# 03 问题路线图指南

## 设计意图

本阶段不直接讲解 Codex 源码，而是先用 3 个问题逼近后续阅读主线：

1. 现实约束：Codex CLI 作为本地 coding agent，必须解决什么生产问题？
2. 核心路径：一次用户 prompt 从 CLI 入口进入后，最可能经过哪些边界？
3. 不变量：如果要做 mini demo，哪些架构决策必须保留，哪些细节可以舍弃？

## 已观察到的源码线索

- 顶层 README 表明 Codex CLI 是“runs locally on your computer”的 coding agent。
- `codex-rs/README.md` 说明 Rust CLI 是当前维护版本，关键 crate 包括 `core/`、`exec/`、`tui/`、`cli/`。
- `codex-rs/cli/src/main.rs` 是 multitool 入口，负责 subcommand 分发，默认进入 interactive CLI。
- `codex-rs/exec/src/main.rs` 是非交互执行入口，适合 runbook 的第一条可验证核心路径。
- `codex-rs/tui/src/main.rs` 是交互式 TUI 入口。
- `codex-rs/core/src/lib.rs` 暴露 `CodexThread`、`ThreadManager`、`ModelClient`、`exec`、`sandboxing`、`tools` 等主线模块。

## 第一轮问题

1. 如果 Codex CLI 必须“在用户本机安全地完成代码任务”，它最容易失败的三个地方是什么？例如上下文错误、误执行命令、权限越界、模型输出不可控等。
2. 你猜一次 `codex exec "..."` 或交互式 prompt 的核心链路会如何流动？请先用自己的话写出 5-8 个节点，不要求准确。
3. 如果我们要做一个 mini demo，只保留 Codex 的核心思想，你认为必须保留哪些不变量？例如会话、工具注册、审批、沙箱、事件流、上下文裁剪等。

## 回答后的验证路径

- 问题 1 对应后续架构分析：找出 Codex 如何把失败点转成模块边界。
- 问题 2 对应运行调试：优先追踪 `codex-rs/exec/src/main.rs` 到 `codex_exec::run_main`，再进入 `codex-core`。
- 问题 3 对应 demo 设计：后续只实现能证明这些不变量的最小闭环。

## 推荐下一步

用户先回答或改写这 3 个问题。Agent 再把回答整理进 `notes/03-socratic-coach/README.md`，并生成下一阶段 runbook。
