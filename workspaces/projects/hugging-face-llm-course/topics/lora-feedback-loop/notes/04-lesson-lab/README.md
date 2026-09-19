# 04 Lesson Lab：运行证据索引

本 README 只做阶段索引和恢复入口；具体运行证据、用户复述、校正点和机制说明放在同目录独立 note 文件中。

## 当前入口

- 课程阅读：用户于 2026-08-27 确认 Hugging Face LLM Course 全部阅读完毕。
- Lesson lab：已有运行证据保留在本索引；阅读完成不自动补写缺失实验或用户复述。
- 当前学习入口：转入 [`../../guides/08-review-loop/README.md`](../../guides/08-review-loop/README.md)，通过 SFT 博文写作检验机制、实验和迁移掌握度。
- 真实业务实践：Qwen3 1.7B、约 27k、Full SFT 与 LoRA 的已确认事实见 [`../../guides/06-practice-transfer/05-qwen3-suggestion-sft-practice-evidence.md`](../../guides/06-practice-transfer/05-qwen3-suggestion-sft-practice-evidence.md)。

## Notes

| Date | Note | 用途 |
| --- | --- | --- |
| 2026-06-16 | [`Transformers Pipeline Smoke Test`](2026-06-16-transformers-pipeline-smoke-test.md) | 验证最小 pipeline 运行与静态类型 alias 差异。 |
| 2026-06-22 | [`ELI5 Dataset Inspection`](2026-06-22-eli5-dataset-inspection.md) | 观察 Causal LM 原始数据结构。 |
| 2026-06-22 | [`ELI5 Tokenization Inspection`](2026-06-22-eli5-tokenization-inspection.md) | 观察 tokenizer 输出和长序列 warning。 |
| 2026-06-22 | [`LM Block Inspection`](2026-06-22-lm-block-inspection.md) | 观察 group_texts、block size 和 labels。 |
| 2026-06-23 | [`BPE vs WordPiece`](bpe-vs-wordpiece.md) | 独立 tokenizer 机制笔记。 |
| 2026-06-23 | [`Data Collator / Trainer Warning Inspection`](2026-06-23-data-collator-trainer-warning-inspection.md) | 解释 special token 与 MPS pin_memory warning。 |
| 2026-06-23 | [`MPS Training Speed Inspection`](2026-06-23-mps-training-speed-inspection.md) | 判断训练慢的原因和缩小实验策略。 |
| 2026-06-23 | [`Causal LM Training Completion`](2026-06-23-causal-lm-training-completion.md) | 记录 DistilGPT2 quick training、perplexity 和 Hub 上传。 |
| 2026-06-26 | [`Sequence Classification Lab Completion`](2026-06-26-sequence-classification-lab-completion.md) | 记录 IMDb sequence classification lab 完成与下一步 derivation review。 |
| 2026-06-26 | [`Sequence Classification Pipeline Load Error`](2026-06-26-sequence-classification-pipeline-load-error.md) | 记录本地 pipeline 加载空模型目录的诊断。 |
| 2026-06-27 | [`Sequence Classification Derivation Review`](2026-06-27-sequence-classification-derivation-review.md) | 记录用户复述、forward/logits probe 和闭卷重写验收。 |
| 2026-06-27 | [`Chapter 1/5 Task Lab Sweep 路线调整`](2026-06-27-chapter1-5-task-lab-sweep-route.md) | 记录先完成 8 个 task labs 的路线判断。 |
| 2026-06-30 | [`Token Classification Lab 复述`](2026-06-30-token-classification-lab-recap.md) | 记录用户对 token classification 的复述、校正点和完成证据。 |
| 2026-07-03 | [`Summarization Lab Recap`](2026-07-03-summarization-lab-recap.md) | 记录 BillSum summarization 训练、ROUGE 排障、checkpoint 推理和 seq2seq 观察。 |
| 2026-07-07 | [`Chapter 1/6 Encoder-only 复述`](2026-07-07-chapter1-6-encoder-only-recap.md) | 记录用户对 encoder-only 任务形状的复述与下一步 decoder-only 检查问题。 |
| 2026-07-07 | [`Chapter 1/6 Decoder-only 复述`](2026-07-07-chapter1-6-decoder-only-recap.md) | 记录用户对 decoder-only 续写任务形状的复述与下一步 encoder-decoder 检查问题。 |
| 2026-07-07 | [`Chapter 1/6 Encoder-decoder 复述`](2026-07-07-chapter1-6-encoder-decoder-recap.md) | 记录用户对 summarization 架构选择的复述，以及“能做”和“更适合”的边界。 |
| 2026-07-07 | [`Chapter 1/6 架构选择总结`](2026-07-07-chapter1-6-architecture-selection-summary.md) | 记录用户用任务形态反推 encoder-only、decoder-only、encoder-decoder 的最终归纳。 |
