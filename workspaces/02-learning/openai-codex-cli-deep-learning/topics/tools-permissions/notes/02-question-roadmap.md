# 问题路线图

## 当前目标

围绕 OpenAI Codex CLI 建立问题驱动的源码学习路线，服务后续运行调试、架构分析、核心代码精读和 mini demo。

## 本轮问题

1. 如果 Codex CLI 必须“在用户本机安全地完成代码任务”，它最容易失败的三个地方是什么？
2. 你猜一次 `codex exec "..."` 或交互式 prompt 的核心链路会如何流动？请先写出 5-8 个节点。
3. 如果我们要做一个 mini demo，只保留 Codex 的核心思想，必须保留哪些不变量？

## 用户回答

### 问题一：本地 coding agent 最容易失败的地方

1. 代码修改有遗漏：对需求理解不全面，对整个代码库的架构思想、架构分层和领域逻辑不够清晰和全面，导致只修改了部分代码，无法满足需求。
2. 代码存在语法错误：对代码的语法规则、语法糖和语法陷阱不够熟悉，导致代码存在语法错误，无法编译或运行。
3. 上下文丢失：长任务执行时，随着上下文逐渐变大，LLM 注意力丢失，导致任务目标偏移或遗漏。

### 问题二：一次 prompt 的核心链路猜测

1. CLI command option 处理层。
2. Coding agent 构建层：构建 master-sub agents 结构，支持 tools、MCP、skills、plugins 等扩展能力。
3. LLM client adapter 适配层：支持多种 LLM provider。
4. 进入 coding agent 的 ReAct loop 循环。
5. 分析用户意图；决定调用工具时，进入 auth 权限层判断是否具备权限。
6. 不具备权限时，pause 进入 REPL 交互等待用户确认；具备或获取许可后，进入 tool gateway 调用 tool。
7. 获取 tool 结果后继续推进：read 查询上下文、补充背景知识、edit 修改代码、bash 执行验证逻辑。
8. 最终没有 tool 调用时，总结当前工作结果，等待下一步命令。
9. 执行 hooks 等收尾操作。

### 问题三：mini demo 必须保留的不变量

1. 会话。
2. 工具体系：tool、MCP、skills、subagents。
3. 审核：权限系统。
4. 沙箱。
5. 事件流。
6. 上下文压缩机制：分层次、分粒度进行精确压缩。

## 可验证假设

- Codex 的核心失败防线应该能映射到三个方向：需求/代码库理解机制、执行验证机制、上下文管理机制。
- `codex exec` 和交互式 TUI 可能共享 `codex-core` 的会话、线程或 Agent loop，但入口层和交互层不同。
- 权限判断、审批暂停、工具网关和沙箱可能不是单一模块，而是分布在 protocol、core、exec、sandboxing、tools 等 crate/module 中。
- `hooks`、`skills`、`plugins`、MCP、subagents 是否属于同一层扩展体系需要源码验证，不能先假定它们结构一致。
- mini demo 应优先保留“会话 + 事件流 + 工具调用 + 权限审批 + 沙箱策略 + 上下文压缩”的骨架，真实 LLM provider 和完整 TUI 可以延后或 mock。

## 回答后要验证

- 失败点是否能映射到 `codex-core` 的模块边界，如上下文、工具、执行、沙箱、审批、事件流。
- 入口猜测是否能通过 `codex-rs/exec/src/main.rs`、`codex-rs/tui/src/main.rs`、`codex-rs/cli/src/main.rs` 追踪到核心路径。
- mini demo 不变量是否足以指导 `demo/design.md`，避免做成普通 CLI toy。

## 推荐下一步

进入运行调试阶段，优先选择 `codex exec` 作为第一条核心路径，因为它比 TUI 更容易复现、记录日志和追踪调用链。
