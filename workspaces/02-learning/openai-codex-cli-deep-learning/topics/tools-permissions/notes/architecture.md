# Codex 架构分析入口

本文件是 `05-arch-analyzer` 阶段的标准入口文件。最终架构分析内容集中维护在：

- [`codex-agent-loop-architecture.md`](codex-agent-loop-architecture.md)

该笔记覆盖：

- Codex 核心 Agent loop 总览。
- 架构分层与关键 struct 关系。
- TUI/APP/Handler/Agent 的事件流。
- `RegularTask`、`run_turn`、`run_sampling_request` 的主链路。
- Tool system、auth/approval、sandbox 和 `ToolOrchestrator`。
- Excalidraw 总览图和局部 Mermaid 调用链。

保留本入口文件是为了满足 daedalus 阶段产物约定，避免把同一份架构结论复制成多份。
