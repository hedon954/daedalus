# Outcome Map

Outcome Map 是 topic 导航仪表盘，不是聊天总结。每次继续学习、深读源码、切换阶段或更新 todo 前，先用它定位：本 topic 的最终产物是什么、当前在哪、正在补哪个缺口、哪些细节停止扩展。

## North Star

- Project：`hugging-face-llm-course`
- Topic：`lora-feedback-loop` - LoRA 微调与数据反馈闭环
- 最终要获得的能力：用系统化训练闭环理解 AI fine-tuning，而不是停留在 API glue code；能判断数据、训练、评测、反馈和迁移边界。
- 最小可验证成果：用博文重建 SFT 机制，并用已完成的 Qwen3 suggestion Full/LoRA 实践检验数据、训练、评测和反馈边界。
- 现实问题中的迁移目标：为 AI Agent 开发补齐模型层和训练层判断力，形成下一份 AI 开发岗位可展示、可解释的技术作品。

## Inherited Context

- Syllabus map：[`../../shared/syllabus-map.md`](../../shared/syllabus-map.md)
- Course progress：[`../../shared/course-progress.md`](../../shared/course-progress.md)
- Concept map：[`../../shared/concept-map.md`](../../shared/concept-map.md)
- Shared evidence：[`../../shared/evidence-registry.md`](../../shared/evidence-registry.md)

## Final Artifacts

| 产物 | 用途 | 状态 |
| --- | --- | --- |
| [`.daedalus/task-card.md`](task-card.md) | 学习目标与验收标准 | 已确认草案 |
| [`guides/01-need-aligner/02-secondhand-buyer-agent-research.md`](../guides/01-need-aligner/02-secondhand-buyer-agent-research.md) | 二手买家 Agent 业务痛点与 fine-tuning suitability gate | Agent 调研完成，待用户确认 |
| [`guides/02-syllabus-mapper/02-schema-and-engineering-shape.md`](../guides/02-syllabus-mapper/02-schema-and-engineering-shape.md) | 输入输出 schema、rubric 与 production-shaped 工程目录 | 草案完成，待用户 review |
| [`guides/02-syllabus-mapper/03-first-lab-default-plan.md`](../guides/02-syllabus-mapper/03-first-lab-default-plan.md) | 第一轮模型、训练环境、数据规模与材料入口默认方案 | 草案完成，待用户 review |
| [`guides/06-practice-transfer/README.md`](../guides/06-practice-transfer/README.md) | 递进问题路线图与第一批数据审核标准 | 已生成，待用户用样本校准 |
| [`guides/06-practice-transfer/01-candidate-samples-v0.1.md`](../guides/06-practice-transfer/01-candidate-samples-v0.1.md) | 第一批 20 条候选样本 | 已生成，待用户审核 |
| [`guides/06-practice-transfer/02-candidate-samples-v0.2.md`](../guides/06-practice-transfer/02-candidate-samples-v0.2.md) | 第一批 20 条短句数组候选样本 | 已生成，待用户审核 |
| [`guides/06-practice-transfer/03-candidate-samples-v0.3.md`](../guides/06-practice-transfer/03-candidate-samples-v0.3.md) | `{s,r}` schema 候选样本 | 已生成，待用户审核 |
| [`guides/06-practice-transfer/04-candidate-samples-v0.4.md`](../guides/06-practice-transfer/04-candidate-samples-v0.4.md) | 用户口吻 Agent 指令样本 | 已生成，待用户审核 |
| [`guides/03-concept-roadmap/README.md`](../guides/03-concept-roadmap/README.md) | HF Course 1/2 概念路线图 | 已生成，当前执行 |
| [`guides/04-lesson-lab/README.md`](../guides/04-lesson-lab/README.md) | HF lesson guide 索引 | 已索引化，当前执行 |
| [`guides/04-lesson-lab/17-chapter3-finetuning-pretrained-model.md`](../guides/04-lesson-lab/17-chapter3-finetuning-pretrained-model.md) | Chapter 3 数据处理、Trainer、full loop、Accelerate、learning curves 与 LoRA 迁移 guide | Chapter 3 已读完；保留未验收运行证据 |
| [`guides/04-lesson-lab/18-chapter5-2-local-remote-dataset-loading.md`](../guides/04-lesson-lab/18-chapter5-2-local-remote-dataset-loading.md) | Chapter 5/2 本地/远程文件加载与 URL 探测排障 guide | Agent 已验证三种来源；待用户观察并复述数据契约 |
| [`guides/04-lesson-lab/06-token-classification-derivation.md`](../guides/04-lesson-lab/06-token-classification-derivation.md) | Token classification 任务契约与 label alignment guide | 已完成 |
| [`guides/04-lesson-lab/07-question-answering-derivation.md`](../guides/04-lesson-lab/07-question-answering-derivation.md) | Question answering 字符答案到 token span 的任务契约与排障 guide | 已完成基础观察 |
| [`guides/04-lesson-lab/08-summarization-derivation.md`](../guides/04-lesson-lab/08-summarization-derivation.md) | Summarization encoder-decoder 任务形状 guide | 已完成 |
| [`guides/04-lesson-lab/09-t5-task-prefix.md`](../guides/04-lesson-lab/09-t5-task-prefix.md) | T5 `summarize:` task prefix guide | 已完成 |
| [`guides/04-lesson-lab/10-billsum-dataset-loading.md`](../guides/04-lesson-lab/10-billsum-dataset-loading.md) | BillSum dataset / split / notebook 变量排障 guide | 已完成 |
| [`guides/04-lesson-lab/11-rouge-metric.md`](../guides/04-lesson-lab/11-rouge-metric.md) | ROUGE metric guide | 已完成 |
| [`guides/04-lesson-lab/12-mixed-precision-fp16-bf16.md`](../guides/04-lesson-lab/12-mixed-precision-fp16-bf16.md) | Mixed precision guide | 已完成 |
| [`guides/04-lesson-lab/13-summarization-compute-metrics-overflow.md`](../guides/04-lesson-lab/13-summarization-compute-metrics-overflow.md) | Summarization metric decode overflow debug guide | 已完成 |
| [`guides/04-lesson-lab/14-summarization-inference-generate.md`](../guides/04-lesson-lab/14-summarization-inference-generate.md) | Summarization inference / `generate` guide | 已完成 |
| [`notes/04-lesson-lab/README.md`](../notes/04-lesson-lab/README.md) | Lesson lab notes 索引 | 已索引化 |
| [`notes/04-lesson-lab/2026-06-30-token-classification-lab-recap.md`](../notes/04-lesson-lab/2026-06-30-token-classification-lab-recap.md) | Token classification 用户复述与完成证据 | 已完成 |
| [`notes/04-lesson-lab/2026-07-03-summarization-lab-recap.md`](../notes/04-lesson-lab/2026-07-03-summarization-lab-recap.md) | Summarization notebook 运行、排障和推理证据 | 已完成 |
| [`guides/06-practice-transfer/05-qwen3-suggestion-sft-practice-evidence.md`](../guides/06-practice-transfer/05-qwen3-suggestion-sft-practice-evidence.md) | 真实 Qwen3 1.7B / 27k Full 与 LoRA 的已确认事实和证据边界 | 已归档事实，实验细节待补 |
| [`guides/08-review-loop/01-sft-blog-series-mastery-plan.md`](../guides/08-review-loop/01-sft-blog-series-mastery-plan.md) | 八篇 SFT 博文的论点、证据和掌握度门槛 | 已生成，当前执行 |
| [`review/mastery-map.md`](../review/mastery-map.md) | 解释、运行、改写、迁移四级掌握度地图 | 已生成，待用户写作验收 |
| [`review/question-bank.md`](../review/question-bank.md) | 以开放题和最小复现检验掌握度 | 已生成，待逐篇回答 |
| [`demo/design.md`](../demo/design.md) | mini demo 设计草案与定稿 | 待填 |
| [`demo/README.md`](../demo/README.md) | mini demo 实现、运行和验收说明 | 待填 |
| [`guides/06-practice-transfer/README.md`](../guides/06-practice-transfer/README.md) | 业务迁移练习 | 待填 |
| knowledge-base entry | 已验证知识归档 | 待填 |

## Current Position

Current Position 是学习地图，不是实现事实源。涉及实现阶段时，必须用当前代码、测试、运行输出和用户已验证观察校准后再判断完成度。

- 当前阶段：`08-review-loop`；课程 0–12 章已全部阅读完毕（用户于 2026-08-27 确认）。
- 当前生命周期：`parked`。用户于 2026-09-19 中途搁置，先释放 active 槽。
- 接回命令：`daedalus topic activate lora-feedback-loop`
- 当前目标：用博文证明能独立解释、复现、改写和迁移 SFT，而不是继续累积阅读量。
- 当前真实实践：Qwen3 1.7B、约 27k suggestion 数据、Full SFT 与 LoRA 均已实际执行。
- 当前证据缺口：两种训练的公平对照、精确配置与结果、真实数据画像、eval protocol 和 feedback retrain 尚未脱敏归档。
- 当前 checkpoint：博文 1《SFT 到底学了什么：从 next-token loss 到行为蒸馏》。

## Contribution Back To Project

- 本 topic 完成后预计新增或更新哪些 shared evidence：
- 哪些 shared glossary / architecture-map 需要更新：
- 哪些 future topics 被发现：

## Artifact Dependency Graph

```text
task-card -> syllabus-map -> concept-roadmap -> lesson-lab
lesson-lab -> mechanism-deep-dive -> practice-transfer
practice-transfer -> capstone/review -> knowledge-base entry
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
- [x] 生成 `06-practice-transfer` 问题路线图和第一批数据审核标准。
- [x] 生成第一批 20 条候选样本并等待用户审核。
- [x] 跑通第一个 Hugging Face Transformers pipeline smoke test。
- [x] 跑通 Causal LM quick training run。
- [x] 跑通 sequence classification lab。
- [x] 跑通 sequence classification pipeline load / inference。
- [x] 完成 sequence classification derivation review。
- [x] 完成 sequence classification closed-book rewrite。
- [x] 完成 Chapter 1/5 task lab sweep 的文本主线；剩余 Translation / ASR / Image classification 已按用户决定跳过/延后，不再阻塞 Chapter 1/6。
- [x] Lab 1/8 Text generation / Causal LM。
- [x] Lab 2/8 Text classification。
- [x] Lab 3/8 Token classification。
- [x] Lab 4/8 Question answering。
- [x] Lab 5/8 Summarization。
- [x] Lab 6/8 Translation：跳过/延后。
- [x] Lab 7/8 Automatic speech recognition：跳过/延后。
- [x] Lab 8/8 Image classification：跳过/延后。
- [x] Chapter 1/6 Transformer Architectures：已完成 encoder-only、decoder-only、encoder-decoder 的任务适配归纳；attention mechanisms 已完成到可推进。
- [x] Chapter 1/7 Ungraded quiz：用户已越过并进入 Chapter 1/8。
- [x] Chapter 1/8 Deep dive into Text Generation Inference with LLMs：用户确认 Chapter 1 已完成。
- [x] 完成 HF Course 第 1、2 章 foundation sprint：用户于 2026-07-13 确认完成 Chapter 2。
- [x] Chapter 3/1 Introduction：全章路线与产出预期已建立。
- [x] Chapter 3/2 Processing the data：用户确认已读完；实验观察仍在待验收队列。
- [x] Chapter 3/3 Fine-tuning with Trainer API：用户确认已读完；运行产物与指标仍在待验收队列。
- [x] Chapter 3/4 A full training loop：用户已完成机制学习；完整训练运行证据按需补齐。
- [x] Chapter 3/5-3/7：用户确认已读完；learning curves 观察与单变量诊断实验仍待验收。
- [x] Chapter 4 Sharing models and tokenizers：用户确认已读完；Hub 上传与 model card 实践仍待验收。
- [x] Chapter 5/1 Introduction：用户确认已读完。
- [x] Hugging Face LLM Course 0–12：用户于 2026-08-27 确认全部阅读完毕；不等同于全部掌握。
- [x] 真实业务迁移：用户确认 Qwen3 1.7B、约 27k、Full SFT 与 LoRA 均已训练。
- [ ] 博文 1：闭卷解释 SFT objective、causal shift、teacher forcing、loss mask 和 behavioral distillation。
- [ ] 博文 2：用真实样本证明 chat template、tokenizer、truncation、EOS 与 labels 正确。
- [ ] 博文 3：补齐 Qwen3 1.7B Full-vs-LoRA 的公平对照矩阵、结果和竞争性解释。
- [ ] 博文 4–5：补齐 27k 数据画像、session split、action policy 与 eval protocol。
- [ ] 博文 6：确认是否完成 feedback data retrain；没有第二轮就不能声称形成反馈闭环。

## Why This Step Matters

当前步骤决定后续应该学哪些章节、停止哪些材料、最终 lesson lab 和迁移任务如何验收。

## Critical Lens

Critical Lens 用来防止把学习素材当成权威。它不是反对模仿，而是让 demo 的模仿变成有意识、有证据、有边界的学习动作。

- 当前素材中可能被过度神化的设计：Hugging Face 工具链和热门 fine-tuning recipe 可能把工具使用伪装成能力掌握。
- 当前 demo 需要忠实模仿的核心机制：面向真实 Agent 子任务的数据集构造、adapter 训练、独立 eval、错误样本反馈和再训练对比。
- 当前 demo 不应无意识照抄的设计：大规模训练工程、复杂 RLHF pipeline、榜单导向调参。
- 当前还没有验证的素材假设：Qwen3 1.7B Full/LoRA 的结果差异能否归因于更新方式；27k 数据是否覆盖 decision boundary；反馈数据是否带来独立 held-out 改善。
- 当前可以尝试简化、改进或丢弃的部分：先用小任务和轻量模型证明闭环，不追生产级平台。
- 当前迁移到业务场景前必须重新验证的约束：真实任务数据质量、评测一致性、隐私/合规、训练成本和线上回归风险。

## Stop Rules

- 不学与 North Star、lesson lab、迁移任务或知识归档无关的课程细节。
- 不再因为课程目录推进阅读；只有写作暴露具体弱点时才回补对应章节或 mechanism。
- 不把 Agent-only 预读写成用户已经掌握的 notes。
- 不把未确认的 0.6B 历史分数并入 1.7B / 27k 实验，也不把助手推断写成实验结论。
