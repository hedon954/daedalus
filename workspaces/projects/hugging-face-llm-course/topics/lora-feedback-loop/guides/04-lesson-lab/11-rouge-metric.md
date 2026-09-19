# ROUGE Metric：摘要评估怎么看

## 核心直觉

ROUGE 是 summarization 里最常见的自动评估指标之一。它的核心思想很朴素：把模型生成的 summary 和人工 reference summary 做文本重叠比较。

```text
prediction: model generated summary
reference: dataset human summary
ROUGE: overlap(prediction, reference)
```

## 常见指标

| 指标 | 比较什么 | 直觉 |
| --- | --- | --- |
| `rouge1` | unigram / 单词重叠 | 生成摘要有没有覆盖 reference 里的关键词 |
| `rouge2` | bigram / 连续两个词重叠 | 生成摘要有没有学到局部短语结构 |
| `rougeL` | longest common subsequence | 生成摘要和 reference 的长顺序片段有多像 |
| `rougeLsum` | summary-level LCS | 更适合多句摘要，通常会按换行处理句子边界 |

名字里的 Recall-Oriented 是历史来源：它关心 reference 中有多少内容被生成摘要覆盖。但在 Hugging Face `evaluate.load("rouge")` 的当前实现里，默认返回的是各项 ROUGE 的 F1 分数，而不是只返回 recall。

## 课程代码里的链路

```python
rouge = evaluate.load("rouge")
result = rouge.compute(
    predictions=decoded_preds,
    references=decoded_labels,
    use_stemmer=True,
)
```

这里必须先 decode，因为 ROUGE 比的是文本，不是 token id：

```text
generated ids -> decoded_preds
label ids -> decoded_labels
rouge.compute(predictions=decoded_preds, references=decoded_labels)
```

`use_stemmer=True` 会把英文词尾做简单归一化，比如 treats / treated / treating 更容易被看成同一词根。对 BillSum 这种英文数据集有帮助；非英文任务要更谨慎，必要时传自定义 tokenizer。

## 边界

- 它奖励词面重叠，不真正理解语义。
- 它不能可靠判断事实是否正确。
- 同义改写可能分低，照抄 reference 关键词可能分高。
- 它适合做训练过程中的粗略趋势指标，不应该替代人工读样本。

对本 lab 来说，最小观察不是追求 ROUGE 很高，而是确认：

```text
model.generate(...)
  -> tokenizer.batch_decode(...)
  -> rouge.compute(predictions=..., references=...)
  -> rouge1 / rouge2 / rougeL / rougeLsum
```

## 来源

- [Hugging Face Evaluate ROUGE metric card](https://huggingface.co/spaces/evaluate-metric/rouge)
- [Hugging Face Transformers summarization task guide](https://huggingface.co/docs/transformers/tasks/summarization)
