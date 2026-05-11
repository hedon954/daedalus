# 长期上下文

只保留可长期复用的上下文，不要粘贴聊天记录。

## 角色边界

- `guides/` 保存 Agent 给用户的行动指南。
- `notes/` 保存用户亲自实践和思考后的学习笔记。
- `source/` 保存外部源码缓存，默认不提交到 daedalus 仓库。
- `.daedalus/validation-log.md` 保存 daedalus 自身教学效果的复盘。

## 目标

- 深入学习 OpenAI Codex CLI 源码，重点掌握生产级 Agent CLI 的核心执行链路、上下文管理、工具调用、权限/沙箱边界，并抽取可复用到自有 Agent/CLI 项目的 mini demo 不变量。

## 决策

- 学习对象固定为 `openai/codex`，源码通过 `source/pull_source.sh` 拉取到 `source/codex`，固定 commit `ebe75bb683b3c237aad9f039ab17b187048aa499`。
- 架构总览不再拆散成多篇阶段性 notes；最终架构笔记集中在 `notes/codex-agent-loop-architecture.md`。
- 复杂全局图优先使用 Excalidraw，局部调用链和控制流使用 Typora 兼容 Mermaid。
- `notes/codex-context-and-compaction.md` 是 `06-code-reader` 的第一个核心专题笔记，保留用户原始回答、Agent 校准、源码验证路径和验证状态。
- 2026-05-11 回退到 `06-code-reader`：后续源码阅读必须按“生产问题 -> naive 失败 -> 源码应对 -> 保护的不变量 -> trade-off -> 可迁移模式”推进。

## 已验证结论

- 最小运行验证已完成：用户执行 `codex --help`、`codex exec --help` 和 `cargo test -p codex-exec`，其中 `codex-exec` 测试结果为 61 passed、0 failed。
- Codex 核心交互循环可以概括为：入口/UI 提交请求，app/session 形成 turn，`RegularTask` 循环调用 `run_turn`，`run_sampling_request` 消费模型 stream，工具调用经 router/runtime/orchestrator 执行后把结果写回 history，再进入下一轮模型请求。
- 工具权限检查是分层的：工具分发前有可用性/门禁，具体工具运行时再处理审批、沙箱选择和失败重试；`ToolOrchestrator` 是审批、沙箱和 retry 语义的中心。
- 上下文管理不能理解成单一 append-only `messages`：`ContextManager` 是运行时 history，`for_prompt` 是本次模型可见 history，`Rollout` 是可恢复持久化日志。
- ReAct 因果链的关键不变量是 tool call 和 tool output 必须一一对应；`normalize_history`、`call_id`、`ResponseInputItem` 都在保护这条链路。
- Compact 是一次 history rewrite，不只是摘要。`replacement_history` 是恢复 checkpoint，`InitialContextInjection` 处理 pre-turn 与 mid-turn compact 的上下文连续性差异。
- 工具执行用 future 是为了让模型 stream、工具执行、history 写入解耦；Codex 用 `FuturesOrdered` 保证并发工具结果仍按模型 tool call 顺序写回。

## 未解决问题

- 交互式 TUI 断点方案尚待用户在 Cursor 中打开 `source/codex/codex-rs` 后验证。
- `06-code-reader` 需要重读核心专题：优先从工业级 Agent CLI 的真实失败模式切入，例如 `auth/approval/sandbox` 如何防止本地执行事故。
- 尚未进入 `07-demo-architecture`、`08-demo-coder`、`09-biz-solver` 和 `10-archivist`。

## 恢复上下文提示

- 当前机器状态：`06-code-reader` active，最近一次状态流转是 `rollback`。`04-debugger-guide` 已完成；`05-arch-analyzer` 已用 `notes/codex-agent-loop-architecture.md` 作为等价架构产物完成。
- 当前最重要的两份 notes：
  - `notes/codex-agent-loop-architecture.md`：Codex agent loop、架构分层、工具系统、权限审批、沙箱和事件流。
  - `notes/codex-context-and-compaction.md`：上下文管理、prompt view、工具结果回灌、compact、rollout 恢复和重点掌握项。
- 下一步建议：以 `auth/approval/sandbox` 为专题，先让用户回答本地执行在生产环境会造成哪些事故，再读 Codex 的审批、沙箱、重试和禁止策略。
