# Summarization Inference：从文本到生成摘要

## 记忆框架

这段代码不要按参数硬背，按数据流记：

```text
saved model dir
  -> load tokenizer / model
raw text
  -> tokenizer: text -> input ids
model.generate
  -> output ids
tokenizer.decode
  -> summary text
```

最短口诀：

```text
load same dir -> encode ids -> generate ids -> decode text
```

## 推荐写法

如果 `my_awesome_billsum_model` 是目录名字符串，要加引号：

```python
model_dir = "my_awesome_billsum_model"
```

如果训练后只看到 `checkpoint-*` 子目录，要加载 checkpoint 子目录：

```python
model_dir = "my_awesome_billsum_model/checkpoint-248"
```

推理代码可以写成：

```python
from transformers import AutoTokenizer, AutoModelForSeq2SeqLM

model_dir = "my_awesome_billsum_model"

tokenizer = AutoTokenizer.from_pretrained(model_dir)
model = AutoModelForSeq2SeqLM.from_pretrained(model_dir)

text = "summarize: " + text
inputs = tokenizer(text, return_tensors="pt")

outputs = model.generate(
    **inputs,
    max_new_tokens=100,
    do_sample=False,
)

summary = tokenizer.decode(outputs[0], skip_special_tokens=True)
summary
```

这里用 `**inputs` 比只传 `.input_ids` 更稳，因为 tokenizer 可能同时返回 `input_ids` 和 `attention_mask`。`attention_mask` 告诉模型哪些位置是真输入，哪些位置是 padding。

## 每个参数怎么理解

```python
AutoTokenizer.from_pretrained(model_dir)
```

从保存目录或 Hub repo 加载 tokenizer。它负责文本和 token ids 的互相转换。要和训练时的 tokenizer 保持一致。

```python
AutoModelForSeq2SeqLM.from_pretrained(model_dir)
```

从保存目录或 Hub repo 加载 seq2seq 生成模型。`Auto` 会根据 config 自动选择具体类，比如 T5 会变成对应的 conditional generation model。

```python
tokenizer(text, return_tensors="pt")
```

把文本转成模型输入，并返回 PyTorch tensor。`pt` 就是 PyTorch。

```python
model.generate(...)
```

进入生成模式。对 summarization 来说，它会先让 encoder 读输入，再让 decoder 一个 token 一个 token 生成摘要。

```python
max_new_tokens=100
```

最多生成 100 个新 token。它限制的是输出摘要长度，不是输入长度，也不是 100 个英文单词。

```python
do_sample=False
```

不随机采样。模型每一步更偏向选择最确定的 token，所以结果更稳定、可复现。摘要任务通常先用它；创意写作才更常打开 sampling。

```python
outputs[0]
```

`generate` 返回的是一批输出。即使只输入一条文本，返回值也有 batch 维度，所以第一个结果是 `outputs[0]`。

```python
tokenizer.decode(outputs[0], skip_special_tokens=True)
```

把生成出来的 token ids 转回人类可读文本，并跳过 `<pad>`、`</s>` 这类特殊 token。

## 和训练时的对应关系

训练时：

```text
input ids + labels -> loss
```

推理时：

```text
input ids -> generated ids -> decoded summary
```

训练需要 reference summary 做 `labels`；推理没有 `labels`，只让模型自己生成。

## 常见坑

- T5 summarization 输入仍然要有 `summarize: ` prefix，除非你训练时刻意去掉了这个约定。
- `my_awesome_billsum_model` 如果是目录名，要写成字符串；如果不加引号，Python 会把它当变量名。
- tokenizer 和 model 要从同一个保存目录加载，避免 vocab/config 不一致。
- 单条文本用 `decode`；多条文本可以用 `batch_decode`。
- 本地只验证链路时，先用较小的 `max_new_tokens`，比如 50 或 100。

## 当前排障：根目录不是可加载模型

如果加载时报：

```text
ValueError: Unrecognized model in my_awesome_billsum_model.
Should have a `model_type` key in its config.json.
```

含义是：`AutoModelForSeq2SeqLM.from_pretrained(...)` 没在这个目录里读到可识别的模型配置。

`AutoModel...` 的工作方式是：

```text
read config.json
  -> find model_type
  -> choose concrete model class
  -> load model weights
```

当前本地目录结构里，`my_awesome_billsum_model` 根目录没有模型文件，真正可加载的是：

```text
my_awesome_billsum_model/checkpoint-248/
  config.json              # model_type: t5
  model.safetensors
  tokenizer.json
  tokenizer_config.json
  generation_config.json
```

所以可以直接加载 checkpoint：

```python
model_dir = "my_awesome_billsum_model/checkpoint-248"

tokenizer = AutoTokenizer.from_pretrained(model_dir)
model = AutoModelForSeq2SeqLM.from_pretrained(model_dir)
```

如果想让根目录本身也变成可加载目录，在训练完成后显式保存一次：

```python
trainer.save_model("my_awesome_billsum_model")
tokenizer.save_pretrained("my_awesome_billsum_model")
```

再加载：

```python
model_dir = "my_awesome_billsum_model"
tokenizer = AutoTokenizer.from_pretrained(model_dir)
model = AutoModelForSeq2SeqLM.from_pretrained(model_dir)
```

记忆原则：

```text
TrainingArguments(output_dir=...)
  是训练输出目录

checkpoint-* 或 save_model(...) 产物
  才是 from_pretrained(...) 可加载模型目录
```

## 来源

- [Hugging Face Transformers text generation](https://huggingface.co/docs/transformers/en/main_classes/text_generation)
- [Hugging Face Transformers generation strategies](https://huggingface.co/docs/transformers/en/generation_strategies)
- [Hugging Face Transformers auto classes](https://huggingface.co/docs/transformers/en/model_doc/auto)
