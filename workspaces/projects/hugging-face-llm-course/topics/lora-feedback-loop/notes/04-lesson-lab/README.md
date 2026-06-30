# 04 Lesson Lab：运行证据索引

本 README 只做阶段索引和恢复入口；具体运行证据、用户复述、校正点和机制说明放在同目录独立 note 文件中。

## 当前入口

- 当前 lab：Chapter 1/5 Question Answering。
- 最近完成：Token Classification，见 [`2026-06-30-token-classification-lab-recap.md`](2026-06-30-token-classification-lab-recap.md)。
- 当前 guide：等待建立 Question Answering 单项 guide；Chapter 1/5 总览见 [`../../guides/04-lesson-lab/05-chapter1-5-task-lab-sweep.md`](../../guides/04-lesson-lab/05-chapter1-5-task-lab-sweep.md)。

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
