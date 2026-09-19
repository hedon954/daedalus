# HF Course 1/2 基础学习导航

本阶段目标：先完成 Hugging Face LLM Course 第 1、2 章，建立 Transformers 推理链路的基础，再进入买家 Agent baseline 和 LoRA。

不要把本阶段变成“看完课程”。每一节都要落到一个问题：

> 我现在能解释 pipeline 的哪一层？能用一个最小实验验证它吗？

## 为什么先学 1 和 2

当前已经跑通 `pipeline("text-classification")`，但还没有真正理解：

- task 如何选择默认模型。
- tokenizer 如何把文本变成模型输入。
- model 输出是什么。
- postprocess 如何把 logits 变成 label / score。
- 为什么不指定 model 不适合生产或严肃实验。

第 1 章解决“Transformer 能做什么、整体架构是什么”；第 2 章解决“pipeline 背后发生了什么、如何直接使用 model 和 tokenizer”。

## Chapter 1：Transformer Models

只抓四个核心问题：

| 课程内容 | 本 topic 要得到什么 |
| --- | --- |
| Transformers, what can they do? | 知道 pipeline task 是任务接口，不是模型本身 |
| How do Transformers work? | 建立 tokenizer -> model -> head/postprocess 的整体图 |
| Transformer Architectures | 区分 encoder / decoder / encoder-decoder 的大方向 |
| Bias and limitations | 记住模型输出不是事实，后续 eval 必须独立设计 |

不展开：

- 不深挖注意力公式。
- 不追模型排行榜。
- 不提前研究完整训练细节。

## Chapter 2：Using Transformers

这是本阶段主线。

| 课程内容 | 本 topic 要完成的实验 |
| --- | --- |
| Behind the pipeline | 把 `pipeline` 拆成 tokenizer、model、postprocess |
| Models | 显式加载 `AutoModelForSequenceClassification` |
| Tokenizers | 打印 tokens、input_ids、attention_mask |
| Handling multiple sequences | 理解 padding、truncation、batch |
| Putting it all together | 写一个不用 pipeline 的等价推理脚本 |

## 最小实验清单

在 `demo/hugging-face-course-learning` 里完成：

1. 固定 model name，不使用默认模型。
2. 打印 pipeline 组件：
   - task
   - model class
   - tokenizer class
   - config id2label / label2id
3. 写 `behind_pipeline.py`：
   - tokenizer(text)
   - model(**inputs)
   - softmax(logits)
   - 映射 label
4. 做一个 batch 输入实验：
   - 两条英文句子。
   - 打印 padding / attention_mask。
5. 记录一个现象：
   - runtime alias `sentiment-analysis` 可运行，但 static overload 更偏 canonical `text-classification`。

## 完成标准

本阶段完成时，用户应能用自己的话解释：

```text
raw text
  -> tokenizer
  -> input_ids / attention_mask
  -> model
  -> logits
  -> softmax
  -> label / score
```

并能回答：

- pipeline 帮我们隐藏了哪些步骤？
- model 和 tokenizer 为什么必须匹配？
- attention_mask 是为了解决什么问题？
- 为什么实验里要固定 model name？
- 哪些 warning 可以忽略，哪些 warning 会影响实验可靠性？

## 之后再进入

完成 1/2 章基础后，再回到：

- 买家 Agent `{s, r}` baseline。
- 第一批 v0.4 样本审核。
- eval rubric 固化。
- LoRA / PEFT 训练闭环。

## 当前补充：从 Text Generation 到 Causal LM

用户当前学习到 Chapter 1/5 Text generation 小节：

```text
GPT-2's pretraining objective is based entirely on causal language modeling,
predicting the next word in a sequence.
```

这里先不要急着把它理解成“我要马上完整微调一个 GPT-2”。更准确的顺序是：

```text
GPT-2 为什么能生成文本
  -> 因为它按 causal LM 目标预训练
  -> causal LM 的核心是只看左边上下文预测下一个 token
  -> fine-tuning recipe 只是把自己的文本也改造成这种 next-token prediction 训练样本
```

`causal_language_model.py` 是顺着这个概念提前接触到的实现 recipe。阅读时先看：

- [`02-causal-lm-code-reading.md`](02-causal-lm-code-reading.md)
