# Token Classification Derivation：从任务契约推导代码

## 本节定位

Token classification 是 Chapter 1/5 task lab sweep 的第三个 lab。它和 sequence classification 很像，都是 encoder-only BERT 类模型，但监督信号从“一整句一个标签”变成“每个 token 一个标签”。

本节不要从 `Trainer` 开始理解，而要从任务契约开始：

```text
输入：一句已经按词切好的句子 tokens
目标：给每个原始词标一个 NER 标签 ner_tags
难点：tokenizer 会把一个词拆成多个 subword，还会加 [CLS] / [SEP]
处理：把原始 word-level label 对齐到 tokenizer 之后的 token 序列
模型：DistilBERT backbone + token classification head
输出：每个 token 位置都有一组实体类别 logits
后处理：忽略 -100 位置，把 token 预测合并成人类可读的实体片段
```

## 和 Text Classification 的差异

```text
Text classification:
  one sequence -> one label
  logits.shape == [batch_size, num_labels]

Token classification:
  one sequence -> one label per token
  logits.shape == [batch_size, seq_len, num_labels]
```

核心差异不是训练 API，而是监督信号的粒度：

```text
Text classification:
  [CLS] / sequence representation -> one logits vector -> one label

Token classification:
  every token hidden state -> one logits vector per token -> one label per token
```

## 推导顺序

### 1. 原始样本

WNUT 样本提供的是 word-level 标注：

```text
tokens:   ["@paulwalk", "It", "'s", ...]
ner_tags: [0,            0,    0,    ...]
```

这里 `tokens[i]` 和 `ner_tags[i]` 一一对应。此时标签坐标系是“原始词”。

### 2. Tokenizer 改变坐标系

`tokenizer(..., is_split_into_words=True)` 会保留“输入已经按词切好”的信息，但输出仍然是模型需要的 token 序列：

```text
["@paulwalk"] -> ["@", "paul", "##walk"]
```

同时 tokenizer 还会加入 `[CLS]`、`[SEP]` 等 special tokens。因此 tokenizer 后的 token 数量通常大于原始词数量。

这就是 token classification 多出来的核心问题：原始标签还在 word-level，但模型训练需要 token-level labels。

### 3. Label Alignment

`tokenized_inputs.word_ids(batch_index=i)` 会告诉你 tokenizer 后每个 token 来自第几个原始词：

```text
token:    [CLS]  @   paul  ##walk  it  ...
word_id:  None   0   0     0       1   ...
```

对齐策略是：

```text
special token -> -100
一个词的第一个 token -> 原始词标签
同一个词拆出来的后续 subword -> -100
```

`-100` 不是实体类别，而是 loss 的忽略标记。它告诉训练过程：这个位置不要参与监督。

注意：`attention_mask` 和 `labels == -100` 不是一回事。

```text
attention_mask: 告诉模型哪些位置是 padding
labels == -100: 告诉 loss 哪些位置不计算监督误差
```

### 4. Collator

`DataCollatorForTokenClassification` 的职责是把不同长度的样本 padding 成一个 batch，并让 `input_ids`、`attention_mask`、`labels` 一起对齐。

必须观察：

```text
batch.keys()
batch["input_ids"].shape
batch["attention_mask"].shape
batch["labels"].shape
```

预期三者前两个维度一致：

```text
[batch_size, seq_len]
```

### 5. Model Forward

`AutoModelForTokenClassification` 使用 DistilBERT backbone，但分类 head 作用在每个 token hidden state 上。

预期输出：

```text
outputs.loss
outputs.logits.shape == [batch_size, seq_len, num_labels]
```

这就是本 lab 和 sequence classification 最关键的结构差异。

### 6. Metric 与 Postprocess

`compute_metrics` 先对 logits 做：

```text
argmax over num_labels
```

然后过滤掉 `labels == -100` 的位置，只把真实监督过的位置交给 `seqeval`。

`pipeline("ner")` 会再做一层人类可读 postprocess：把 token-level 的预测合并成实体片段。

## 当前 Notebook 的坑位

### 数据集加载

课程示例里的 `load_dataset("wnut_17")` 依赖旧式 dataset loading script；在 `datasets>=4.0.0` 中会报：

```text
Dataset scripts are no longer supported, but found wnut_17.py
```

当前 lab 使用 Parquet 转换版：

```python
wnut = load_dataset("flaitenberger/wnut_17")
```

它保留 `id`、`tokens`、`ner_tags` 字段，足够继续观察 label alignment 和 token-level logits。

### `word_ids` 变量引用

`word_ids(batch_index=i)` 必须从当前 batch 的 `tokenized_inputs` 取：

```python
word_ids = tokenized_inputs.word_ids(batch_index=i)
```

如果误用单条样本的 `tokenized_input`，`batched=True` 时会在第 2 条样本开始触发：

```text
IndexError: list index out of range
```

### 本地 Pipeline 加载

`TrainingArguments(output_dir=...)` 只是指定训练输出目录，不等于已经保存了一个可被 `pipeline` 加载的模型。

如果没有产生 checkpoint 或没有显式保存，`pipeline(..., model=output_dir)` 会因为缺少 `config.json`、权重或 tokenizer 文件而失败。

训练结束后需要保存：

```python
trainer.save_model("my_awesome_wnut_model")
tokenizer.save_pretrained("my_awesome_wnut_model")
```

## 最小验收

本 lab 完成时，用户应能闭卷解释：

```text
raw tokens / ner_tags
-> tokenizer changes the token coordinate system
-> word_ids maps model tokens back to original words
-> labels use -100 to ignore special tokens and extra subwords
-> collator pads input_ids / attention_mask / labels together
-> model outputs [batch_size, seq_len, num_labels]
-> seqeval and pipeline filter / merge token-level predictions
```

不要把跑通训练等同于完成。完成证据应该来自用户能说清：为什么 token classification 比 sequence classification 多了 label alignment，以及为什么 logits 多出 `seq_len` 维。
