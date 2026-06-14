# 06 Code Reader Guides

本目录是 `06-code-reader` 的 Agent-side 阅读入口。它保存待用户回答、待源码验证的阅读地图；不把 Agent 预读伪装成用户已掌握的 notes。

## Stage Status

- Final artifact: [`demo/design.md`](../../demo/design.md)
- Current gap: none
- Current status: `06-code-reader` 已完成；本目录保留阶段阅读地图和历史 guide。
- After this: 相关源码结论已进入 `demo/design.md` 和后续 `08-demo-coder` 实现。

## Topic Guides

| Topic | File | Status |
| --- | --- | --- |
| auth / approval / sandbox guide | [`02-auth-approval-sandbox.md`](02-auth-approval-sandbox.md) | migrated |
| Runtime request assembly | [`01-runtime-request-assembly.md`](01-runtime-request-assembly.md) | completed |

## Stop Rules

- 不扩展完整 TUI UI。
- 不扩展完整 MCP elicitation。
- 不扩展 Windows sandbox 细节，除非它改变 demo 的跨平台抽象。
- Agent-only 预读只能留在 `guides/06-code-reader/`，用户回答和源码证据再进入 `notes/06-code-reader/`。
