# Chapter 1/8 Deep Dive Into Text Generation Inference With LLMs

## 学习现场

用户已进入 Hugging Face LLM Course Chapter 1/8 `Deep dive into Text Generation Inference with LLMs`。本节不是继续比较 task head，而是把前面学过的 decoder-only / causal LM / attention 连接到 LLM 真实推理过程。

官方页面主线：

- LLM inference 基础：从 prompt 生成 response。
- attention / context length：模型一次能看多少 token，以及长上下文为什么贵。
- prompting：输入文本如何引导生成。
- prefill / decode：推理分成读 prompt 和逐 token 生成两阶段。
- sampling controls：temperature、top-p、top-k、repetition penalties、length controls、beam search。
- performance / KV cache：TTFT、TPOT、throughput、VRAM，以及 KV cache 为什么重要。

## 先抓住一条总链路

```text
raw prompt
-> tokenizer
-> prefill：一次性处理 prompt，建立上下文表示 / KV cache
-> decode：每次生成一个 token
-> sampling：从 logits 变成下一个 token
-> append token
-> repeat until stop
```

本节最重要的分界是：

```text
prefill:
  读完整 prompt
  更影响 first token 前的等待时间

decode:
  逐 token 生成
  更影响后续每个 token 的速度
```

## 阅读检查点

读本节时每一段都回到这几个问题：

1. 这个概念影响的是 prompt 处理，还是逐 token 生成？
2. 这个参数控制的是质量、随机性、长度，还是成本？
3. 这个优化是在省重复计算，还是在牺牲一部分搜索空间？

## 必须理解的性能词

| 指标 | 先这样理解 | 常见影响因素 |
| --- | --- | --- |
| TTFT | Time To First Token：第一个 token 出来前等多久 | prompt 长度、prefill 成本、排队 |
| TPOT | Time Per Output Token：后续每个 token 多快 | decode 成本、KV cache、batching |
| Throughput | 单位时间能处理多少请求 / token | batching、硬件、模型大小 |
| VRAM | 推理需要多少显存 | 模型权重、KV cache、context length、batch size |

## KV Cache 的最小直觉

decoder-only 模型生成第 `t+1` 个 token 时，需要参考之前 token 的 key/value。如果每次都从头重算全部历史，会浪费很多。

KV cache 的想法：

```text
已经算过的历史 token 的 key/value
-> 存起来
-> 后续 decode 直接复用
```

收益：

```text
少重复计算，后续 token 更快
```

代价：

```text
要存历史 token 的 key/value，context 越长、batch 越大，显存越高
```

## 本节不要急着做的事

- 不提前进入 Chapter 2 `Using Transformers`。
- 不把 TGI 部署工程展开成生产配置；当前只读 inference 机制。
- 不把 Agent 解释写成用户 notes。notes 等用户复述或实验观察后再写。

## 通过标准

读完本节后，用户至少能不用教程解释：

```text
1. LLM inference 为什么是逐 token 生成？
2. prefill 和 decode 分别做什么？
3. temperature / top-p / top-k 分别怎么影响 token selection？
4. TTFT / TPOT / throughput / VRAM 分别衡量什么？
5. KV cache 为什么加速 decode，但会增加显存占用？
```

## 来源

- [Hugging Face LLM Course Chapter 1/8: Deep dive into Text Generation Inference with LLMs](https://huggingface.co/learn/llm-course/en/chapter1/8)
