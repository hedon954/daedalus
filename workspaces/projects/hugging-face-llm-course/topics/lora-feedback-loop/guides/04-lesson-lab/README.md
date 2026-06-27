# 04 Lesson Lab：Transformers 可观察实验

本阶段把已经跑通的 Hugging Face Course 示例转成可观察实验。目标不是继续抄 recipe，而是让用户能解释每个对象在训练链路里的职责、输入输出和可验证证据。

## Current Lesson

当前 lesson：

```text
Sequence classification derivation review
  -> raw text / label
  -> tokenized sample
  -> dynamically padded batch
  -> sequence classification logits / loss
  -> metric / inference label
```

## Lesson Sequence

| Lesson | Guide | 目标 |
| --- | --- | --- |
| Pipeline foundation | [`01-hf-course-ch1-ch2-foundation.md`](01-hf-course-ch1-ch2-foundation.md) | 把 `pipeline` 展开成 tokenizer、model、postprocess |
| Causal LM recipe | [`02-causal-lm-code-reading.md`](02-causal-lm-code-reading.md) | 解释 dataset -> tokenizer -> blocks -> labels |
| Trainer loop | [`03-trainer-train-under-the-hood.md`](03-trainer-train-under-the-hood.md) | 观察 batch -> forward -> loss -> backward |
| Sequence classification | [`04-sequence-classification-derivation.md`](04-sequence-classification-derivation.md) | 从任务契约反推 tokenizer、collator、model、metric、Trainer |

## Required Observation

用户需要亲自运行或复查 notebook，并把观察写入 [`../../notes/04-lesson-lab/README.md`](../../notes/04-lesson-lab/README.md)：

- sequence classification：raw sample keys、tokenized sample keys、collated batch shapes、`outputs.loss`、`outputs.logits.shape`
- causal LM / Trainer：`batch.keys()`、`input_ids / attention_mask / labels` 的 shape、dtype、device、shifted loss、至少一个 token 位置的 top-k prediction

## Exit Criteria

完成本阶段前，用户应能不用教程解释：

```text
Dataset 提供样本
Collator 组装 batch
Model forward 计算 logits/loss
Trainer 驱动 backward、optimizer、eval、save
```

如果 lesson lab 的观察仍无法解释 hidden mechanism，再进入 `05-mechanism-deep-dive`。
