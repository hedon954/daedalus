# Trainer.train 背后发生了什么？

本 guide 接在 `casual_language_model.ipynb` 后面。你已经跑通了：

```text
dataset -> tokenizer -> group_texts -> data_collator -> Trainer.train -> evaluate -> generate
```

现在要补的是最关键的黑箱：

```text
Trainer.train() 到底在一次训练 step 里做了什么？
```

目标不是读完整个 Trainer 源码，而是把一条 batch 的张量流动看清楚。看清这一层后，后面学 LoRA / SFT / DPO 时，你会知道它们本质上是在改哪一部分：数据、模型参数、loss、optimizer，还是训练循环外壳。

## 先建立心智模型

`Trainer.train()` 可以先简化理解为：

```text
for batch in train_dataloader:
    model.train()
    batch = move_to_device(batch)
    outputs = model(**batch)
    loss = outputs.loss
    loss.backward()
    optimizer.step()
    scheduler.step()
    optimizer.zero_grad()

    if should_log:
        log(loss)
    if should_eval:
        evaluate()
    if should_save:
        save_checkpoint()
```

Hugging Face Trainer 真实源码会处理更多东西，比如：

- device placement
- gradient accumulation
- mixed precision
- distributed training
- logging / eval / save
- hub push
- callback

但这条主线没有变：

```text
batch -> model forward -> loss -> backward -> optimizer step
```

## 当前 notebook 里的角色分工

你的 notebook 中，几个对象分别负责：

| 对象 | 它负责什么 | 不是负责什么 |
| --- | --- | --- |
| `lm_dataset` | 存放已经切成 block 的 token 样本 | 不负责转 tensor，不负责训练 |
| `data_collator` | 把多条样本组 batch，必要时 padding，准备 labels | 不负责 forward/backward |
| `model` | 接收 `input_ids / attention_mask / labels`，输出 `loss / logits` | 不负责遍历数据集 |
| `TrainingArguments` | 配置训练循环的行为 | 不负责模型数学本身 |
| `Trainer` | 把 dataset、collator、model、args 组装成训练循环 | 不改变 Causal LM 的训练目标 |

一句话：

```text
Dataset 提供样本，Collator 打包 batch，Model 计算 loss，Trainer 驱动循环。
```

## 实验 1：观察 Trainer 取出来的一个 batch

在 notebook 训练 cell 后面新增：

```python
batch = next(iter(trainer.get_train_dataloader()))

print(batch.keys())
print("input_ids:", batch["input_ids"].shape, batch["input_ids"].dtype)
print("attention_mask:", batch["attention_mask"].shape, batch["attention_mask"].dtype)
print("labels:", batch["labels"].shape, batch["labels"].dtype)
print("device before forward:", batch["input_ids"].device)
```

你应该看到类似：

```text
dict_keys(['input_ids', 'attention_mask', 'labels'])
input_ids: torch.Size([16, 128])
attention_mask: torch.Size([16, 128])
labels: torch.Size([16, 128])
```

这说明：

- `16` 来自 `per_device_train_batch_size=16`。
- `128` 来自 `block_size=128`。
- `labels` 和 `input_ids` shape 一样，因为 Causal LM 的 shift 发生在模型/loss 内部。

如果你看到 batch 仍在 CPU 上，不奇怪。Trainer 在正式训练 step 里会通过 `_prepare_inputs` 把它移动到目标 device。手动 forward 时需要自己移动。

## 实验 2：手动做一次 model forward

继续新增：

```python
model_device = next(model.parameters()).device
print("model device:", model_device)

batch_on_device = {
    k: v.to(model_device)
    for k, v in batch.items()
}

outputs = model(**batch_on_device)

print("loss:", outputs.loss)
print("logits:", outputs.logits.shape)
```

你应该重点观察：

```text
loss: 一个标量
logits: [batch_size, sequence_length, vocab_size]
```

对你当前设置，大概是：

```text
logits: [16, 128, 50257]
```

这里的 `50257` 是 GPT-2 词表大小。含义是：

```text
每个 batch
  每个 token 位置
    都输出一个“下一个 token 是词表里每个 token 的分数”
```

所以 `logits[0, 0, :]` 表示：

```text
第 1 条样本，第 1 个位置，对整个词表的预测分数
```

## 实验 3：理解 labels 为什么等于 input_ids

打印一小段 token：

```python
row = 0
ids = batch["input_ids"][row]
labels = batch["labels"][row]

print("input ids first 12:", ids[:12].tolist())
print("labels first 12:", labels[:12].tolist())
print("same?", torch.equal(ids, labels))

print("input text:")
print(tokenizer.decode(ids[:24]))
```

你会再次看到：

```text
same? True
```

但训练目标不是“复制输入”。GPT2 源码说明了 `labels` 会在模型/loss 内部 shift。当前 Transformers 版本中：

- `GPT2LMHeadModel.forward(...)` 接收 `labels`，并说明可以设置 `labels = input_ids`。
- `ForCausalLMLoss(...)` 会把 labels 右移：让 token `< n` 预测 token `n`。

可理解为：

```text
input_ids:  A   B   C   D
labels:     A   B   C   D

实际 loss:
预测位置:   A   B   C
目标 token:     B   C   D
```

所以 `labels = input_ids.copy()` 是接口约定，不是训练目标本身。真正的目标仍然是 next-token prediction。

## 实验 4：手动看一个位置在预测什么

先取第一个样本第一个位置的 logits：

```python
pos = 0
next_token_scores = outputs.logits[row, pos]
top = next_token_scores.topk(10)

print("context:")
print(tokenizer.decode(batch["input_ids"][row, : pos + 1]))

print("real next token:")
print(tokenizer.decode([batch["labels"][row, pos + 1].item()]))

print("model top predictions:")
for token_id, score in zip(top.indices.tolist(), top.values.tolist()):
    print(repr(tokenizer.decode([token_id])), score)
```

这个实验的意义：

- `context` 是模型当前位置能看到的上下文。
- `real next token` 是 loss 希望它预测的答案。
- `model top predictions` 是模型当前最相信的候选 token。

你会直观看到：Causal LM 不是一次生成整段文本，而是在每个位置做一次词表分类。

## 实验 5：把 Trainer.training_step 对应回来

源码锚点来自当前 demo venv：

```text
.venv/lib/python3.11/site-packages/transformers/trainer.py
```

关键路径：

```text
training_step(...)
  -> model.train()
  -> inputs = self._prepare_inputs(inputs)
  -> loss = self.compute_loss(model, inputs, ...)
  -> self.accelerator.backward(loss)
```

`compute_loss(...)` 的关键路径：

```text
outputs = model(**inputs)
loss = outputs["loss"] or outputs[0]
```

也就是说，默认情况下，Trainer 并没有自己发明一个 Causal LM loss。它把 batch 传给 model，model 返回 loss，Trainer 拿这个 loss 做 backward。

对你的 notebook 来说：

```text
batch["input_ids"]
batch["attention_mask"]
batch["labels"]
  -> model(**batch)
  -> outputs.loss
  -> backward
```

## 可选实验：看 backward 是否真的产生梯度

这一步会改变当前 `model` 的梯度状态，但不会自动更新参数。只用来观察：

```python
model.zero_grad()
outputs = model(**batch_on_device)
outputs.loss.backward()

param = model.transformer.wte.weight
print(param.grad is None)
print(param.grad.norm())

model.zero_grad()
```

如果看到 `param.grad.norm()` 有值，说明：

```text
loss 已经沿着计算图反传到了 embedding 参数
```

Trainer 在正式训练时会在 backward 之后再做：

```text
optimizer.step()
scheduler.step()
optimizer.zero_grad()
```

这就是模型参数真正被更新的地方。

## 这一步要形成的理解

完成这份 guide 后，你应该能说清楚：

1. `Trainer.train()` 不是魔法，而是训练循环封装。
2. `DataCollatorForLanguageModeling` 输出的是模型可吃的 batch。
3. `AutoModelForCausalLM` 接收 `labels` 后会返回 `loss`。
4. `logits` 的形状是 `[batch, seq_len, vocab_size]`，表示每个位置对整个词表的预测。
5. `labels == input_ids` 是 Causal LM 的接口形态，真正训练目标是 shift 后的 next-token prediction。
6. 训练变慢、生成跑偏、perplexity 不等于生成质量，这些都能从训练循环和目标函数里找到原因。

## 下一步

完成这几个观察后，再进入 Chapter 2 的 pipeline 拆解：

```text
pipeline("text-generation")
  -> tokenizer(prompt, return_tensors="pt")
  -> model.generate(...)
  -> tokenizer.decode(...)
```

到那一步，你会把“训练时的 forward/loss”和“推理时的 generate/decode”区分开。这个区分非常重要：训练是在学习参数，推理是在用参数生成 token。
