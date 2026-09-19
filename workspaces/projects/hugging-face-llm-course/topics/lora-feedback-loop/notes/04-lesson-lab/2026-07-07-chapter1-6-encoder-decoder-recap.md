# Chapter 1/6 Encoder-decoder 复述

## 场景

用户继续阅读 Hugging Face LLM Course Chapter 1/6 `Transformer Architectures`，当前检查点是：为什么 summarization 更适合映射到 encoder-decoder，同时辨析 encoder-only / decoder-only 是否也能做。

## 用户复述

用户的理解：

> summarization 是先全部理解，然后再基于全部理解进行 text generation。不过其实，我觉得 encoder-only 或 decoder-only 也可以做呀。encoder 本身就是可以通读原文然后提取，而 decoder 可以基于 input 进行 text generation 出 summarization

## Agent 校验

这个复述抓住了 encoder-decoder 的核心，也提出了一个重要边界：架构分类不是能力绝对边界，而是任务接口和归纳偏置。

对于 summarization，需要区分两类任务形态：

```text
Extractive summarization:
  从原文中挑句子 / 片段
  -> encoder-only 可以做，因为它主要是理解和选择

Abstractive summarization:
  读完整原文后生成一段新的摘要
  -> encoder-decoder 更自然，因为它把“读输入”和“生成输出”分成两个组件
```

Decoder-only 也可以做 summarization，尤其是现代 LLM：把原文和指令一起放进 prompt，然后继续生成摘要。但它的默认训练目标是 next-token prediction，长输入理解、源文对齐和输出控制都依赖 prompt、上下文长度和训练数据；它能做，不等于这个任务最自然地由 decoder-only 表达。

## 机制归纳

```text
encoder-only:
  适合理解 / 分类 / 标注 / 抽取
  可以做 extractive summary

decoder-only:
  适合续写 / 生成
  可以通过 prompt 做 abstractive summary

encoder-decoder:
  适合 source -> target 的转换
  对 summarization / translation 这类“读完整输入再生成新序列”的任务最贴合
```

## 下一步

完成 Chapter 1/6 的架构选择总结：

```text
看到一个新任务时，先问：
1. 是理解已有输入，还是生成新文本？
2. 输出是标签 / span，还是新 token 序列？
3. 如果是新序列，它是否强依赖完整输入？
```
