# Chapter 1/6 Encoder-only 复述

## 场景

用户开始阅读 Hugging Face LLM Course Chapter 1/6 `Transformer Architectures`，当前检查点是：为什么 text classification、token classification、extractive QA 更适合映射到 encoder-only。

## 用户复述

用户的理解：

> text classification、token classification、extractive QA 都是基于完整的输入文本，在里面去找信息，不管是情感识别、文本分类、实体提取、QA 提取，都是要看到完整的上下文，基于对完整上下文的综合理解，才来做的。

## Agent 校验

这个复述抓住了 encoder-only 的核心：任务目标不是继续生成新文本，而是在已有输入中做理解、分类、标注或抽取。因此关键机制不是“模型名字叫 BERT”，而是：

```text
完整输入
-> 双向上下文理解
-> task-specific head
-> label / token labels / answer span
```

## 下一步

继续读 Chapter 1/6 的 `Decoder models`。下一次检查问题：

```text
为什么 Causal LM / text generation 更像 decoder-only，而不是 encoder-only？
```
