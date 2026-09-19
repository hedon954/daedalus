# Summarization Derivation：为什么从 BERT 转到 BART/T5

## 本节问题

Hugging Face LLM Course Chapter 1/5 在 summarization 这里切到 BART/T5，不是因为 BERT 不重要，而是因为任务契约变了：

```text
BERT 类任务：
  input text -> fixed labels / token labels / span positions

Summarization：
  long input text -> shorter generated text
```

前者更像“读懂后做判断或定位”，后者更像“读懂后重新写一段文本”。

## BERT 的任务形状

BERT 是 encoder-only。它会双向读取输入，得到每个 token 的上下文表示。

前面几个 labs 的共同点是：模型不需要重新生成一段完整文本，只需要把 hidden states 变成某种标签。

```text
Text classification:
  [CLS] hidden state -> sequence label logits

Token classification:
  each token hidden state -> token label logits

Question answering:
  each token hidden state -> start/end logits
```

所以 BERT 的强项是“把输入编码成可判断的表示”。它可以做抽取式 QA，因为答案 span 原本就在 context 里；但它不天然适合 abstractive summarization，因为摘要往往需要生成输入中没有逐字出现的新句子。

## BART/T5 的任务形状

BART/T5 是 encoder-decoder。encoder 先读完整输入，decoder 再根据 encoder 输出自回归生成目标文本。

```text
long article
  -> encoder: understand source sequence
  -> decoder: generate summary token by token
  -> decode generated_ids into summary text
```

这就是 seq2seq：输入是一个序列，输出也是另一个序列。Summarization 和 translation 都属于这个形状。

BART 的预训练方式也贴近这个目标：把文本破坏掉，再训练模型重建原文。这个过程迫使模型学会“从损坏或压缩的信息中生成完整文本”，所以它适合需要生成目标序列的任务。

## Lab 里要观察什么

Summarization lab 不要只看最终摘要像不像人话。优先打印这些中间对象：

```text
raw sample:
  article / document
  summary

tokenized batch:
  input_ids.shape
  attention_mask.shape
  labels.shape

model:
  AutoModelForSeq2SeqLM / T5ForConditionalGeneration / BartForConditionalGeneration

forward:
  outputs.loss
  outputs.logits.shape

generation:
  generated_ids.shape
  tokenizer.batch_decode(generated_ids, skip_special_tokens=True)
```

关键观察：

- `labels` 不是分类 id，而是目标 summary 的 token ids。
- `outputs.logits.shape` 通常是 `[batch_size, target_seq_len, vocab_size]`，不是 `[batch_size, num_labels]`。
- 训练 loss 比较的是 decoder 每个位置预测的 token 和目标 summary token。
- 推理时用 `model.generate(...)`，再把 `generated_ids` decode 成文本。
- postprocess 的核心不是 `argmax -> label`，而是 `generated ids -> text`。

## 和前几个 lab 的最大差异

```text
Sequence classification:
  one text -> one label

Token classification:
  one text -> one label per token

Question answering:
  question + context -> start/end token positions

Summarization:
  long text -> generated short text
```

这也是为什么课程先用 BERT 讲 classification / NER / QA，再用 BART/T5 讲 summarization / translation：不是模型名字切换，而是任务输出形状切换。

## 最小验收

跑完本 lab 后，至少能不用教程说清：

1. 为什么 summarization 是 seq2seq，而不是 classification。
2. 为什么 encoder-decoder 比 encoder-only 更自然。
3. `labels` 为什么是一串 summary token ids。
4. `generate()` 和前面 `argmax(logits)` 的区别。
5. Summarization 和 translation 为什么可以共用同一类模型结构。

## 来源

- [Hugging Face LLM Course Chapter 1/5: How Transformers solve tasks](https://huggingface.co/learn/llm-course/en/chapter1/5)
- [Hugging Face Transformers summarization task guide](https://huggingface.co/docs/transformers/tasks/summarization)
