# 2026-06-26 Sequence Classification Lab Completion

### 运行目标

完成 Hugging Face Transformers text / sequence classification recipe，建立从 IMDb 文本分类数据到 DistilBERT fine-tuning 和 pipeline 推理的整体体感。

### 运行环境

- Notebook：`demo/hugging-face-course-learning/transformer-work/sequence_classification.ipynb`
- Dataset：`stanfordnlp/imdb`
- Base model：`distilbert/distilbert-base-uncased`
- 关键组件：`AutoTokenizer`、`DataCollatorWithPadding`、`AutoModelForSequenceClassification`、`TrainingArguments`、`Trainer`、`pipeline`

### 用户观察

用户已完成第二个 lab，对大致流程有一些理解，但整体仍是模糊印象；当前主要卡点不是“代码是否能跑”，而是仍然只能按教程抄，无法自己推导出这条链路。

### 当前结论

Sequence classification 的主链路应被理解为：

```text
raw text + label
  -> tokenizer(text, truncation=True)
  -> DataCollatorWithPadding 动态组成 batch
  -> AutoModelForSequenceClassification 输出 [batch_size, num_labels] logits
  -> compute_metrics 把 logits argmax 后与 labels 比较
  -> Trainer 驱动 train / eval / save / push
  -> pipeline 或手动 forward 做推理
```

当前退出标准不是“能复现官方代码”，而是能从任务契约反推每个对象为什么存在、输入输出是什么、删掉会坏在哪里。

### 下一步动作

- 按 [`../../guides/04-lesson-lab/04-sequence-classification-derivation.md`](../../guides/04-lesson-lab/04-sequence-classification-derivation.md) 复查 notebook。
- 补 raw sample、tokenized sample、collated batch、model forward、logits -> label 的实际打印观察。
- 盖住教程，用 guide 里的 8 段 skeleton 重新写一遍最小 sequence classification flow。
