# 04 Lesson Lab：Causal LM / Trainer 可观察实验

本阶段把已经跑通的 Hugging Face Course 示例转成可观察实验。目标不是继续抄 recipe，而是让用户能解释每个对象在训练链路里的职责、输入输出和可验证证据。

## Current Lesson

当前 lesson：

```text
ELI5 Causal LM quick training
  -> Trainer batch
  -> model forward
  -> loss / logits
  -> backward / optimizer step
```

## Lesson Sequence

| Lesson | Guide | 目标 |
| --- | --- | --- |
| Pipeline foundation | [`01-hf-course-ch1-ch2-foundation.md`](01-hf-course-ch1-ch2-foundation.md) | 把 `pipeline` 展开成 tokenizer、model、postprocess |
| Causal LM recipe | [`02-causal-lm-code-reading.md`](02-causal-lm-code-reading.md) | 解释 dataset -> tokenizer -> blocks -> labels |
| Trainer loop | [`03-trainer-train-under-the-hood.md`](03-trainer-train-under-the-hood.md) | 观察 batch -> forward -> loss -> backward |

## Required Observation

用户需要亲自运行或复查 notebook，并把观察写入 [`../../notes/04-lesson-lab/README.md`](../../notes/04-lesson-lab/README.md)：

- `batch.keys()`
- `input_ids / attention_mask / labels` 的 shape、dtype、device
- `outputs.loss`
- `outputs.logits.shape`
- `labels == input_ids` 与 shifted loss 的关系
- 至少一个 token 位置的 top-k prediction

## Exit Criteria

完成本阶段前，用户应能不用教程解释：

```text
Dataset 提供样本
Collator 组装 batch
Model forward 计算 logits/loss
Trainer 驱动 backward、optimizer、eval、save
```

如果 lesson lab 的观察仍无法解释 hidden mechanism，再进入 `05-mechanism-deep-dive`。
