# Outcome Map

Outcome Map 是 topic 导航仪表盘，不是聊天总结。每次继续学习、深读源码、切换阶段或更新 todo 前，先用它定位：本 topic 的最终产物是什么、当前在哪、正在补哪个缺口、哪些细节停止扩展。

## North Star

- Project：`hugging-face-llm-course`
- Topic：`lora-feedback-loop` - LoRA 微调与数据反馈闭环
- 最终要获得的能力：用系统化训练闭环理解 AI fine-tuning，而不是停留在 API glue code；能判断数据、训练、评测、反馈和迁移边界。
- 最小可验证 demo：一个可运行的 LoRA feedback loop lab，覆盖初始数据、训练、测试/线上样本、eval、错误分析、反馈数据、再训练和对比报告。
- 现实问题中的迁移目标：为 AI Agent 开发补齐模型层和训练层判断力，形成下一份 AI 开发岗位可展示、可解释的技术作品。

## Inherited Context

- Shared runbook：[`../../shared/runbook.md`](../../shared/runbook.md)
- Shared architecture：[`../../shared/architecture-map.md`](../../shared/architecture-map.md)
- Shared evidence：[`../../shared/evidence-registry.md`](../../shared/evidence-registry.md)

## Final Artifacts

| 产物 | 用途 | 状态 |
| --- | --- | --- |
| [`.daedalus/task-card.md`](task-card.md) | 学习目标与验收标准 | 已确认草案 |
| [`notes/03-socratic-coach/README.md`](../notes/03-socratic-coach/README.md) | 递进问题路线图 | 待填 |
| [`notes/04-debugger-guide/README.md`](../notes/04-debugger-guide/README.md) | 本地运行与调试证据 | 待填 |
| [`notes/05-arch-analyzer/README.md`](../notes/05-arch-analyzer/README.md) | 架构与核心边界 | 待填 |
| [`notes/06-code-reader/README.md`](../notes/06-code-reader/README.md) | 用户参与后的核心源码阅读证据 | 待填 |
| [`demo/design.md`](../demo/design.md) | mini demo 设计草案与定稿 | 待填 |
| [`demo/README.md`](../demo/README.md) | mini demo 实现、运行和验收说明 | 待填 |
| [`notes/09-biz-solver/README.md`](../notes/09-biz-solver/README.md) | 业务迁移方案 | 待填 |
| knowledge-base entry | 已验证知识归档 | 待填 |

## Current Position

Current Position 是学习地图，不是实现事实源。涉及实现阶段时，必须用当前代码、测试、运行输出和用户已验证观察校准后再判断完成度。

- 当前阶段：01-goal-aligner
- 当前目标：将已确认的选题发现结果固化为 task card，并准备进入材料选择。
- 当前障碍：具体小模型、数据集任务、训练环境和材料读取顺序尚未确认。
- 当前动作服务的产物：`.daedalus/task-card.md`、`.daedalus/outcome-map.md`
- 当前光标：暂无具体源码或 demo 实现边界；以 `todo.md` 的 `Current Cursor` 为临时恢复坐标。

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
- [x] 确认最小可验证 demo：LoRA fine-tuning feedback loop lab。
- [x] 确认业务迁移目标：提升短期 AI 岗位竞争力，同时恢复长期底层能力信心。
- [ ] 确认具体小模型、数据集任务和训练环境。
- [ ] 确认材料入口：顺序学习 Hugging Face LLM Course，还是围绕 lab 反向读取官方文档。

## Why This Step Matters

当前步骤决定后续应该读哪些源码、停止哪些源码、最终 demo 如何验收。

## Critical Lens

Critical Lens 用来防止把学习素材当成权威。它不是反对模仿，而是让 demo 的模仿变成有意识、有证据、有边界的学习动作。

- 当前素材中可能被过度神化的设计：Hugging Face 工具链和热门 fine-tuning recipe 可能把工具使用伪装成能力掌握。
- 当前 demo 需要忠实模仿的核心机制：数据集构造、adapter 训练、独立 eval、错误样本反馈和再训练对比。
- 当前 demo 不应无意识照抄的设计：大规模训练工程、复杂 RLHF pipeline、榜单导向调参。
- 当前还没有验证的素材假设：小模型和小数据集能否足够展示反馈闭环的行为变化。
- 当前可以尝试简化、改进或丢弃的部分：先用小任务和轻量模型证明闭环，不追生产级平台。
- 当前迁移到业务场景前必须重新验证的约束：真实任务数据质量、评测一致性、隐私/合规、训练成本和线上回归风险。

## Stop Rules

- 不读与 North Star、demo、业务迁移或知识归档无关的源码细节。
- 不因为某个目录、函数或专题还没覆盖完就继续读。
- 不把 Agent-only 预读写成用户已经掌握的 notes。
