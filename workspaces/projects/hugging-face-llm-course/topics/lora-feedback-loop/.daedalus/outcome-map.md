# Outcome Map

Outcome Map 是 topic 导航仪表盘，不是聊天总结。每次继续学习、深读源码、切换阶段或更新 todo 前，先用它定位：本 topic 的最终产物是什么、当前在哪、正在补哪个缺口、哪些细节停止扩展。

## North Star

- Project：`hugging-face-llm-course`
- Topic：`lora-feedback-loop` - LoRA 微调与数据反馈闭环
- 最终要获得的能力：用系统化训练闭环理解 AI fine-tuning，而不是停留在 API glue code；能判断数据、训练、评测、反馈和迁移边界。
- 最小可验证 demo：闲鱼二手买家 Agent 的 suggestion next action LoRA feedback loop lab，覆盖初始数据、训练、测试/线上样本、eval、错误分析、反馈数据、再训练和对比报告。
- 现实问题中的迁移目标：为 AI Agent 开发补齐模型层和训练层判断力，形成下一份 AI 开发岗位可展示、可解释的技术作品。

## Inherited Context

- Shared runbook：[`../../shared/runbook.md`](../../shared/runbook.md)
- Shared architecture：[`../../shared/architecture-map.md`](../../shared/architecture-map.md)
- Shared evidence：[`../../shared/evidence-registry.md`](../../shared/evidence-registry.md)

## Final Artifacts

| 产物 | 用途 | 状态 |
| --- | --- | --- |
| [`.daedalus/task-card.md`](task-card.md) | 学习目标与验收标准 | 已确认草案 |
| [`guides/01-goal-aligner/02-secondhand-buyer-agent-research.md`](../guides/01-goal-aligner/02-secondhand-buyer-agent-research.md) | 二手买家 Agent 业务痛点与 fine-tuning suitability gate | Agent 调研完成，待用户确认 |
| [`guides/02-repo-scout/02-schema-and-engineering-shape.md`](../guides/02-repo-scout/02-schema-and-engineering-shape.md) | 输入输出 schema、rubric 与 production-shaped 工程目录 | 草案完成，待用户 review |
| [`guides/02-repo-scout/03-first-lab-default-plan.md`](../guides/02-repo-scout/03-first-lab-default-plan.md) | 第一轮模型、训练环境、数据规模与材料入口默认方案 | 草案完成，待用户 review |
| [`guides/03-socratic-coach/README.md`](../guides/03-socratic-coach/README.md) | 递进问题路线图与第一批数据审核标准 | 已生成，待用户用样本校准 |
| [`guides/03-socratic-coach/01-candidate-samples-v0.1.md`](../guides/03-socratic-coach/01-candidate-samples-v0.1.md) | 第一批 20 条候选样本 | 已生成，待用户审核 |
| [`notes/04-debugger-guide/README.md`](../notes/04-debugger-guide/README.md) | 本地运行与调试证据 | 待填 |
| [`notes/05-arch-analyzer/README.md`](../notes/05-arch-analyzer/README.md) | 架构与核心边界 | 待填 |
| [`notes/06-code-reader/README.md`](../notes/06-code-reader/README.md) | 用户参与后的核心源码阅读证据 | 待填 |
| [`demo/design.md`](../demo/design.md) | mini demo 设计草案与定稿 | 待填 |
| [`demo/README.md`](../demo/README.md) | mini demo 实现、运行和验收说明 | 待填 |
| [`notes/09-biz-solver/README.md`](../notes/09-biz-solver/README.md) | 业务迁移方案 | 待填 |
| knowledge-base entry | 已验证知识归档 | 待填 |

## Current Position

Current Position 是学习地图，不是实现事实源。涉及实现阶段时，必须用当前代码、测试、运行输出和用户已验证观察校准后再判断完成度。

- 当前阶段：03-socratic-coach。
- 当前目标：等待用户审核第一批 20 条候选样本，校准 keep/revise/weak/reject 审核标准。
- 当前障碍：第一批样例数据尚未用户审核；训练代码尚未实现。
- 当前动作服务的产物：`guides/02-repo-scout/README.md`、`guides/02-repo-scout/03-first-lab-default-plan.md`、后续 `notes/03-socratic-coach/README.md`。
- 当前光标：第一批 20 条候选样本已生成；下一步用户审核至少 5-10 条。

## Contribution Back To Project

- 本 topic 完成后预计新增或更新哪些 shared evidence：
- 哪些 shared glossary / architecture-map 需要更新：
- 哪些 future topics 被发现：

## Artifact Dependency Graph

```text
task-card -> question-roadmap -> runbook -> architecture/code-reading
architecture/code-reading -> demo/design -> demo/README
demo evidence -> business-application -> knowledge-base entry
```

## Open Gaps

- [x] 确认最终要获得的能力：AI Agent 工程师的 fine-tuning / training loop 能力补齐。
- [x] 确认最小可验证 demo：闲鱼二手买家 Agent 的 suggestion next action LoRA fine-tuning feedback loop lab。
- [x] 确认业务迁移目标：提升短期 AI 岗位竞争力，同时恢复长期底层能力信心。
- [x] 确认数据集任务类型：贴近 AI Agent 的 suggestion next action 生成。
- [x] 完成 Agent 调研版 fine-tuning suitability：生产先 prompt/context/tooling，学习 topic 中用 LoRA 做对照实验。
- [x] 确认第一轮品类：手机。
- [x] 确认核心质量标准：推动购买任务往前、多帮用户想一步，并满足安全性、语义完整性和相关性。
- [x] 确认手机品类初始样例集结构、suggestion 输出 schema、购买阶段枚举和 eval rubric 草案。
- [x] 确认具体小模型、训练环境和第一批样例数据规模。
- [x] 确认材料入口：围绕 lab 反向读取 Hugging Face LLM Course + PEFT/TRL/Transformers/Datasets 官方文档。
- [x] 生成 `03-socratic-coach` 问题路线图和第一批数据审核标准。
- [x] 生成第一批 20 条候选样本并等待用户审核。
- [ ] 用户审核至少 5-10 条候选样本。

## Why This Step Matters

当前步骤决定后续应该读哪些源码、停止哪些源码、最终 demo 如何验收。

## Critical Lens

Critical Lens 用来防止把学习素材当成权威。它不是反对模仿，而是让 demo 的模仿变成有意识、有证据、有边界的学习动作。

- 当前素材中可能被过度神化的设计：Hugging Face 工具链和热门 fine-tuning recipe 可能把工具使用伪装成能力掌握。
- 当前 demo 需要忠实模仿的核心机制：面向真实 Agent 子任务的数据集构造、adapter 训练、独立 eval、错误样本反馈和再训练对比。
- 当前 demo 不应无意识照抄的设计：大规模训练工程、复杂 RLHF pipeline、榜单导向调参。
- 当前还没有验证的素材假设：小模型和小数据集能否足够展示 suggestion next action 质量的可评测变化；模糊、边界和 negative/unsafe case 是否能被 eval 稳定区分。
- 当前可以尝试简化、改进或丢弃的部分：先用小任务和轻量模型证明闭环，不追生产级平台。
- 当前迁移到业务场景前必须重新验证的约束：真实任务数据质量、评测一致性、隐私/合规、训练成本和线上回归风险。

## Stop Rules

- 不读与 North Star、demo、业务迁移或知识归档无关的源码细节。
- 不因为某个目录、函数或专题还没覆盖完就继续读。
- 不把 Agent-only 预读写成用户已经掌握的 notes。
