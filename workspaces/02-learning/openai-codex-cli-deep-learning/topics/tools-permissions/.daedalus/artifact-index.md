# 产物索引

记录 `tools-permissions` topic 的关键学习产物，以及它们服务的学习目的。

> `状态` 列只能使用 `草稿`、`已验证`、`待补充`、`不适用`。

| 产物 | 阶段 | 目的 | 状态 |
| --- | --- | --- | --- |
| [`.daedalus/task-card.md`](task-card.md) | 01-goal-aligner | 学习目标与验收标准 | 已验证 |
| [`.daedalus/outcome-map.md`](outcome-map.md) | all | 全程导航：最终产物、当前位置、demo 缺口、停止规则 | 已验证 |
| [`.daedalus/todo.md`](todo.md) | all | 动态路径看板和下一步行动 | 已验证 |
| [`.daedalus/long-context.md`](long-context.md) | all | 可恢复长期上下文 | 待补充 |
| [`notes/01-repo-selection.md`](../notes/01-repo-selection.md) | 02-repo-scout | repo 选择、风险接受和源码准备证据 | 已验证 |
| [`notes/02-question-roadmap.md`](../notes/02-question-roadmap.md) | 03-socratic-coach | 递进问题路线图和用户回答 | 已验证 |
| [`notes/03-runbook.md`](../notes/03-runbook.md) | 04-debugger-guide | 本地运行、测试和调试证据 | 已验证 |
| [`notes/04-architecture.md`](../notes/04-architecture.md) | 05-arch-analyzer | 架构阶段标准入口 | 已验证 |
| [`notes/05-codex-agent-loop-architecture.md`](../notes/05-codex-agent-loop-architecture.md) | 05-arch-analyzer | Codex agent loop、工具、权限、沙箱和事件流架构 | 已验证 |
| [`notes/06-code-reader/README.md`](../notes/06-code-reader/README.md) | 06-code-reader | code-reader 阶段 notes 入口 | 已验证 |
| [`notes/06-code-reader/01-runtime-request-assembly.md`](../notes/06-code-reader/01-runtime-request-assembly.md) | 06-code-reader | Shell / unified exec request 组装上下文 | 已验证 |
| [`notes/06-code-reader/02-orchestrator-retry.md`](../notes/06-code-reader/02-orchestrator-retry.md) | 06-code-reader | sandbox denied 后 retry / approval 状态机 | 已验证 |
| [`demo/design.md`](../demo/design.md) | 07-demo-architecture | mini demo 蓝图、状态机、事件协议和验收测试 | 已验证 |
| [`guides/08-demo-coder/README.md`](../guides/08-demo-coder/README.md) | 08-demo-coder | 08 阶段 slice 子地图 | 已验证 |
| [`demo/README.md`](../demo/README.md) | 08-demo-coder | demo 架构、运行方式、Phase 2B CLI 验收说明 | 已验证 |
| [`demo/src/`](../demo/src) | 08-demo-coder | Codex 工具/权限/沙箱 mini demo 实现 | 已验证 |
| [`demo/examples/os_execution_runner.rs`](../demo/examples/os_execution_runner.rs) | 08-demo-coder | OS sandbox runner 手动验收入口 | 已验证 |
| [`demo/examples/os_tool_runtime.rs`](../demo/examples/os_tool_runtime.rs) | 08-demo-coder | ToolRuntime + OsExecutionRunner smoke 验收入口 | 已验证 |
| [`notes/08-demo-coder/README.md`](../notes/08-demo-coder/README.md) | 08-demo-coder | demo 实现阶段 notes 索引 | 已验证 |
| [`notes/08-demo-coder/13-slice-13-codex-like-cli-closeout.md`](../notes/08-demo-coder/13-slice-13-codex-like-cli-closeout.md) | 08-demo-coder | Phase 2B CLI UI 收口、approval-test 和测试边界 | 已验证 |
| [`notes/09-biz-solver/README.md`](../notes/09-biz-solver/README.md) | 09-biz-solver | 安全本地命令执行模式的业务迁移方案 | 已验证 |
| [`knowledge-base/02-ai-engineering/local-agent-command-execution.md`](../../../../../../knowledge-base/02-ai-engineering/local-agent-command-execution.md) | 10-archivist | 可复用知识归档：本地 Agent 命令执行安全模式 | 已验证 |

## 验证记录

```bash
cargo test --manifest-path workspaces/02-learning/openai-codex-cli-deep-learning/topics/tools-permissions/demo/Cargo.toml -j 2
```

结果：

```text
84 passed; 0 failed; 3 ignored
```

TUI 验证：

```bash
cargo run --manifest-path workspaces/02-learning/openai-codex-cli-deep-learning/topics/tools-permissions/demo/Cargo.toml
```

已验证 TUI 能启动、绘制 ready 页面、通过 `q` 正常退出；事件驱动 loop 已改为 agent event / keyboard event 双 channel，避免 streaming 事件依赖键盘输入触发刷新。
