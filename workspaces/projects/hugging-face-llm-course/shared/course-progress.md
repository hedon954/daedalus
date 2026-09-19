# Course Progress

记录课程进度、当前 checkpoint 和复习状态。这里是恢复学习现场的入口，不替代 topic todo。

## Current Checkpoint

- Active topic：`lora-feedback-loop`
- 课程阅读：用户于 2026-08-27 确认 Hugging Face LLM Course 当前 0–12 章全部阅读完毕。
- 当前阶段：从 lesson reading 转入 `08-review-loop`，通过博文检验 SFT 的解释、复现、改写与迁移能力。
- 当前 review：先写《SFT 到底学了什么：从 next-token loss 到行为蒸馏》的提纲、机制图和最小伪代码。
- 真实业务迁移：用户确认用 Qwen3 1.7B、约 27k 数据分别执行过 Full SFT 与 LoRA。
- 当前 open evidence：两种训练的精确配置、评测结果、最佳 checkpoint、数据 split 和反馈再训练尚未脱敏归档。

## Reading Board

| Course Part | Chapters | Status | Evidence Boundary |
| --- | --- | --- | --- |
| 环境与 Transformers 基础 | 0–4 | read-complete-by-user-confirmation | Chapter 1/2 有大量 notebook、复述与运行证据；Chapter 3/4 部分实践待 review |
| Datasets、Tokenizers 与经典任务 | 5–8 | read-complete-by-user-confirmation | Chapter 5/2 Agent loading 证据、BPE/WordPiece 笔记、五类文本 task labs；其余内容不自动算实践完成 |
| Demo 与数据策展 | 9–10 | read-complete-by-user-confirmation | 阅读确认；Gradio/Argilla 实践不是当前 SFT review 的阻塞项 |
| LLM SFT / LoRA | 11 | read-complete-by-user-confirmation + real-work-transfer | Qwen3 1.7B / 27k Full SFT 与 LoRA 为用户确认事实；实验结果待归档 |
| Reasoning / GRPO | 12 | read-complete-by-user-confirmation | 尚无运行或闭卷解释证据，以博文 8 验收 |

## Existing Verified Learning Evidence

| Evidence | Status | What It Proves |
| --- | --- | --- |
| DistilGPT2 Causal LM quick training | user-run | `dataset -> tokenizer -> blocks -> labels -> Trainer` 能运行；记录 `training_loss≈3.98`、`perplexity≈47.81` 与 Hub 上传 |
| Sequence classification | user-run + closed-book rewrite | 能从任务契约反推 tokenizer、collator、head、loss、logits 与 `id2label` |
| Token classification | user-run + recap | 能解释 word label 到 token logits 的 alignment |
| Question answering | user-run + debug | 能解释字符 answer span、`offset_mapping`、start/end labels 与合法 span |
| Summarization | user-run + recap | 能解释 T5 prefix、seq2seq labels、ROUGE、checkpoint 与 `generate` |
| Qwen3 suggestion model | user-confirmed real work | 已把 SFT/LoRA 迁移到真实 Buyer Agent；公开结论仍需脱敏实验报告支撑 |

## Review Queue

- 用博文 1 验收 SFT / causal LM / teacher forcing / loss mask，而不是重新做入门训练。
- 用博文 2 验收 chat template、tokenizer、truncation 和真实训练样本 preflight。
- 用博文 3 重建 Qwen3 1.7B Full-vs-LoRA 的公平实验矩阵；不混入未确认的 0.6B 历史实验。
- 用博文 4–5 验收 27k 数据的 policy/coverage/split 与 eval protocol。
- 只有补齐反馈数据再训练证据后，博文 6 才能声称完成 feedback loop。
- Chapter 12 的 GRPO 目前仅为阅读完成；用博文 8 判断是否真正掌握方法边界。
