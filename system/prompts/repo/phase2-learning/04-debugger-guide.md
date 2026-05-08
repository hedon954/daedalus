---
title: Run And Debug Repo
description: 建立 repo 本地运行手册，并通过断点或日志走通核心链路。用于代码阅读前验证入口、依赖和架构假设。
phase: repo.phase2-learning
---

@system/prompts/common/first-principles.md
@system/prompts/common/summarize.md

# Run And Debug Repo

## Layer Contract

本 prompt 只定义 repo 的本地运行和核心链路调试。验证结论总结方式和调试目的的因果链来自上方 `@` 引用。

## Repo-Specific Trigger

- repo 已确定，准备进入代码阅读。
- 需要先证明核心链路可以在本地复现。
- 架构假设需要通过断点或日志验证。

## Repo-Specific Workflow

1. 读取 README、开发文档和 package/build 配置，确认启动方式。
2. 识别最小可运行路径：示例、测试、CLI、server endpoint 或 benchmark。
3. 记录环境变量、依赖服务、数据库、端口和常见失败点。
4. 从入口设置断点，跟踪一次核心链路。
5. 形成“入口 -> 核心模块 -> 状态变化 -> 输出”的链路笔记。

## Output Delta

```markdown
## Runbook
- 启动命令：
- 依赖：
- 最小验证方式：
- 核心入口：
- 断点建议：
- 已发现问题：
```

## Repo-Specific Constraints

- 遇到失败时，优先缩小复现，不要跳到大规模重构。
- 不要为了跑全量系统而阻塞核心链路验证。
- 所有环境依赖都要写入 runbook。
