---
name: daedalus-knowledge-reorganize
description: 为 daedalus knowledge-base 提出结构重组方案。用于条目变得混乱、重复、难导航，或用户要求演化分类体系时。
---

# Daedalus Knowledge Reorganize

## Contract

重组必须先从 proposal 开始。没有用户明确批准，不要执行大规模移动、合并、删除或分类重写。

CLI 只用于确定性检查：

```bash
daedalus knowledge index
daedalus knowledge validate
daedalus knowledge link-check
```

## First Principles

按回忆和使用组织，而不是只按静态文件夹组织：

- problems 是问题入口
- skills 是能力
- patterns 是可复用方法
- concepts 是机制原子
- cases 是证据
- drills 是验证方式
- trees 是学习路径

## Output

输出 proposal，包含：

- 当前导航痛点
- 建议的新结构
- 需要拆分、合并、移动或归档的条目
- 预期收益
- 迁移风险
- 验证步骤
