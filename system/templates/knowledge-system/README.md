# Knowledge System Template

Knowledge System 用来把 topic/project 的已验证学习证据萃取成可迁移知识结构。它不是摘要文件夹。

## Files

- [`extraction.md`](extraction.md)：候选知识萃取入口。
- [`concept-map.md`](concept-map.md)：概念地图。
- [`invariant-map.md`](invariant-map.md)：不变量地图。
- [`failure-mode-map.md`](failure-mode-map.md)：失败模式地图。
- [`pattern-catalog.md`](pattern-catalog.md)：可迁移模式。
- [`relation-map.md`](relation-map.md)：知识关系。
- [`promotion-log.md`](promotion-log.md)：晋升记录。

## First-Principles Chain

```text
业务目标 / 现实任务
  -> 现实制约
  -> naive solution 为什么失败
  -> 核心抽象 / 不变量
  -> 实现机制
  -> trade-off
  -> 对比最佳实践
  -> 可迁移模式
  -> 复习题 / 应用题
```
