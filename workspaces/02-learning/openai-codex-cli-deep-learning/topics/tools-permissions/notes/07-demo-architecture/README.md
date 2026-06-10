# 07 Demo Architecture Notes

本目录是 `07-demo-architecture` 阶段的用户学习证据入口。阶段的正式设计产物是 [`../../demo/design.md`](../../demo/design.md)；本目录保存从源码学习过渡到 demo 设计的方法和决策笔记。

## Learning Navigation

- Final artifact: [`../../demo/design.md`](../../demo/design.md)
- Current stage: `07-demo-architecture`
- Current status: 已完成，作为 `08-demo-coder` 的设计输入
- After this: 进入 demo slice 实现，并在实现过程中持续校准设计

## Topic Index

| Topic | File | Status | Demo impact |
| --- | --- | --- | --- |
| 从源码学习到可迁移设计的方法 | [`01-design-method-from-source-to-demo.md`](01-design-method-from-source-to-demo.md) | 已验证 | 决定从现实问题、系统不变量、状态机和验收用例反推 demo 结构 |

## Stage Exit Criteria

- [x] `demo/design.md` 已从源码阅读缺口收敛为可实现蓝图。
- [x] Phase 1 / Phase 2 的 demo 边界已明确。
- [x] 核心状态机、事件协议和验收用例足以指导 `08-demo-coder`。
