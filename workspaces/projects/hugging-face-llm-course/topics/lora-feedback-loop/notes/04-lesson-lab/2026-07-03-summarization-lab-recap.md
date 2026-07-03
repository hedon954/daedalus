# Summarization Lab Recap

## 学习现场

本 note 记录用户在 Chapter 1/5 Summarization lab 中已经实际跑通和排障过的内容。它是运行证据恢复入口，不替代独立 guide。

相关 guide：

- [`../../guides/04-lesson-lab/08-summarization-derivation.md`](../../guides/04-lesson-lab/08-summarization-derivation.md)
- [`../../guides/04-lesson-lab/09-t5-task-prefix.md`](../../guides/04-lesson-lab/09-t5-task-prefix.md)
- [`../../guides/04-lesson-lab/10-billsum-dataset-loading.md`](../../guides/04-lesson-lab/10-billsum-dataset-loading.md)
- [`../../guides/04-lesson-lab/11-rouge-metric.md`](../../guides/04-lesson-lab/11-rouge-metric.md)
- [`../../guides/04-lesson-lab/13-summarization-compute-metrics-overflow.md`](../../guides/04-lesson-lab/13-summarization-compute-metrics-overflow.md)
- [`../../guides/04-lesson-lab/14-summarization-inference-generate.md`](../../guides/04-lesson-lab/14-summarization-inference-generate.md)

## 运行证据

- Notebook：`demo/hugging-face-course-learning/transformer-work/summarization.ipynb`
- 数据集：`FiscalNote/billsum`，`split="ca_test"`，原始样本数 1237。
- 训练/评估切分：`train_test_split(test_size=0.2)`，得到约 989 条 train、248 条 eval。
- 模型：`google-t5/t5-small`，使用 `AutoModelForSeq2SeqLM` / `Seq2SeqTrainer`。
- 训练输出：`global_step=248`，`epoch=4.0`，`train_loss≈3.02`。
- 推理加载：`my_awesome_billsum_model/checkpoint-248`。
- 推理样例输出：模型能对 Inflation Reduction Act 示例生成简短摘要，说明 `tokenizer -> model.generate -> decode` 链路已跑通。

## 本次修正点

- `load_dataset("billsum")` 在当前环境中会触发短名 URI 解析问题；改用完整 ID `FiscalNote/billsum`。
- `split="ca_test"` 返回单个 `Dataset`；再用 `train_test_split` 生成训练/评估 `DatasetDict`。
- notebook 中用 `billsum_raw` 保留原始 `Dataset`，避免反复运行 cell 时把变量覆盖成 `DatasetDict` 后再调用 `.train_test_split(...)`。
- T5 输入前缀 `summarize: ` 是任务提示，会进入 `input_ids`，不是 label。
- ROUGE 依赖需要 `rouge-score` / `nltk`；依赖已加入 demo 环境。
- 当前 Transformers 路径下，完整 eval 的 `predictions` 中会混入 `-100`，需要在 `compute_metrics` 中把 predictions 和 labels 的 `-100` 都替换为 `tokenizer.pad_token_id` 后再 decode。
- `TrainingArguments(output_dir="my_awesome_billsum_model")` 的根目录不一定是可直接加载模型；本次可加载模型在 `my_awesome_billsum_model/checkpoint-248`。

## 机制观察

Summarization 和前几个 BERT-style lab 的核心差异：

```text
classification / NER / QA:
  input text -> label / token labels / span positions

summarization:
  source text -> generated target text
```

本 lab 已观察到：

- `labels` 是目标 summary 的 token ids，不是分类 id。
- `outputs.logits.shape` 的语义是 decoder 每个目标位置上的 vocab 分布。
- `model.generate(...)` 生成的是 token ids，需要 `tokenizer.decode(...)` 或 `batch_decode(...)` 变回文本。
- ROUGE 比较的是 decoded prediction 和 decoded reference 的文本重叠，不是 token id。

## 下一步

进入 Translation lab，重点不再重新解释 encoder-decoder，而是对比：

```text
summarization:
  source document -> shorter same-language summary

translation:
  source language text -> target language text
```

观察它和 summarization 在 tokenizer、labels、`generate`、decode/postprocess 上哪些完全同构，哪些来自任务提示、语言方向和指标差异。
