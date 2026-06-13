---
kind = "drill"
slug = "agent-command-safety-review"
status = "verified"
source = "knowledge-base/problems/agent-local-command-execution.md"
created_at = "2026-06-13"
---

# Agent 本地命令安全复习练习

## 回忆钩子

能复述不算掌握；能审查一个新 Agent tool 的权限链路，才算可迁移。

## 现实问题

本 topic 的知识很容易退化成“approval + sandbox 很重要”的口号。复习练习要逼自己重新做设计判断。

## 第一性原理

复习要验证主动调用能力：不看笔记，能否从现实约束推出设计，而不是回忆结论。

## 底层原理

练习覆盖三层：

- 安全链路：capability、approval、sandbox、retry、observation。
- 执行边界：shell parsing、argv、cwd、network、filesystem。
- Agent loop：tool observation 如何回灌 messages。

## 关键不变量

- 练习问题必须有现实约束。
- 答案必须说明 trade-off。
- 答案必须说出不能照搬的边界。

## 取舍

复习练习越接近真实设计，越能暴露理解缺口；但题目太大容易拖慢节奏。每次复习只做一个小场景即可。

## 不要照搬

不要把练习变成背诵题。重点是设计判断，不是枚举名。

## 迁移方式

每次遇到新的高风险 tool，都可以套用这组 drill 做设计 review。

## 证据来源

- [Agent 本地命令执行为什么危险](../problems/agent-local-command-execution.md)
- [Agent 本地命令安全执行链路](../patterns/agent-command-safety-pipeline.md)
- [OS Sandbox 与 Shell 执行边界](../concepts/os-sandbox-and-shell-boundary.md)

## 复习练习

1. 设计一个 `run_command` tool 的 approval scope，必须包含 command、cwd、sandbox、network、persistence。
2. 解释 `NeedsApproval + approved` 为什么仍然可以 sandbox first。
3. 解释 sandbox denied 后为什么不能自动裸跑。
4. 给一个线上业务 tool，例如 `delete_customer_data`，把 shell 权限模型迁移成业务资源权限模型。
5. 画出 tool event 和 tool observation 的区别。

## 评分标准

- 能从“受控副作用”推出权限链路，而不是背结论。
- 能区分 approval、sandbox、retry 三个层级。
- 能说明至少一个生产不可照搬点。
- 能把模型迁移到非 shell 的业务 action。
