# 知识候选表

> 状态只能使用：`候选中`、`总结中`、`已归档`、`已忽略`。

## 主题核心


| 候选                    | 为什么值得看                                                                 | 证据                                                                                               | 状态  |
| --------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ | --- |
| 本地命令执行安全链路            | 模型不能直接执行命令，宿主必须经过 capability、approval、sandbox、retry、observation 保持控制权。 | `demo/src/tool/runtime.rs`、`demo/src/tool/shell/`、`demo/src/execution/`、`reflection/closeout.md` | 总结中 |
| 审批和沙箱分层               | 审批决定“是否允许做”，沙箱决定“以什么边界做”，二者不能混成一个布尔开关。                                 | Codex exec policy 阅读笔记、`demo/src/approval.rs`、`demo/src/retry.rs`、`reflection/closeout.md`        | 总结中 |
| sandbox denied 后的重试策略 | 失败后是否重试取决于失败类型、审批策略、重试策略、审批结果和执行尝试。                                    | `demo/src/retry.rs`、Slice 6/7 guides、retry policy tests、`reflection/closeout.md`                  | 总结中 |
| 工具事件可观察性              | 外界需要看到审批、执行尝试、沙箱/本地、重试和工具结果，才能相信工具没有被直接裸跑。                             | `demo/src/event.rs`、`demo/src/event_emitter.rs`、`demo/src/tool/runtime.rs`、`reflection/closeout.md` | 总结中 |


## 底层原理


| 候选                       | 为什么值得看                                                                                     | 证据                                                                    | 状态  |
| ------------------------ | ------------------------------------------------------------------------------------------ | --------------------------------------------------------------------- | --- |
| Tokio 异步、channel 和 waker | ReAct loop、LLM streaming、tool event、approval response 都依赖 async task、channel 和 runtime 调度。 | `demo/src/agent/react.rs`、`demo/src/agent/llm/openai.rs`、Tokio guides、`reflection/closeout.md` | 候选中 |
| Rust 流式能力实现 | LLM streaming、Agent event stream 和 TUI 增量输出都依赖 Rust 中的 Stream、channel、SSE parsing 和 delta 合并。 | `demo/src/agent/llm/openai.rs`、`demo/src/agent/react.rs`、`demo/src/cli/event_loop.rs`、`reflection/closeout.md` | 候选中 |
| `async move` 与 `tokio::spawn(async move)` | demo 并发执行 tool call 时暴露出任务调度、所有权、生命周期、JoinHandle 和 `'static` 边界的差异。 | `demo/src/tool/runtime.rs`、并发 run_tools 讨论、Tokio guides | 候选中 |
| OpenAI 兼容流式协议            | SSE `data:` 行、增量合并、tool call 参数聚合和消息回灌，是接真实模型的基础能力。                                        | `demo/src/agent/llm/openai.rs`、OpenAI integration guides              | 候选中 |
| Shell 解析和进程调用边界             | `raw_command`、`argv`、`sh -c`、heredoc、多命令和 prefix matching 是权限系统最容易误判的边界。 | Codex exec policy notes、OS runner tests、`reflection/closeout.md`                    | 候选中 |
| OS sandbox 与 `sandbox-exec` | 从模拟 runner 到真实 OS sandbox，会暴露 profile、路径转义、argv/shell quoting 和跨平台隔离差异。  | `demo/src/execution/os_execution_runner.rs`、sandbox guides、`reflection/closeout.md` | 候选中 |
| sandbox 方案的第一性原理差异          | `sandbox-exec`、Linux namespace、container、语言 VM sandbox 解决的是不同层级的隔离问题。 | sandbox-exec guides、OS runner 实现、`reflection/closeout.md`           | 候选中 |


## 工程模式


| 候选                     | 为什么值得看                                                 | 证据                                                                       | 状态  |
| ---------------------- | ------------------------------------------------------ | ------------------------------------------------------------------------ | --- |
| ToolRuntime 先规划再执行     | 先把 tool call 规划成纯函数或本地命令路径，再进入执行层，可以降低 ReAct、权限和执行层耦合。 | `demo/src/tool/runtime.rs`、`reflection/closeout.md`                         | 总结中 |
| session approval scope | 会话级授权应绑定 scope，由 `ApprovalGateway` 生命周期承载，而不是永久放行。     | `demo/src/approval.rs`、`demo/src/tool/shell/`、approval persistence tests、`reflection/closeout.md` | 总结中 |
| 事件发射器集中管理生命周期事件        | 安全事件散落在各处会漏发；集中包装 execution attempt 能让生命周期更可审计。        | `demo/src/event_emitter.rs`、事件出口复盘笔记、`reflection/closeout.md`              | 总结中 |
| ReAct observation 回灌循环 | 工具结果必须先变成 tool message 回到 messages，再进入下一轮模型推理，而不是直接当作最终答案。 | `demo/src/agent/react.rs`、`reflection/closeout.md`                         | 总结中 |
| CLI 事件循环分层             | 用户输入和 Agent 流式事件统一进入事件循环，再归约状态、渲染 UI，可以避免 UI 与 Agent 执行强耦合。 | `demo/src/cli/event_loop.rs`、`demo/src/cli/app.rs`、`demo/src/cli/view.rs`、`reflection/closeout.md` | 总结中 |
