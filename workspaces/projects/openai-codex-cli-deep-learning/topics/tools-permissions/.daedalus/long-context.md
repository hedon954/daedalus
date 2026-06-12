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
- 架构总览集中在 `notes/05-arch-analyzer/01-codex-agent-loop-architecture.md`，阶段入口为 `notes/05-arch-analyzer/README.md`。
- 复杂全局图优先使用 Excalidraw，局部调用链和控制流使用 Typora 兼容 Mermaid。
- `notes/06-code-reader/01-context-and-compaction.md` 是 `06-code-reader` 的第一个核心专题笔记，保留用户原始回答、Agent 校准、源码验证路径和验证状态。
- 2026-05-11 回退到 `06-code-reader`：后续源码阅读必须按“生产问题 -> naive 失败 -> 源码应对 -> 保护的不变量 -> trade-off -> 可迁移模式”推进。
- 2026-05-16 重构 repo learning 协议：后续学习以 `.daedalus/outcome-map.md` 为终点地图，`06-code-reader` 只补阻塞 `demo/design.md` 的源码缺口；当前 auth/approval/sandbox 只保留 Decision 合成、Runtime request assembly、Orchestrator retry 三个缺口。
- 2026-05-16 增量迁移 active 阶段为文件夹入口：`guides/06-code-reader/README.md` 与 `notes/06-code-reader/README.md` 是后续 code-reader 导航入口；后续已把早期 code-reader 根目录产物收拢到该阶段目录。
- 2026-05-16 `06-code-reader` 的 auth/approval/sandbox 三个 demo 缺口已补齐：Decision 合成、Runtime request assembly、Orchestrator retry。下一步进入 `07-demo-architecture`，定稿 `demo/design.md`。
- 2026-05-16 `07-demo-architecture` 已将 demo 收敛为两阶段方案：Phase 1 使用 `SimulatedExecutionRunner` 跑通安全执行闭环，Phase 2 在同一 `ExecutionRunner` 接口下接入 `OsExecutionRunner`；已制定 AT-01 到 AT-14 验收测试和 `guides/08-demo-coder/README.md` 编码子地图。
- 2026-05-16 沉淀设计学习方法：`notes/07-demo-architecture/01-design-method-from-source-to-demo.md`，总结从 Reality Problem 到 Build Slices 的源码学习转可迁移设计流程。
- 2026-05-16 用户确认继续后，状态推进到 `08-demo-coder`。随后 Agent 曾错误地代写 Slice 0/1 demo 代码，用户指出 08 应该一步步带用户实现；已回滚代写代码，并需要补强 repo-learning 指令中的实现练习门禁。
- 2026-05-16 用户亲手完成 `08-demo-coder` Slice 0：创建 `demo/Cargo.toml`、`demo/src/lib.rs`、`demo/src/main.rs`、`demo/Makefile` 和 `Cargo.lock`；Agent 验证 `cargo test` 通过 1 个占位测试，`cargo run` 输出 `Hello, world!`。
- 2026-06-02 Slice 7 `RetryPolicy` hardening 已完成：`decide_retry` 已消费 capability-level `RetryPolicy`，并和 `ApprovalPolicy` / `NetworkPolicy` 组合判断；`cargo test` 为 61 passed、3 ignored。下一步进入 Slice 8 Event Protocol Hardening。
- 2026-06-06 Slice 8 Event Protocol Hardening 已完成：事件类型包含 `CommandNeedsApproval`、`CommandExecutionStarted/Finished/Failed`、`CommandRetryEvaluated`；已从旧 `ApprovalController / ApprovalBroker` 重构为 `ApprovalGateway + PendingApproval`，由 `run_shell_command` 通过当前 run 的 `EventSender` 发送 `CommandNeedsApproval`，内部 `ToolApprovalResult` 通过 `approval_id + oneshot` 回流；send failure fail closed、pending cleanup、retry 成功路径 attempt trace、`CommandEventEmitter` 和 `run_execution_attempt` 均已闭合，`cargo test` 为 63 passed、3 ignored。下一步进入 Slice 9 multi-tool independent execution。
- 2026-06-06 Slice 9 策略已定稿：同批 tool calls 互不影响；能并发就并发执行；每个 call 独立返回 `Finished / Failed / Denied`；不因为某个 call 失败或被拒而取消其他 call；最终按原始 index 稳定回灌 observations。
- 2026-06-08 Slice 9 已实现并收口：`ToolRuntime::batch_run` 使用 `tokio::spawn` 并发执行同批 tool calls，并按原始 index 稳定返回 results；`ToolEventEmitter` 统一发出 `ToolRunStarted/Finished/Failed`；`react.rs` 只负责把 results 写成下一轮 LLM 的 `role=tool` observations。已补 runtime tool event tests、mixed batch tests 和 approval-blocked batch 并发验证，`cargo test` 为 65 passed、3 ignored。下一步进入 Slice 10 approval persistence，或先补 `demo/README.md` trace/runbook。
- 2026-06-10 `08-demo-coder` Phase 2 已收口：`OsExecutionRunner` 接入 macOS `sandbox-exec` 并增加 execution timeout，`ratatui` Agent CLI REPL 接入真实 `ReActAgent`，支持 prompt、Codex-like transcript、thinking/text delta 合并、assistant/thinking Markdown 渲染、GFM table 终端兜底、同进程短期 messages memory、滚动、approval once/session/reject 和无副作用 `echo approval-test` 验收能力。TUI loop 已从同步键盘 poll 改为 keyboard channel + agent event channel 的 `tokio::select!`，修复“只有输入时才继续刷新 agent events”的问题；Transcript tail scroll 已改为按视觉行计算，修复长文本换行后最终 `turn completed` 不可见的问题；`run_command` 已对 heredoc、重定向、管道和命令串联等复杂 shell syntax fail closed，避免 prefix 白名单误放行；UI 已参考 Codex / Claude Code 的 terminal agent 交互原则，从大框仪表盘改为 transcript-first。`cargo test` 为 90 passed、3 ignored。

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
- `demo/design.md` 已从 draft 收敛为 architecture blueprint，核心数据结构、状态机、事件协议和验收用例已有设计；当前 `08-demo-coder` 已完成 Phase 1/2，包括真实 OS sandbox runner 和真人 TUI approval UI。
- `09-biz-solver` 已输出 `notes/09-biz-solver/README.md`，把安全本地命令执行链路迁移为业务 Agent/CLI 设计方案。
- `10-archivist` 已回滚为 active：当前不能使用 Agent 生成的知识库条目替代用户总结。下一步需要用户先完成 `reflection/closeout.md`，再由 Agent challenge、补外部参照、组织链接，并在用户确认后决定是否进入 `knowledge-base/`。

## 恢复上下文提示

- 当前机器状态：`08-demo-coder` 与 `09-biz-solver` 已有完成产物；`10-archivist` 尚未完成，因为用户还没有写 closeout reflection。当前收口重点是先让用户完成 `reflection/closeout.md`，再进行 challenge、外部参照和知识库归档。`04-debugger-guide`、`05-arch-analyzer`、`06-code-reader`、`07-demo-architecture` 已完成；`05-arch-analyzer` 已用 `notes/05-arch-analyzer/01-codex-agent-loop-architecture.md` 作为架构专题产物。
- 当前最重要的两份 notes：
  - `notes/05-arch-analyzer/01-codex-agent-loop-architecture.md`：Codex agent loop、架构分层、工具系统、权限审批、沙箱和事件流。
  - `notes/06-code-reader/01-context-and-compaction.md`：上下文管理、prompt view、工具结果回灌、compact、rollout 恢复和重点掌握项。
- 当前阶段入口：`guides/08-demo-coder/README.md`；demo 实现和阶段 notes/guides 以 `topics/tools-permissions` 下的 `demo/`、`notes/08-demo-coder/`、`guides/08-demo-coder/` 为准。
- 当前导航：不是继续泛读 Codex 权限系统，也不是继续扩展完整 Codex TUI；`auth/approval/sandbox` 已收敛成 demo 不变量，`08-demo-coder` Phase 2 已完成。下一步是用户主动 closeout reflection，而不是让 Agent 直接生成知识库。
