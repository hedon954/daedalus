# Todo Path Board

Todo 是动态路径看板。学习证据变化、阶段完成、学习路径需要收窄或扩展时，都要同步更新它和 [`outcome-map.md`](outcome-map.md)。

> Agent 负责拆解、指导、排障和验收；用户负责关键实践、观察和手写笔记。不要把 Agent 自动完成的事项伪装成用户已经掌握。

## North Star

- Project：`hugging-face-llm-course`
- Topic：`lora-feedback-loop` - LoRA 微调与数据反馈闭环
- 最终产物：闲鱼二手买家 Agent suggestion next action production-shaped mini LoRA lab、训练闭环 runbook、实验对比报告、业务迁移笔记。
- 最小 demo：`suggestion dataset -> train -> eval -> error analysis -> feedback data -> retrain -> compare`。
- 业务迁移目标：让用户从 AI Agent 应用层进入模型训练闭环层，提升下一份 AI 开发岗位竞争力。

## Current Path

当前 active topic 已完成 01-goal-aligner：业务任务、fine-tuning suitability、第一轮品类和核心质量标准已经确认。下一步进入材料选择。

## Now

- 当前问题：已进入 `03-socratic-coach`，问题路线图、数据审核标准和第一批 20 条候选样本已创建；下一步需要用户按 keep/revise/weak/reject 审核样本，校准训练数据质量口径。
- 为什么现在做它：用户明确表示自己是这方面小白；没有基础判断地图时，训练环境和模型选择会变成蒙选。现在已确认优先 pipeline，因此材料选择可以收窄。
- 完成后解锁：进入问题路线图，定义输入输出 schema、rubric、baseline 和第一轮数据集。

## Current Cursor

Current Cursor 是恢复定位器，不是完成证明。恢复或判断阶段状态时，必须用当前代码、测试、运行输出或用户已验证观察重新校准。

- Code frontier：暂无，尚未进入训练代码或 demo 实现。
- Already wired：project/topic 已通过 daedalus lifecycle 初始化；task-card / outcome-map 已填入确认过的主线目标。
- Current open decision：第一批候选数据已生成 20 条，不一次性扩展 120 条；用户用 keep/revise/weak/reject 校准审核标准后，再批量扩展。
- Do not suggest：不要跳过 source/material scout 直接写训练代码；不要把 DDIA 升级为第二个 active topic。

## Critical Checkpoint

Critical Checkpoint 只记录会影响后续判断的关键取舍，不写成聊天日志。

- Source constraint：Hugging Face/PEFT 示例通常默认数据、任务和评测已清楚；本 topic 必须把闲鱼买家 Agent 的输入、输出、偏好标准和评测边界显式化。
- Faithful imitation：必须忠实保留训练闭环的因果链，而不是只跑一次 fine-tune。
- Engineering standard：第一阶段可以低成本、小数据、小模型，但不能用 demo 标准放过工程结构；训练、评测、反馈构造和报告生成必须脚本化、配置化、产物化。
- Dataset difficulty：数据集不能只覆盖典型清晰 case；必须加入 ambiguous、boundary、negative/unsafe 样本，让模型学会补证据、暂停推进和处理风险冲突。
- Simplified / improved / discarded：简化模型规模和数据规模；暂时丢弃生产级平台和完整 RLHF。
- Transfer risk：迁移到真实业务前必须重新验证数据质量、评测一致性、成本、隐私和线上回归风险。

## Gaps Blocking Next Stage

- [x] 学习目标：LoRA / fine-tuning feedback loop 能力补齐。
- [x] 最小 demo：闲鱼二手买家 Agent suggestion next action 训练反馈闭环 lab。
- [x] 业务迁移目标：AI Agent 工程师模型层和训练层判断力。
- [x] 数据集任务：贴近 AI Agent 的 suggestion next action 生成。
- [x] Fine-tuning suitability：已形成 Agent 调研版判断，结论是先做 prompt/context baseline，再用 LoRA 判断稳定策略和格式偏好是否改善。
- [x] 第一轮品类：手机。
- [x] 核心质量标准：推动购买任务往前、多帮用户想一步，同时满足安全性、语义完整性和相关性。
- [x] 材料入口：确认课程/文档/示例组合。
- [x] 实验优先级：第一阶段先证明完整 pipeline。
- [x] 训练环境约束：可接受 Colab / 云 GPU，但需要成本可控。
- [x] 数据策略：Agent 生成候选样本，用户审核修改。
- [x] 企业平台候选：阿里云百炼可作为第二阶段迁移目标。
- [x] 工程标准：从第一阶段开始采用 script-first / config-first / artifact-first，不用 demo 标准放过自己。
- [x] Schema 草案：买家上下文、suggestion 输出结构、购买阶段枚举、rubric 和工程目录已形成草案。
- [x] 实验边界：默认模型、训练环境和第一批样例数据规模已确认。
- [x] 下一阶段入口：生成 `03-socratic-coach` 问题路线图、数据生成标准和 demo 设计前置问题。
- [x] 第一批样本：生成 20 条候选数据，供用户校准审核口径。
- [x] 难度覆盖：确认数据集需要覆盖 typical / ambiguous / boundary / negative-unsafe case。
- [ ] 用户审核：用户至少审核 5-10 条样本，校准 keep/revise/weak/reject 口径。

## Stage Exit Criteria

- [x] 可以解释本任务为什么值得进入 active learning。
- [x] 可以说清最终产物和最小 demo。
- [x] 可以用验收标准判断是否进入 `02-source-scout` / `02-repo-scout`。
- [x] 用户 review 本次初始化内容，并确认 fine-tuning suitability、第一轮品类和质量标准。

## Done

- [x] 01-goal-aligner：完成 topic 目标、业务任务、fine-tuning suitability、第一轮品类和核心质量标准确认。
- [x] 02-repo-scout：完成材料入口、schema、工程形态、默认模型、训练环境和数据规模收敛。
- [x] 03-socratic-coach：创建问题路线图和第一批数据审核标准。
- [x] 第一批候选样本：生成 20 条手机品类 suggestion next action 候选样本。

## Canceled
