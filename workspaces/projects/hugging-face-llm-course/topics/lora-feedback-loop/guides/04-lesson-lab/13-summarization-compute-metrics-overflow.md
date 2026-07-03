# Summarization Metric Debug：`batch_decode` OverflowError

## 现象

如果 `trainer.train()` 在 evaluation 阶段报：

```text
OverflowError: out of range integral type conversion attempted
```

并且 stack trace 指到：

```python
decoded_preds = tokenizer.batch_decode(predictions, skip_special_tokens=True)
```

问题在 ROUGE 前一步：`tokenizer.batch_decode(...)` 收到了不能 decode 的 token id。

## 官方代码的隐含前提

先不要理解成“官方代码错了”。官方这段 `compute_metrics` 隐含了一个前提：`predictions` 已经是 `model.generate(...)` 产生的合法 token ids，所以可以直接 decode。这个前提通常由下面几件事共同保证：

- 使用 `Seq2SeqTrainer`。
- `Seq2SeqTrainingArguments(predict_with_generate=True)`。
- trainer 收到 tokenizer / processing class，能正确处理 generation 和 padding。

一个常见配置错误是：

```python
processing_class=preprocess_function
```

这里应该传 tokenizer：

```python
processing_class=tokenizer
```

`preprocess_function` 是给 `Dataset.map(...)` 用的；`tokenizer` 才是给 trainer/collator/decode 用的。

## 最小诊断

如果修正 trainer 后仍然在 `batch_decode(predictions)` 炸，就说明当前环境下传进 `compute_metrics` 的 `predictions` 仍然含有不能 decode 的值。先打印 shape/range：

```python
def compute_metrics(eval_pred):
    predictions, labels = eval_pred
    if isinstance(predictions, tuple):
        predictions = predictions[0]

    print("predictions shape:", predictions.shape)
    print("predictions min/max:", predictions.min(), predictions.max())
    print("labels min/max:", labels.min(), labels.max())
    ...
```

判断方式：

```text
predictions.shape == [batch, generated_seq_len]:
  正常 generation 输出

predictions.shape == [batch, seq_len, vocab_size]:
  这不是 generated ids，而是 logits

predictions.min() < 0:
  里面有 ignore index，不能直接 decode
```

## 当前环境实证

本地最小复现中，用同一环境、2 条 BillSum eval 样本、`processing_class=tokenizer` 跑 `trainer.evaluate()`，`predictions` 是正常 generated ids：

```text
PRED_SHAPE (2, 21)
PRED_DTYPE int64
PRED_MINMAX 0 21012
```

这种情况下官方 `tokenizer.batch_decode(predictions, ...)` 可以正常运行。

但用完整 `ca_test` 切分后的 248 条 eval 样本复现时，当前环境会在 predictions 中混入 `-100`：

```text
PRED_SHAPE (248, 21)
PRED_DTYPE int64
PRED_MINMAX -100 31433
NEG_VALUES [(-100, 232)]
```

因此当前 notebook 不能只照官方简化版处理 labels；也要先处理 predictions。

## 修复写法

```python
def compute_metrics(eval_pred):
    predictions, labels = eval_pred

    if isinstance(predictions, tuple):
        predictions = predictions[0]

    predictions = np.where(predictions != -100, predictions, tokenizer.pad_token_id)
    labels = np.where(labels != -100, labels, tokenizer.pad_token_id)

    decoded_preds = tokenizer.batch_decode(predictions, skip_special_tokens=True)
    decoded_labels = tokenizer.batch_decode(labels, skip_special_tokens=True)

    result = rouge.compute(
        predictions=decoded_preds,
        references=decoded_labels,
        use_stemmer=True,
    )

    prediction_lens = [
        np.count_nonzero(pred != tokenizer.pad_token_id)
        for pred in predictions
    ]
    result["gen_len"] = np.mean(prediction_lens)

    return {k: round(v, 4) for k, v in result.items()}
```

这个修复不改变模型输出含义，只是把 evaluation padding 用的 ignore index 换成 tokenizer 能 decode 的 pad token。

## 来源

- [Hugging Face Transformers summarization task guide](https://huggingface.co/docs/transformers/tasks/summarization)
