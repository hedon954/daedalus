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
- 2026-05-16 重构 repo learning 协议：后续学习以 `.daedalus/outcome-map.md` 为终点地图，`06-code-reader` 只补阻塞 `demo/design.md` 的源码缺口；当前 auth/approval/sandbox 只保留 Decision 合成、Runtime request assembly、Orchestrator retry 三个缺口。
- 2026-05-16 增量迁移 active 阶段为文件夹入口：`guides/06-code-reader/README.md` 与 `notes/06-code-reader/README.md` 是后续 code-reader 导航入口；旧 `guides/06-code-reader-guide.md` 和 `notes/code-reading.md` 保留为 legacy 产物。
- 2026-05-16 `06-code-reader` 的 auth/approval/sandbox 三个 demo 缺口已补齐：Decision 合成、Runtime request assembly、Orchestrator retry。下一步进入 `07-demo-architecture`，定稿 `demo/design.md`。
- 2026-05-16 `07-demo-architecture` 已将 demo 收敛为两阶段方案：Phase 1 使用 `SimulatedExecutionRunner` 跑通安全执行闭环，Phase 2 在同一 `ExecutionRunner` 接口下接入 `OsExecutionRunner`；已制定 AT-01 到 AT-14 验收测试和 `guides/08-demo-coder/README.md` 编码子地图。
- 2026-05-16 沉淀设计学习方法：`notes/design-method-from-source-to-demo.md`，总结从 Reality Problem 到 Build Slices 的源码学习转可迁移设计流程。
- 2026-05-16 用户确认继续后，状态推进到 `08-demo-coder`。随后 Agent 曾错误地代写 Slice 0/1 demo 代码，用户指出 08 应该一步步带用户实现；已回滚代写代码，并需要补强 repo-learning 指令中的实现练习门禁。
- 2026-05-16 用户亲手完成 `08-demo-coder` Slice 0：创建 `demo/Cargo.toml`、`demo/src/lib.rs`、`demo/src/main.rs`、`demo/Makefile` 和 `Cargo.lock`；Agent 验证 `cargo test` 通过 1 个占位测试，`cargo run` 输出 `Hello, world!`。
- 2026-06-02 Slice 7 `RetryPolicy` hardening 已完成：`decide_retry` 已消费 capability-level `RetryPolicy`，并和 `ApprovalPolicy` / `NetworkPolicy` 组合判断；`cargo test` 为 61 passed、3 ignored。下一步进入 Slice 8 Event Protocol Hardening。
- 2026-06-05 Slice 8 Event Protocol Hardening 进行中：事件类型包含 `CommandNeedsApproval`、`CommandExecutionStarted/Finished/Failed`、`CommandRetryEvaluated`；`ApprovalBroker` 使用 approval request event 与内部 `ToolApprovalResult` 的 `approval_id + oneshot` 配对；review 发现事件发送散落、approval request sender 非 run-scoped、retry 成功路径 attempt trace 不闭合。下一步按 `guides/08-demo-coder/09-slice-8-command-event-emitter-refactor.md` 引入 `CommandEventEmitter` 和 `run_execution_attempt`。

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
- `06-code-reader` 已完成核心专题收敛：`auth/approval/sandbox` 已转化为 demo 的 approval、execution、retry 不变量。
- `demo/design.md` 已从 draft 收敛为 architecture blueprint，核心数据结构、状态机、事件协议和验收用例已有设计；当前 `08-demo-coder` 正在推进 Slice 8 Event Protocol Hardening，下一步是统一 command event emitter 和 trace tests。
- 尚未进入 `09-biz-solver` 和 `10-archivist`。

## 恢复上下文提示

- 当前机器状态：`08-demo-coder` active。`04-debugger-guide`、`05-arch-analyzer`、`06-code-reader`、`07-demo-architecture` 已完成；`05-arch-analyzer` 已用 `notes/codex-agent-loop-architecture.md` 作为等价架构产物完成。
- 当前最重要的两份 notes：
  - `notes/codex-agent-loop-architecture.md`：Codex agent loop、架构分层、工具系统、权限审批、沙箱和事件流。
  - `notes/codex-context-and-compaction.md`：上下文管理、prompt view、工具结果回灌、compact、rollout 恢复和重点掌握项。
- 当前阶段入口：`guides/08-demo-coder/README.md`；demo 实现和阶段 notes/guides 以 `topics/tools-permissions` 下的 `demo/`、`notes/08-demo-coder/`、`guides/08-demo-coder/` 为准。
- 当前导航：不是继续泛读 Codex 权限系统；`auth/approval/sandbox` 已收敛成 demo 不变量。当前按 `guides/08-demo-coder/README.md` 的 slice 地图编码；下一步是用 `CommandEventEmitter` 收敛 Slice 8 的事件透出。
