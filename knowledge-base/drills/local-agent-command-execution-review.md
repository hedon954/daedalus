---
kind = "drill"
slug = "local-agent-command-execution-review"
status = "stable"
source = "workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions"
created_at = "2026-06-11 00:00:00"
---

# Local Agent Command Execution Review Drill

## 回忆钩子

如果能在白板上讲清楚这条链路，说明 Codex 工具权限专题已经内化。

## 现实问题

安全命令执行的概念多，容易只记住名词，忘记它们为什么分层。

## 第一性原理

复习要逼迫自己重建机制，而不是重新阅读原文。

## 机制模型

```text
recall -> draw state machine -> explain trade-off -> apply to new command
```

## 关键不变量

- 先回忆，再看 notes。
- 先解释失败路径，再解释成功路径。
- 必须举一个反例说明边界。

## 取舍

这类 drill 花时间，但能发现“看懂了”和“能迁移”之间的差距。

## 不要照搬

不要把 drill 写成普通阅读清单；它必须能产生可判断的输出。

## 迁移方式

后续每个 repo-learning topic 结束后，都应至少沉淀一个类似 drill。

## 证据来源

- `workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/notes/08-demo-coder/README.md`

## 复习练习

1. 画出 command execution 状态机。
2. 说明 `ApprovalPolicy::OnRequest` 为什么不能由 sandbox failure 自动触发 retry approval。
3. 给出一个 safe-read 和一个 network-install 的不同 retry 路径。

## 评分标准

- 能解释每个状态存在的理由。
- 能指出至少两个 trade-off。
- 能迁移到一个新 command tool 设计。
