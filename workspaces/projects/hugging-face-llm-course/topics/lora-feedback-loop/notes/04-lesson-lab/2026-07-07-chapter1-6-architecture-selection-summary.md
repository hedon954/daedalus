# Chapter 1/6 架构选择总结

## 场景

用户完成 Hugging Face LLM Course Chapter 1/6 `Transformer Architectures` 的核心归纳：从任务形态反推 encoder-only、decoder-only、encoder-decoder 的选择。

## 用户总结

用户的最终归纳：

> 理解 encoder，生成 decoder，extract 是 encoder，generate new token 是 decoder，如果 generate 需要理解完整的 input，那要么用 decoder 把“读原文”和“生成摘要”都塞进一个续写过程，要么就分成 encoder + decoder 2 个环节，先理解，后生成。

## Agent 校验

这个总结已经能覆盖 Chapter 1/6 当前学习目标。更精炼地说：

```text
encoder-only:
  理解已有输入
  输出 label / token label / span

decoder-only:
  基于已有前文继续生成
  输出新的 token 序列

encoder-decoder:
  先理解完整 source
  再生成新的 target
```

一个关键进步是用户区分了“能做”和“更适合”：decoder-only 可以通过 prompt 把读原文和生成摘要都塞进续写过程；encoder-decoder 则把 source 理解和 target 生成显式拆成两个环节。这个区别足够说明 Chapter 1/6 的三类架构映射已经通过。但 Chapter 1/6 还没有结束，后续仍需读完 attention mechanisms，再进入 Chapter 2 拆 `pipeline` 如何选择 tokenizer、model 和 postprocess。

## 后续提醒

Chapter 1/6 架构映射已经完成。后续仍需提醒用户亲自完成：

- Chapter 1/6 attention mechanisms：理解 full attention 的 O(n^2) 成本，以及 LSH / local attention / axial positional encodings 的取舍。
- Chapter 2 前后手写 `pipeline("text-classification")` 的 tokenizer -> model -> postprocess 等价流程。
- Trainer 复盘时补 batch keys / shape / device，并手动运行 `model(**batch)` 观察 loss/logits。
