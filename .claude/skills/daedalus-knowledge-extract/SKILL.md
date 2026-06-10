---
name: daedalus-knowledge-extract
description: 从已完成或阶段闭环的 daedalus topic/project 中萃取可复用知识候选。用于用户要求沉淀知识、总结学习收获、或把 topic 产物转成 knowledge-base 条目时。
---

# Daedalus Knowledge Extract

## Contract

这是认知工作流，不是 CLI 内容生成命令。不要调用已移除的高认知 knowledge CLI 子命令。

CLI 只用于确定性支持：

```bash
daedalus knowledge template <kind> <slug> --title "<title>"
daedalus knowledge index
daedalus knowledge validate
daedalus knowledge link-check
```

## Extraction Rule

只萃取同时具备以下条件的知识：

- 现实问题入口
- 能力出口
- 第一性原理
- 机制模型
- 关键不变量
- trade-off 和不要照搬的边界
- 来源证据
- 复习或迁移练习

## Workflow

1. 读取 topic/project state、outcome map、artifact index、demo README、业务迁移记录和 closeout evidence。
2. 识别候选 concepts、skills、patterns、problems、cases、source maps、trees 和 drills。
3. 对每个候选说明：它为什么值得进入 knowledge-base。
4. 使用 `daedalus knowledge template` 创建空模板，然后由 agent 和用户基于证据填写 Markdown。
5. 运行 `daedalus knowledge index`、`validate` 和 `link-check`。

## Stop Rule

如果候选内容只是资料摘要，没有未来回忆或使用场景，就留在 workspace notes，不进入 knowledge-base。
