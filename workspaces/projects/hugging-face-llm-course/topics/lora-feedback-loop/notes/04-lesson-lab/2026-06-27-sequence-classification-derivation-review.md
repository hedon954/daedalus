# 2026-06-27 Sequence Classification Derivation Review

### 用户当前复述

用户已经能解释前三段链路：

- Dataset：加载原始数据集，提供 train / test，每条数据包含 `text` 和 `label`；`label` 服务于最终分类目标。
- Tokenizer：把 dataset 中的 `text` 转为 `input_ids` 和 `attention_mask`，因为模型不能直接处理自然语言字符串。
- Collator：把多条变长样本通过 padding 填充到一样长，从而可以 batch 训练。

### 需要校正的点

Tokenizer 不是把文字“向量化”。更准确的链路是：

```text
text
  -> tokenizer
  -> tokens
  -> token ids
  -> model embedding layer
  -> embedding vectors
```

`input_ids` 是 embedding table 的查表索引；真正的向量表示发生在模型内部的 embedding layer。

### 当前卡点

用户暂时不知道 4 / 5 要做什么：

```text
4. 手动跑 model(**batch)：观察 outputs.loss 和 outputs.logits.shape。
5. 手动 argmax(logits) 再查 id2label：理解 pipeline 的 label 如何从模型输出得到。
```

### 下一步动作

- 按 guide 的 `Model forward` / `Manual prediction` 小节，把 4 / 5 的 probe 加到 notebook。
- 只回答三个问题：`batch` 里有什么，`model forward` 输出了什么，`logits` 如何变成最终 `label`。
- 相关说明已补入 [`../../guides/04-lesson-lab/04-sequence-classification-derivation.md`](../../guides/04-lesson-lab/04-sequence-classification-derivation.md)。

### Collator probe 报错

用户在手动运行：

```python
features = [tokenized_imdb["test"][i] for i in range(3)]
batch = data_collator(features=features)
```

时遇到：

```text
ValueError: too many dimensions 'str'
Perhaps your features (`text` in this case) have excessive nesting
```

诊断：`tokenized_imdb["test"][i]` 里仍保留原始 `text` 字符串字段。手动调用 `data_collator` 时，它会尝试把 features 中的字段整理成 tensor；`text` 不是模型输入 tensor，因此报错。

修复：手动 probe 时只保留 `input_ids`、`attention_mask`、`label`，再交给 `DataCollatorWithPadding`。guide 中的 Collated batch 和 4/5 probe 代码已同步修正。

### Model forward / Manual prediction 观察

用户按 guide 跑通 4 / 5 probe，观察到：

```text
batch keys: dict_keys(['input_ids', 'attention_mask', 'labels'])
labels: tensor([0, 0, 0], device='mps:0')
loss: tensor(0.5448, device='mps:0', grad_fn=<NllLossBackward0>)
logits shape: torch.Size([3, 2])
logits: tensor([[ 0.3006, -0.3274],
        [-0.0302, -0.0735],
        [ 0.1679, -0.1778]], device='mps:0', grad_fn=<LinearBackward0>)
pred ids: tensor([0, 0, 0], device='mps:0')
pred labels: ['POSITIVE', 'POSITIVE', 'POSITIVE']
gold labels: ['POSITIVE', 'POSITIVE', 'POSITIVE']
```

当前解释：

- `batch keys` 表明模型真正吃到的是 `input_ids`、`attention_mask`、`labels`，不再包含原始 `text`。
- `labels` 是真实类别 id；当前 3 条样本的真实类别 id 都是 `0`。
- `loss=0.5448` 是模型把 `logits` 和 `labels` 对比后算出的分类损失；`grad_fn=<NllLossBackward0>` 说明这个 loss 仍连接着反向传播计算图。
- `logits shape=[3, 2]` 表示 batch 有 3 条样本，每条样本有 2 个类别分数。
- 每行 logits 里第 0 列都大于第 1 列，所以 `argmax` 得到 `pred ids=[0,0,0]`。
- `pred labels` 和 `gold labels` 都是 `POSITIVE`，说明在当前 `model.config.id2label` 下，这 3 条样本预测正确。

需要继续确认：当前输出显示 label id `0` 被映射为 `POSITIVE`。这必须和数据集真实 label 语义保持一致；否则可能出现数字预测对了，但文字标签反了的问题。guide 已补充 `id2label` / dataset label sanity check。

### 当前 checkpoint

用户确认已理解 sequence classification 的 forward / logits / loss / label mapping 链路：

```text
batch
  -> model(**batch)
  -> loss + logits
  -> argmax(logits)
  -> pred ids
  -> id2label
  -> human-readable labels
```

本轮 lesson 不再继续扩展新 API。下一步是闭卷重写最小 sequence classification flow，验证是否能脱离教程按任务契约推导代码。

### Closed-book rewrite 验收

用户在临时 notebook `sequence_classification_2.ipynb` 中闭卷实现了最小 sequence classification flow：

```text
load dataset
-> tokenizer
-> map preprocess
-> DataCollatorWithPadding
-> AutoModelForSequenceClassification
-> TrainingArguments / Trainer
-> compute_metrics
-> model forward
-> logits -> argmax -> id2label
```

验收结果：通过。notebook 无保存的错误输出，并打印了 batch keys、labels、loss、logits shape、logits、pred ids、pred labels、gold labels。

学习 caveat：临时 notebook 使用 `select(range(300))` 构造小数据集，IMDb 前若干条样本可能标签分布偏斜；它适合机制复现，不适合作为效果评估。后续小样本训练应使用 `shuffle(seed=42)` 或构造 label-balanced subset。

临时文件已删除，因为主 notebook `sequence_classification.ipynb` 已保留该逻辑。

下一步：进入 Chapter 2 `Behind the pipeline`，手写 `pipeline("text-classification")` 的 tokenizer -> model -> postprocess 最小等价流程。
