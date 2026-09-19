# 04 Lesson Lab：Transformers 可观察实验

本阶段把已经跑通的 Hugging Face Course 示例转成可观察实验。目标不是继续抄 recipe，而是让用户能解释每个对象在训练链路里的职责、输入输出和可验证证据。

## Current Lesson

当前 lesson：

```text
Chapter 1/5 task lab sweep
  -> completed text-task core through summarization
  -> skip/defer Translation / ASR / Image classification
  -> Chapter 1/6 Transformer Architectures / attention mechanisms
  -> complete Chapter 1/8 and Chapter 2 (user-confirmed)
  -> complete reading Chapter 3/1-3/3
  -> complete reading Chapter 3 and Chapter 4
  -> complete Chapter 5/1 Introduction
  -> enter Chapter 5/2 non-Hub dataset loading
  -> map format + local/remote source + split + schema into DatasetDict
```

## Lesson Sequence

| Lesson | Guide | 目标 |
| --- | --- | --- |
| Pipeline foundation | [`01-hf-course-ch1-ch2-foundation.md`](01-hf-course-ch1-ch2-foundation.md) | 把 `pipeline` 展开成 tokenizer、model、postprocess |
| Causal LM recipe | [`02-causal-lm-code-reading.md`](02-causal-lm-code-reading.md) | 解释 dataset -> tokenizer -> blocks -> labels |
| Trainer loop | [`03-trainer-train-under-the-hood.md`](03-trainer-train-under-the-hood.md) | 观察 batch -> forward -> loss -> backward |
| Sequence classification | [`04-sequence-classification-derivation.md`](04-sequence-classification-derivation.md) | 从任务契约反推 tokenizer、collator、model、metric、Trainer |
| Chapter 1/5 task sweep | [`05-chapter1-5-task-lab-sweep.md`](05-chapter1-5-task-lab-sweep.md) | 完成文本主线任务观察，记录剩余 lab 的跳过/延后边界 |
| Token classification | [`06-token-classification-derivation.md`](06-token-classification-derivation.md) | 理解 word-level 标签如何对齐到 token-level logits |
| Question answering | [`07-question-answering-derivation.md`](07-question-answering-derivation.md) | 理解字符级 answer span 如何对齐到 token 级 start/end positions |
| Summarization task shape | [`08-summarization-derivation.md`](08-summarization-derivation.md) | 理解 encoder-decoder 如何把长输入转换成短生成文本 |
| T5 task prefix | [`09-t5-task-prefix.md`](09-t5-task-prefix.md) | 理解 `summarize: ` 如何作为任务提示进入 input ids |
| BillSum dataset loading | [`10-billsum-dataset-loading.md`](10-billsum-dataset-loading.md) | 记录 BillSum 完整 ID、split 和 notebook 变量覆盖边界 |
| ROUGE metric | [`11-rouge-metric.md`](11-rouge-metric.md) | 理解 summarization 自动评估的重叠指标和边界 |
| Mixed precision | [`12-mixed-precision-fp16-bf16.md`](12-mixed-precision-fp16-bf16.md) | 区分 `fp16`、`bf16`、XPU 和本地 MPS 排障默认值 |
| Summarization metric debug | [`13-summarization-compute-metrics-overflow.md`](13-summarization-compute-metrics-overflow.md) | 排查 `batch_decode(predictions)` 的 `OverflowError` |
| Summarization inference | [`14-summarization-inference-generate.md`](14-summarization-inference-generate.md) | 理解 tokenizer、`generate`、`max_new_tokens`、`decode` 的推理链路 |
| Chapter 1/6 architectures | [`15-chapter1-6-transformer-architectures.md`](15-chapter1-6-transformer-architectures.md) | 把已做过的任务映射到 encoder-only、decoder-only、encoder-decoder |
| Chapter 1/8 LLM inference | [`16-chapter1-8-llm-inference.md`](16-chapter1-8-llm-inference.md) | 理解 prompt -> prefill -> decode -> sampling -> KV cache 的推理主线 |
| Chapter 3 fine-tuning | [`17-chapter3-finetuning-pretrained-model.md`](17-chapter3-finetuning-pretrained-model.md) | 从数据契约到 Trainer、full loop、Accelerate、learning curves 与 LoRA 迁移 |
| Chapter 4 Hub sharing | [官方 Chapter 4](https://huggingface.co/learn/llm-course/en/chapter4/1) | Hub 仓库、预训练模型复用、分享与 model card；已读，实践待验收 |
| Chapter 5 Datasets | [`18-chapter5-2-local-remote-dataset-loading.md`](18-chapter5-2-local-remote-dataset-loading.md) | 从本地/远程文件加载 DatasetDict，并排查 URL 探测、split mapping 与 JSON `field` |

## Required Observation

用户需要亲自运行或复查 notebook，并把观察写入 [`../../notes/04-lesson-lab/`](../../notes/04-lesson-lab/) 下的独立 note 文件；[`../../notes/04-lesson-lab/README.md`](../../notes/04-lesson-lab/README.md) 只做索引：

- chapter1/5 text-task sweep：已完成 lab 的 input、batch keys/shape、model class、logits/generated output、postprocess、与上一个 task 的差异
- chapter1/6 architecture summary：encoder-only、decoder-only、encoder-decoder 的 attention 可见范围、典型任务、head / decoder 和输出形态
- chapter1/8 LLM inference：prefill/decode、sampling controls、TTFT/TPOT/throughput/VRAM 和 KV cache
- pipeline internals：tokenizer 输出、model logits、softmax scores、argmax label、`id2label` 映射
- sequence classification：raw sample keys、tokenized sample keys、collated batch shapes、`outputs.loss`、`outputs.logits.shape`
- question answering：`offset_mapping`、`sequence_ids`、`start_positions/end_positions`、`start_logits/end_logits` 和 decoded span
- summarization：`input_ids/attention_mask/labels` shape、`outputs.logits.shape`、`generated_ids` 和 decoded summary
- causal LM / Trainer：`batch.keys()`、`input_ids / attention_mask / labels` 的 shape、dtype、device、shifted loss、至少一个 token 位置的 top-k prediction
- chapter5/2：当前打印 loader format、local/remote source、`data_files`、split、rows、columns 与 nested field；Chapter 3/4 未明确提供的实践证据继续保留

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
