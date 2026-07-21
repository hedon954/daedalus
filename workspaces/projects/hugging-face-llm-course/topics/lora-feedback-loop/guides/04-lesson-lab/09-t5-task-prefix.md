# T5 Task Prefix：为什么输入前要加 `summarize: `

## 本节问题

截图里的代码：

```python
prefix = "summarize: "
inputs = [prefix + doc for doc in examples["text"]]
```

不是给人看的注释，而是把任务指令直接拼进模型输入里。

## 输入实际变成什么

假设原始 bill text 是：

```text
The people of the State of California do enact as follows: ...
```

拼接后送给 tokenizer 的输入会变成：

```text
summarize: The people of the State of California do enact as follows: ...
```

`prefix + doc` 发生在 tokenizer 之前，所以 `summarize:` 也会被 tokenizer 编进 `input_ids`。

## 为什么 T5 需要它

T5 被设计成 text-to-text 模型：很多任务都统一成“输入一段文本，输出另一段文本”。同一个模型可以做 summarization、translation、question answering 等任务，所以输入里需要带一个任务提示，告诉模型“这次你要做摘要”。

可以把它理解成：

```text
普通 encoder-decoder:
  source text -> target text

T5 style:
  task prefix + source text -> target text
```

在这个 lab 里：

```text
input text:
  "summarize: " + bill text

target text:
  bill summary
```

它不是 label，也不会出现在 `labels` 里；`labels` 来自 `examples["summary"]`。

## 为什么 tokenizer 分两行

```python
model_inputs = tokenizer(inputs, max_length=1024, truncation=True)
labels = tokenizer(text_target=examples["summary"], max_length=128, truncation=True)
```

第一行处理“带任务提示的输入文本”，第二行处理“模型应该生成的目标摘要”。

## 来源

- [Hugging Face Transformers summarization task guide](https://huggingface.co/docs/transformers/tasks/summarization)
