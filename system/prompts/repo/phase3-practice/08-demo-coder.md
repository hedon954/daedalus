---
title: Implement Repo Mini Demo
description: 按验收点实现 mini demo，并用最小验证命令证明核心能力。用于 demo 架构确定后编码、补边界和对照原 repo 时。
phase: repo.phase3-practice
---

@system/prompts/common/first-principles.md
@system/prompts/common/summarize.md

# Implement Repo Mini Demo

## Layer Contract

本 prompt 只定义 repo mini demo 的实作节奏。实现后的学习总结和可迁移结论来自 `Summarize Learning Progress`，实现解释服从 `Apply First Principles`。

## Repo-Specific Trigger

- mini demo 架构已经确定。
- 用户准备开始编码或补齐关键边界。
- 需要用测试或脚本验证 demo 的核心能力。

## Repo Implementation Rhythm

1. 建立最小工程骨架。
2. 写出核心数据结构和接口。
3. 跑通主链路的 happy path。
4. 补上关键边界：错误、并发、持久化或扩展点中最重要的一项。
5. 用测试或脚本验证 demo 的核心能力。
6. 对照原 repo，说明相同点、简化点和代价。

## Repo Step Rules

- 每次改动都要服务于一个明确验收点。
- 避免为了完整性引入额外框架。
- 保留能帮助学习的命名和模块边界。
- 实现后立刻运行最小验证。

## Output Delta

```markdown
## Implementation Step
- 本步目标：
- 改动文件：
- 验证命令：
- 学到的 repo 模式：
- 下一步：
```

## Repo-Specific Constraints

- 不要跳过验证命令。
- 不要引入与核心架构无关的框架或功能。
- 如果实现偏离 demo 设计，先更新设计再继续编码。
