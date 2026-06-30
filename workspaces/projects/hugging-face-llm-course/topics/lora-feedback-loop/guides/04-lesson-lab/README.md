# 04 Lesson Lab：Transformers 可观察实验

本阶段把已经跑通的 Hugging Face Course 示例转成可观察实验。目标不是继续抄 recipe，而是让用户能解释每个对象在训练链路里的职责、输入输出和可验证证据。

## Current Lesson

当前 lesson：

```text
Chapter 1/5 task lab sweep
  -> repeat task labs
  -> observe input / batch / model / output shapes
  -> compare task heads and postprocess
  -> then return to Chapter 2 pipeline internals
```

## Lesson Sequence

| Lesson | Guide | 目标 |
| --- | --- | --- |
| Pipeline foundation | [`01-hf-course-ch1-ch2-foundation.md`](01-hf-course-ch1-ch2-foundation.md) | 把 `pipeline` 展开成 tokenizer、model、postprocess |
| Causal LM recipe | [`02-causal-lm-code-reading.md`](02-causal-lm-code-reading.md) | 解释 dataset -> tokenizer -> blocks -> labels |
| Trainer loop | [`03-trainer-train-under-the-hood.md`](03-trainer-train-under-the-hood.md) | 观察 batch -> forward -> loss -> backward |
| Sequence classification | [`04-sequence-classification-derivation.md`](04-sequence-classification-derivation.md) | 从任务契约反推 tokenizer、collator、model、metric、Trainer |
| Chapter 1/5 task sweep | [`05-chapter1-5-task-lab-sweep.md`](05-chapter1-5-task-lab-sweep.md) | 完成本节 task labs，用重复建立任务输入输出和 head/postprocess 直觉 |
| Token classification | [`06-token-classification-derivation.md`](06-token-classification-derivation.md) | 理解 word-level 标签如何对齐到 token-level logits |

## Required Observation

用户需要亲自运行或复查 notebook，并把观察写入 [`../../notes/04-lesson-lab/`](../../notes/04-lesson-lab/) 下的独立 note 文件；[`../../notes/04-lesson-lab/README.md`](../../notes/04-lesson-lab/README.md) 只做索引：

- chapter1/5 task sweep：每个 lab 的 input、batch keys/shape、model class、logits/generated output、postprocess、与上一个 task 的差异
- pipeline internals：tokenizer 输出、model logits、softmax scores、argmax label、`id2label` 映射
- sequence classification：raw sample keys、tokenized sample keys、collated batch shapes、`outputs.loss`、`outputs.logits.shape`
- causal LM / Trainer：`batch.keys()`、`input_ids / attention_mask / labels` 的 shape、dtype、device、shifted loss、至少一个 token 位置的 top-k prediction

## Exit Criteria

完成本阶段前，用户应能不用教程解释：

```text
不同 task 如何改变输入、head、loss 和 postprocess
Pipeline 如何展开成 tokenizer -> model -> postprocess
Dataset 提供样本
Collator 组装 batch
Model forward 计算 logits/loss
Trainer 驱动 backward、optimizer、eval、save
```

如果 lesson lab 的观察仍无法解释 hidden mechanism，再进入 `05-mechanism-deep-dive`。
