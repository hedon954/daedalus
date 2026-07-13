# Course Progress

记录课程进度、当前 checkpoint 和复习状态。这里是恢复学习现场的入口，不替代 topic todo。

## Current Checkpoint

- Active topic：`lora-feedback-loop`
- 当前课程章节：Hugging Face LLM Course Chapter 3/2 `Processing the data`
- 当前 lesson / mechanism：Chapter 3 全章 guide 已完成；当前只推进 3/2，观察 raw dataset -> paired tokenization -> `Dataset.map` -> dynamic padding -> training batch
- 当前 open question：`Dataset.map(batched=True)` 的 preprocessing batch 与 DataLoader/collator 生成的训练 batch 有什么本质区别
- 下一步：进入 Chapter 3/2 `Processing the data`，先预测并打印 raw/tokenized/collated 三层 keys、length、shape 与 label mapping

## Progress Board

| Chapter / Section | Status | Evidence | Review |
| --- | --- | --- | --- |
| Chapter 1/5 How Transformers solve tasks | done-with-scope-cut | 5/8 labs complete：Causal LM / sequence classification / token classification / question answering / summarization；Translation / ASR / Image classification 按用户决定跳过或延后 | 足够进入架构归纳 |
| Chapter 1/6 Transformer Architectures | done-enough-to-advance | 架构家族已归纳；attention mechanisms 已完成 Agent-guided walkthrough，LSH/local/axial 可回看 guide | 若后续长上下文机制薄弱再回看 |
| Chapter 1/7 Ungraded quiz | passed-through | 用户已进入 Chapter 1/8 | 不单独归档 quiz 答案 |
| Chapter 1/8 Deep dive into Text Generation Inference with LLMs | done-by-user-confirmation | 已有 inference 主线 guide；用户确认 Chapter 1 已完成并已越过 Chapter 2 | 机制弱点按需回看 |
| Chapter 2 Using Transformers | done-by-user-confirmation | 已有 tokenizer、model、pipeline 与 forward 观察；用户于 2026-07-13 确认完成整章 | 深挖项不再阻塞前进 |
| Chapter 3/1 Introduction | done | 官方页与本章路线已确认；全章 guide 已生成 | 进入 3/2 |
| Chapter 3/2 Processing the data | active | guide 已给出预测、最小观察代码与通过标准 | 当前 lesson |

## Review Queue

- Chapter 1/8 inference 与 Chapter 2 的深挖项保留为按需复习，不再作为 Chapter 3 的前置阻塞。
- Chapter 3：对比 Trainer API 与 full training loop；进入数据处理时观察 dataset -> tokenize -> dynamic padding -> batch，进入训练时补 batch keys / shape / device、loss/logits、optimizer/scheduler 与 evaluation 证据。
