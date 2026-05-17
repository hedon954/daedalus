# 06 Code Reader Guides

本目录是 `06-code-reader` 的 Agent-side 阅读入口。它保存待用户回答、待源码验证的阅读地图；不把 Agent 预读伪装成用户已掌握的 notes。

## Current Focus

- Final artifact: [`demo/design.md`](../../demo/design.md)
- Current gap: Runtime request assembly
- Current question: shell / unified exec 如何组装 `CommandRequest` 的上下文字段。
- After this: 可以定稿 demo 的 `CommandRequest` 字段边界。

## Topic Guides

| Topic | File | Status |
| --- | --- | --- |
| auth / approval / sandbox legacy guide | [`../06-code-reader-guide.md`](../06-code-reader-guide.md) | legacy |
| Runtime request assembly | [`runtime-request-assembly.md`](runtime-request-assembly.md) | active |

## Stop Rules

- 不扩展完整 TUI UI。
- 不扩展完整 MCP elicitation。
- 不扩展 Windows sandbox 细节，除非它改变 demo 的跨平台抽象。
- Agent-only 预读只能留在 `guides/06-code-reader/`，用户回答和源码证据再进入 `notes/06-code-reader/`。
