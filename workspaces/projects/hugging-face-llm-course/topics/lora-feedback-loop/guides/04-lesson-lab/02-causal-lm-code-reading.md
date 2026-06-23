# Causal Language Modeling 代码阅读指南

本文件解释 `causal_language_model.py` 这类 Hugging Face recipe 背后的底层逻辑。

概念入口不是“我要先完整微调一个模型”，而是 Chapter 1/5 Text generation 小节里的这句话：

```text
GPT-2's pretraining objective is based entirely on causal language modeling,
predicting the next word in a sequence.
```

这句话讲的是 GPT-2 为什么擅长生成文本；`causal_language_model.py` 展示的是如何把自己的文本数据也整理成同一种 next-token prediction 训练目标。

用户当前感受：代码能跟着抄，但不知道每一步在做什么、为什么这么写、如果没有参考资料如何自己写出来。

## 先判断：这段代码到底在做什么？

它不是单纯的 text generation inference。

它是在做一个 causal language modeling fine-tuning pipeline。它是 “GPT-2 预训练目标” 的下游应用，而不是 Chapter 1/5 那句话本身：

```text
原始问答文本
  -> 抽取训练文本
  -> tokenizer 转成 token ids
  -> 拼成长 token stream
  -> 切成固定长度 block
  -> labels = input_ids
  -> causal LM 内部右移 labels
  -> 计算 next-token prediction loss
  -> Trainer 微调模型
```

一句话：把很多段文本整理成“给前面的 token，预测下一个 token”的训练样本。

## 底层任务：为什么叫 Causal LM？

Chapter 1/5 里讲到：

- BERT 这类 encoder 常用 masked language modeling：看左右文，预测被遮住的 token。
- GPT 这类 decoder 常用 causal language modeling：只能看左边上下文，预测下一个 token。

Causal LM 的训练目标可以写成：

```text
输入:  token_1, token_2, token_3
目标:          token_2, token_3, token_4
```

模型不是一次预测整篇文章的“语义”，而是在每个位置预测下一个 token 的概率分布。

## 你的代码每一步在解决什么问题

### 1. 登录 Hugging Face

```python
notebook_login()
```

用途：

- 如果要 push model 到 Hub，需要身份。
- 如果下载 gated/private 资源，也需要 token。

当前学习阶段不一定需要。尤其本地脚本里用 `notebook_login()` 会让流程不够脚本化，后面更适合改成环境变量或 `huggingface-cli login`。

### 2. 加载数据集

```python
eli5 = load_dataset("dany0407/eli5_category", split="train[:5000]")
eli5 = eli5.train_test_split(test_size=0.2)
```

用途：

- 先拿一部分 ELI5 数据，避免全量太慢。
- 切出 train/test，后面训练和评估要分开。

底层逻辑：

训练不是只看模型能不能记住训练文本，还要看它在没训练过的文本上 loss 如何。

### 3. 加载 tokenizer

```python
tokenizer = AutoTokenizer.from_pretrained("distilbert/distilgpt2")
```

用途：

- 把字符串变成模型能处理的 token ids。
- tokenizer 必须和模型匹配，否则 token id 的含义会错。

底层逻辑：

模型不认识文字，只认识整数 id。tokenizer 是“文本世界”和“模型张量世界”的边界。

### 4. flatten 数据

```python
eli5 = eli5.flatten()
```

用途：

- ELI5 的字段里有嵌套结构，例如 `answers.text`。
- flatten 后更容易在 `map` 里批量取字段。

底层逻辑：

训练管线需要稳定、扁平、批处理友好的字段。

### 5. preprocess：抽文本 + tokenization

```python
def preprocess_function(examples):
    return tokenizer([" ".join(x) for x in examples["answers.text"]])
```

用途：

- 取每条样本的回答文本。
- 多个回答合并成一段训练文本。
- 用 tokenizer 转成 `input_ids` 和 `attention_mask`。

底层逻辑：

Causal LM 不关心“这是第几个回答”的原始结构，它需要的是连续文本 token 序列。

### 6. remove_columns

```python
tokenized_eli5 = eli5.map(
    preprocess_function,
    batched=True,
    num_proc=4,
    remove_columns=eli5["train"].column_names,
)
```

用途：

- 去掉原始文本字段，只保留模型训练需要的 token 字段。

底层逻辑：

Trainer 后面只需要张量化字段。原始字段如果混在 batch 里，collator 可能不知道怎么处理。

### 7. group_texts：拼接再切块

```python
block_size = 128

def group_texts(examples):
    concatenated_examples = {k: sum(examples[k], []) for k in examples.keys()}
    total_length = len(concatenated_examples[list(examples.keys())[0]])
    total_length = (total_length // block_size) * block_size
    result = {
        k: [t[i : i + block_size] for i in range(0, total_length, block_size)]
        for k, t in concatenated_examples.items()
    }
    result["labels"] = result["input_ids"].copy()
    return result
```

这是最容易迷糊的一步。

它做了三件事：

1. 把 batch 里的 token list 全部拼成一条长 token stream。
2. 按 `block_size` 切成固定长度片段。
3. 复制 `input_ids` 作为 `labels`。

为什么要这样？

- GPT 类模型训练时需要固定长度或批内可对齐的 token blocks。
- 文本生成模型不需要每条原始问答都保持边界，它只需要大量连续 token 来练 next-token prediction。
- `labels = input_ids` 看起来奇怪，但 causal LM 的 model 内部会做 shift：用当前位置之前的 token 预测当前位置 token。

## 8. data collator

```python
tokenizer.pad_token = tokenizer.eos_token
data_collator = DataCollatorForLanguageModeling(tokenizer=tokenizer, mlm=False)
```

用途：

- 把多条样本组成 batch。
- 动态 padding。
- `mlm=False` 表示不是 BERT 式 masked LM，而是 causal LM。

为什么 `pad_token = eos_token`？

GPT2 系列默认没有 pad token。为了 batch padding，需要指定一个 pad token。常见做法是复用 eos token。

## 9. AutoModelForCausalLM

```python
model = AutoModelForCausalLM.from_pretrained("distilbert/distilgpt2")
```

用途：

- 加载适合 causal language modeling 的模型头。

底层逻辑：

`AutoModelForCausalLM` 不只是 base transformer，它带了 language modeling head，可以把 hidden states 映射到 vocabulary logits。

输出形状可以理解成：

```text
[batch_size, sequence_length, vocab_size]
```

每个位置都在预测下一个 token 的概率分布。

## 10. Trainer

```python
trainer = Trainer(
    model=model,
    args=training_args,
    train_dataset=lm_dataset["train"],
    eval_dataset=lm_dataset["test"],
    data_collator=data_collator,
    processing_class=tokenizer,
)

trainer.train()
```

用途：

- 管理训练循环。
- 负责 batch、forward、loss、backward、optimizer、eval、checkpoint。

底层逻辑：

Trainer 是工程封装。它不改变机器学习问题本身，只是帮你少写训练循环。

## 如果没有参考资料，怎么自己写出来？

按这条推理链：

1. 我想训练什么任务？
   - text generation。

2. text generation 对应什么训练目标？
   - causal LM / next-token prediction。

3. 模型需要吃什么？
   - token ids、attention mask、labels。

4. 原始数据是什么？
   - 人类文本，不是 token。

5. 所以第一步必须做什么？
   - tokenizer。

6. Causal LM 的 labels 是什么？
   - 原始 input_ids 的 copy，shift 在模型内部处理。

7. 训练 batch 怎么构造？
   - 把 token 切成固定 block，再 padding/collate。

8. 用什么模型头？
   - AutoModelForCausalLM。

9. 用什么训练循环？
   - Trainer 或自己写 PyTorch loop。

这就是背后的底层逻辑。不是“记住 API”，而是从任务目标倒推出数据形状、模型头和训练循环。

## 当前代码里值得调整的点

- `from numpy import block` 没有必要，容易和 `block_size` 概念混淆。
- 本地脚本里不建议一开始就 `notebook_login()` 和 `push_to_hub=True`。
- `block_size = 128` 是为了轻量实验，不是模型上限。
- 后续应先做一个“只跑 preprocessing、不训练”的观察脚本，打印每一步数据形状。

## 下一步

不要马上完整训练。

先把当前脚本拆成三个可观察阶段：

1. `inspect_dataset.py`：看原始 ELI5 样本结构。
2. `inspect_tokenization.py`：看 tokenizer 输出。
3. `inspect_lm_blocks.py`：看 group_texts 后的 block 和 labels。

确认这些都看懂，再进入 Trainer。
