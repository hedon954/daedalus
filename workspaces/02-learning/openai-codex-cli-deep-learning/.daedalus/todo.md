# Todo

Todo 是动态的。学习证据变化、阶段完成、学习路径需要收窄或扩展时，都要及时调整它。

> Agent 负责拆解、指导、排障和验收；用户负责关键实践、观察和手写笔记。不要把 Agent 自动完成的事项伪装成用户已经掌握。

## 当前

- [x] 完成 `04-debugger-guide`：用户已执行最小 runbook，`codex-exec` 测试通过。
- [x] 完成 `05-arch-analyzer`：形成 `notes/codex-agent-loop-architecture.md`，覆盖 agent loop、工具、权限、沙箱和事件回流。
- [ ] 进行 `06-code-reader`：深读核心代码，当前已完成上下文管理 / compact 机制专题。

## 下一步

- [x] 选择候选 repo。
- [x] 用户回答或改写第一轮 3 个问题。
- [x] 创建问题路线图。
- [x] 为当前阶段创建或更新 [`guides/`](../guides) 中的行动指南。
- [x] 在 [`validation-log.md`](validation-log.md) 中记录 daedalus 的引导效果。
- [x] 创建 `codex exec` 核心路径 runbook。
- [x] 用户亲自执行 runbook 中的最小命令并记录观察。
- [x] 沿 Codex 核心 turn/session 链路追踪到 `RegularTask`、`run_turn`、`run_sampling_request` 和工具执行入口。
- [ ] 用户在 Cursor 中打开 `source/codex/codex-rs`，使用 repo-local `.vscode/launch.json` 验证交互式 TUI 断点能命中。
- [x] 在 TUI 执行循环笔记中补充简约 Mermaid 图。
- [x] 以“你好”为例梳理 TUI 用户 prompt 的前向提交和事件回流链路。
- [x] 补充用户 prompt 事件流的关键组件概念、分层模型和通道解耦图。
- [x] 梳理 `RegularTask`、`run_sampling_request`、工具执行、审批和沙箱入口。
- [x] 明确工具权限检查分层：`dispatch_any` 前置门禁与具体 `ToolRuntime` 审批/沙箱。
- [x] 补充 `ToolOrchestrator` 在审批、沙箱选择和重试中的中心角色。
- [x] 提炼 TUI 交互循环、agent loop、工具分发、权限审批和沙箱执行，重写为单一最终结论 notes。
- [x] 使用 Excalidraw 重绘 Codex 核心交互循环与 Tool System 图，并嵌入最终结论 notes。
- [x] 梳理上下文管理、history、prompt 构建、工具结果回灌、rollout 恢复与 compact 机制。
- [ ] 继续深读一个核心专题：建议在 `auth/approval/sandbox` 和 `mini demo 设计前的不变量清单` 中二选一。

## 阻塞

- 暂无。

## 后续

- [ ] 视需要补做交互式 TUI 断点验证。
- [ ] 将核心代码阅读收敛成 demo 不变量清单。
- [ ] 设计并实现 mini demo。
- [ ] 将学习结果迁移到业务问题。
- [ ] 归档已验证知识。

## 已完成

- 2026-05-09：用户确认学习对象为 OpenAI Codex CLI，重点为 Agent 主循环、沙箱/权限和整体架构，输出为 runbook、架构笔记、核心代码精读和 mini demo。
- 2026-05-09：选择 `openai/codex`，通过 `source/pull_source.sh` 拉取源码到 `source/codex`，固定 commit `ebe75bb683b3c237aad9f039ab17b187048aa499`。
- 2026-05-09：基于 `codex-rs` 顶层入口和关键 crate 生成第一轮问题路线图草案，等待用户回答或改写。
- 2026-05-09：用户回答第一轮问题，形成需求覆盖、语法/运行正确性、上下文丢失三类失败点，以及 CLI -> agent 构建 -> LLM adapter -> ReAct loop -> 权限/tool gateway -> hooks 的链路假设。
- 2026-05-09：生成 `codex exec` 运行调试指南和 runbook 草案，等待用户亲自执行最小命令。
- 2026-05-09：用户执行 `codex --help`、`codex exec --help` 和 `cargo test -p codex-exec`；`codex-exec` 61 个测试全部通过。
- 2026-05-09：补充 Cursor + CodeLLDB 交互式 TUI 和 `exec` 路径断点调试方案。
- 2026-05-09：修正调试配置位置：`launch.json` 应放在 `source/codex/codex-rs/.vscode/`，单独用 Cursor 打开 `codex-rs` 时 `cwd` 使用 `${workspaceFolder}`。
- 2026-05-09：将 TUI 主循环、用户 prompt 前向提交/事件回流、`RegularTask` agent loop、工具分发、权限审批、`ToolOrchestrator` 与沙箱执行重新提炼为最终结论版 `notes/codex-agent-loop-architecture.md`，删除阶段性重复 notes。
- 2026-05-09：根据源码复核，将 `notes/codex-agent-loop-architecture.md` 再次重写为更简洁的架构笔记：核心流程 loop 总览、架构分层、输入启动、ReAct loop、tool system、auth/approval、sandbox、event 回流和核心 struct 关系。
- 2026-05-09：生成 Codex agent loop、架构分层、tool system 三类 Excalidraw 图及 PNG 导出，并嵌入最终 notes。
- 2026-05-09：修正图表策略：保留 Excalidraw 作为复杂总览图，同时恢复局部 Mermaid 图用于源码调用链、ReAct loop、tool 分发、approval、sandbox 和 event 回流。
- 2026-05-09：按用户参考样式重新拆分图表：`codex-core-agent-loop-flow` 使用标准 loop 流程图，`codex-architecture-layers` 使用分层架构图，避免把 loop 和架构混在一张图里。
- 2026-05-09：新增 `notes/codex-context-and-compaction.md`，基于源码梳理 `ContextManager`、`SessionState.history`、`for_prompt`、工具结果 `ResponseInputItem` 回灌、pre/mid-turn auto compact、manual compact、`replacement_history` 和 rollout replay。
- 2026-05-10：使用 daedalus CLI 将任务状态推进到 `06-code-reader` active；`04-debugger-guide` 已完成，`05-arch-analyzer` 以 `notes/codex-agent-loop-architecture.md` 作为等价架构产物完成。
- 2026-05-10：整理 `notes/codex-context-and-compaction.md`，明确三层上下文模型、ReAct 因果链、prompt view/runtime history 分离、compact checkpoint 和 tool future 顺序保证。

## 已取消
