# Chapter 1/6 Decoder-only 复述

## 场景

用户继续阅读 Hugging Face LLM Course Chapter 1/6 `Transformer Architectures`，当前检查点是：为什么 Causal LM / text generation 更适合映射到 decoder-only。

## 用户复述

用户的理解：

> Causal LM / text generation 的核心是续写当前 input，而不是从当前 input 里面摘东西。

## Agent 校验

这个复述抓住了 decoder-only 的核心：任务目标不是理解完整输入后做抽取或分类，而是基于已有前文继续生成后续 token。因此关键机制是：

```text
已有前文
-> causal / masked self-attention，只看当前位置之前的信息
-> next-token logits
-> 选择或采样下一个 token
-> 把新 token 接回上下文继续生成
```

这也解释了为什么 Causal LM 训练中 `labels` 看起来像 `input_ids`：目标不是复制整段输入，而是在每个位置预测“下一个 token”。

## 下一步

继续读 Chapter 1/6 的 `Sequence-to-sequence models`。下一次检查问题：

```text
为什么 summarization 更像 encoder-decoder，而不是单纯 encoder-only 或 decoder-only？
```
