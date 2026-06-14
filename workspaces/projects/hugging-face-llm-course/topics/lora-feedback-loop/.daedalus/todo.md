# Todo Path Board

Todo 是动态路径看板。学习证据变化、阶段完成、学习路径需要收窄或扩展时，都要同步更新它和 [`outcome-map.md`](outcome-map.md)。

> Agent 负责拆解、指导、排障和验收；用户负责关键实践、观察和手写笔记。不要把 Agent 自动完成的事项伪装成用户已经掌握。

## North Star

- Project：`hugging-face-llm-course`
- Topic：`lora-feedback-loop` - LoRA 微调与数据反馈闭环
- 最终产物：mini LoRA fine-tuning lab、训练闭环 runbook、实验对比报告、业务迁移笔记。
- 最小 demo：`dataset -> train -> eval -> error analysis -> feedback data -> retrain -> compare`。
- 业务迁移目标：让用户从 AI Agent 应用层进入模型训练闭环层，提升下一份 AI 开发岗位竞争力。

## Current Path

当前 active topic 已完成选题 discovery 到 task card 的初步固化，下一步是确认材料入口和最小实验边界。

## Now

- 当前问题：确认小模型、数据集任务、训练环境和材料读取策略。
- 为什么现在做它：LoRA lab 的学习价值来自完整闭环，模型/数据/环境选错会导致训练不可跑、eval 不可信或 demo 失去解释力。
- 完成后解锁：进入 `02-source-scout`，选择 Hugging Face LLM Course / PEFT / TRL / Transformers / Datasets 的最小材料集合。

## Current Cursor

Current Cursor 是恢复定位器，不是完成证明。恢复或判断阶段状态时，必须用当前代码、测试、运行输出或用户已验证观察重新校准。

- Code frontier：暂无，尚未进入训练代码或 demo 实现。
- Already wired：project/topic 已通过 daedalus lifecycle 初始化；task-card / outcome-map 已填入确认过的主线目标。
- Current open decision：选择教学小模型、数据集任务、训练环境、材料读取顺序。
- Do not suggest：不要跳过 source/material scout 直接写训练代码；不要把 DDIA 升级为第二个 active topic。

## Critical Checkpoint

Critical Checkpoint 只记录会影响后续判断的关键取舍，不写成聊天日志。

- Source constraint：Hugging Face/PEFT 示例通常默认数据、任务和评测已清楚；本 topic 必须把这些边界显式化。
- Faithful imitation：必须忠实保留训练闭环的因果链，而不是只跑一次 fine-tune。
- Simplified / improved / discarded：简化模型规模和数据规模；暂时丢弃生产级平台和完整 RLHF。
- Transfer risk：迁移到真实业务前必须重新验证数据质量、评测一致性、成本、隐私和线上回归风险。

## Gaps Blocking Next Stage

- [x] 学习目标：LoRA / fine-tuning feedback loop 能力补齐。
- [x] 最小 demo：完整训练反馈闭环 lab。
- [x] 业务迁移目标：AI Agent 工程师模型层和训练层判断力。
- [ ] 材料入口：确认课程/文档/示例组合。
- [ ] 实验边界：确认模型、数据集任务、训练环境。

## Stage Exit Criteria

- [x] 可以解释本任务为什么值得进入 active learning。
- [x] 可以说清最终产物和最小 demo。
- [x] 可以用验收标准判断是否进入 `02-source-scout` / `02-repo-scout`。
- [ ] 用户 review 本次初始化内容，确认是否接受 task card。

## Done

## Canceled
