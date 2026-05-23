---
title: Run And Debug Repo
description: 建立 repo 本地运行手册，并通过断点或日志走通核心链路。用于代码阅读前验证入口、依赖和架构假设。
phase: repo.phase2-learning
---

@system/prompts/common/first-principles.md
@system/prompts/common/summarize.md
@system/prompts/common/coach-questioning.md
@system/prompts/common/diagram-guidelines.md

# Run And Debug Repo

## Layer Contract

本 prompt 只定义 repo 的本地运行和核心链路调试。验证结论总结方式和调试目的的因果链来自上方 `@` 引用。

所有 `guides/`、`notes/` 产物默认写入 active topic 目录；跨专题可复用的启动结论再同步到 project root 的 `shared/runbook.md`。

## Repo-Specific Trigger

- repo 已确定，准备进入代码阅读。
- 需要先证明核心链路可以在本地复现。
- 架构假设需要通过断点或日志验证。

## Repo-Specific Workflow

1. 读取 README、开发文档和 package/build 配置，确认启动方式。
2. 识别最小可运行路径：示例、测试、CLI、server endpoint 或 benchmark。
3. 记录环境变量、依赖服务、数据库、端口和常见失败点。
4. 将 Agent 生成的启动指南、预期输出和排障路径写入 active topic 的 `guides/04-debugger-guide/README.md`，复杂排障可拆到同目录专题文件。
5. 指导用户亲自运行最小命令，并把用户观察到的输出、错误和问题写入 active topic 的 `notes/04-debugger-guide/README.md` 或同目录 runbook 专题文件。
6. 从入口设置断点，指导用户跟踪一次核心链路。
7. 形成“入口 -> 核心模块 -> 状态变化 -> 输出”的链路笔记。
8. 外部 repo 的实际运行/调试应优先在该 repo 或其实际 workspace 根目录单独打开 Cursor 窗口；`launch.json` 示例必须以被学习 repo 的 `${workspaceFolder}` 为基准。

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

```markdown
## Role Split
- daedalus 应该做：给出启动步骤、预期现象、观察点、断点建议和排障方案。
- 用户必须亲自做：执行关键启动/调试命令，观察输出，记录自己的 runbook。
- daedalus 可以协助但不能代替：在用户明确要求时可代跑命令，但必须标注“由 Agent 执行”。

## Before Completion
- 用户亲自执行的命令：
- 用户观察到的关键输出：
- 用户遇到的问题与排障过程：
- 已验证的核心链路：
- 是否允许进入下一阶段：
```

## Repo-Specific Constraints

- 遇到失败时，优先缩小复现，不要跳到大规模重构。
- 不要为了跑全量系统而阻塞核心链路验证。
- 所有环境依赖都要写入 runbook。
- 默认不要替用户完成启动和调试；本阶段核心是让用户亲自运行、观察和形成手感。
- Agent 在等待构建或测试时，应提示用户可以观察什么、思考什么、记录什么。
- 不要假设 Cursor workspace 是 daedalus 根目录；调试配置默认以被学习 repo 的根目录为 workspace。
- 完成阶段前，在 active topic 的 `.daedalus/validation-log.md` 记录 daedalus 是否有效促进用户实践。
