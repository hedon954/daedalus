# BPE 和 WordPiece 的区别

## 学习现场

用户在复看 Hugging Face LLM Course Chapter 1/5 的 text classification 部分时，先暂停 Trainer 深入，想先把本节实验跑一遍，建立体感。当前问题是：

```text
BPE 和 WordPiece 的区别是什么？
```

## 当前理解

在当前课程上下文里，BPE 和 WordPiece 都是 subword tokenizer：它们不是只能按完整单词切，也不是只能按字符切，而是把文本拆成模型词表里的子词片段。

最重要的直觉差异：

```text
BPE 更像频率驱动的压缩规则。
WordPiece 更像词表驱动的最长匹配，并在训练时更重视片段之间的相对绑定强度。
```

## 训练时和使用时的主宾语

这里的“训练时 / 使用时”首先说的是 tokenizer，不是 Transformer 模型：

```text
tokenizer 训练时：
  主语：BPE / WordPiece 算法
  宾语：课程语料 / 训练语料
  目标：学出 vocab、subword、merge rules 或可匹配的 subword 词表

tokenizer 使用时：
  主语：已经训练好的 tokenizer
  宾语：新的输入文本
  目标：把文本切成 tokens，再映射成 token ids

model 训练时：
  主语：Transformer 模型
  宾语：token ids / attention_mask / labels
  目标：更新模型参数

model 使用时：
  主语：已经训练好的模型
  宾语：token ids
  目标：输出 logits、分类结果或生成下一个 token
```

因此 tokenizer 的输出不是 embedding 本身，而是：

```text
raw text
  -> tokenizer
  -> tokens
  -> token ids
  -> model embedding layer
  -> embedding vectors
```

也就是说，`input_ids` 只是 embedding table 的查表索引；真正的向量表示是在模型内部的 embedding layer 里产生的。

## BPE

BPE 训练时从更小的单位开始，反复合并最常见的相邻 token pair：

```text
("u", "g") 出现最多
-> 合并成 "ug"
-> 继续找下一组最常见 pair
```

因此 BPE 的核心问题是：

```text
哪两个相邻片段一起出现得最多？
```

BPE 的频率不是概率，没有分子分母。它更像一个计数：

```text
pair_freq(pair) = sum(word_freq * pair 在当前 split 中出现的次数)
```

例如语料里有：

```text
("hug", 10), ("pug", 5), ("pun", 12), ("bun", 4), ("hugs", 5)
```

初始按字符切开：

```text
("h" "u" "g", 10)
("p" "u" "g", 5)
("p" "u" "n", 12)
("b" "u" "n", 4)
("h" "u" "g" "s", 5)
```

那么：

```text
("h", "u") = 10 + 5 = 15
("u", "g") = 10 + 5 + 5 = 20
("p", "u") = 5 + 12 = 17
("u", "n") = 12 + 4 = 16
("g", "s") = 5
```

所以第一条 merge rule 是：

```text
("u", "g") -> "ug"
```

`merge rules` 可以理解为 BPE 训练出来的“相邻 token 合并优先级表”。BPE 使用时不是把 subword 合并成一个“目标词汇”，而是从字符、byte 或已有小 token 开始，按训练好的 merge rules 逐步合并，最终得到一串 token：

```text
l o w e r
=> lo w e r
=> low e r
=> low er
=> lower
```

GPT-2 使用 BPE。放回课程里的 text generation 部分，可以理解为：

```text
raw text
  -> BPE tokenizer
  -> token embeddings + position embeddings
  -> decoder blocks
  -> LM head
  -> next-token logits
```

## WordPiece

WordPiece 是 BERT 系模型常见 tokenizer。它的切词结果常用 `##` 表示“这个 token 不是词首”，例如：

```text
"hugs" -> ["hug", "##s"]
```

WordPiece 推理时更像从左到右找词表中最长的合法 subword；训练时不只是看 pair 出现频率，还会考虑 pair 的相对绑定强度。课程中给出的直觉 score 是：

```text
score = freq(pair) / (freq(first) * freq(second))
```

所以高频 pair 不一定最优先合并。如果某个片段本身到处出现，它和另一个片段一起出现并不一定说明它们是稳定组合。

这个 score 的直觉是：

```text
freq(pair) 是这两个片段挨在一起出现多少次。
freq(first) 是左片段自己总共出现多少次。
freq(second) 是右片段自己总共出现多少次。
```

BPE 只问：

```text
这两个片段挨在一起出现得多不多？
```

WordPiece 更像问：

```text
这两个片段是不是更专属于彼此？
```

如果 `first` 和 `second` 各自都到处出现，即使 `pair` 出现次数也不少，score 也会被分母压低。

### WordPiece 使用时的贪心最长匹配

WordPiece 训练阶段可能会学习 merge，但使用阶段通常只保存最终 vocab，不保存 merge rules。使用时的核心算法是：

```text
从当前 word 的最左边开始
-> 找 vocab 里最长的合法 subword
-> 找到后切下来
-> 剩余部分继续找最长合法 subword，后续片段通常带 ## 前缀
-> 如果中途找不到合法 subword，整个 word 变成 [UNK]
```

例如 vocab 里有：

```text
hug
##s
b
##u
##gs
```

那么：

```text
"hugs" -> ["hug", "##s"]
"bugs" -> ["b", "##u", "##gs"]
```

这里确实是贪心算法，并不保证全局最优。如果 vocab 里有：

```text
abc
ab
##cd
```

输入：

```text
abcd
```

贪心最长匹配会先选：

```text
abc
```

剩下的 `d` 如果找不到 `##d`，那么整个词会失败并变成：

```text
["[UNK]"]
```

但如果当初先选 `ab`，其实可以得到：

```text
["ab", "##cd"]
```

所以 WordPiece 的使用阶段不是全局搜索，也不会回溯。它的工程取舍是：训练阶段构造一个足够好的 subword vocab，使用阶段保持确定、快速、简单的最长匹配。

## 和当前 Text Classification 的关系

BERT text classification 的链路可以先记成：

```text
raw text
  -> WordPiece tokenizer
  -> input_ids / attention_mask / token_type_ids
  -> [CLS] final hidden state
  -> classification head
  -> logits
```

这里 tokenizer 的任务是把自然语言变成模型能查 embedding table 的 token ids。分类真正依赖的是 encoder 输出的表示，尤其是 `[CLS]` 位置的 final hidden state。

## 当前结论

- BPE：训练时按最频繁的相邻 pair 学 merge rules，常见于 GPT-2 这类生成模型 tokenizer。
- WordPiece：BERT 系常见，使用时按 vocab 做贪心最长匹配，训练时更重视 pair 的相对绑定强度。
- BPE / WordPiece 都会产出 subword tokens；区别不是“有没有 subword”，而是训练时如何选择新 subword，以及使用时如何把新文本切成 token。
- 当前阶段先能解释“文本为什么会变成 subword ids、token ids 为什么还不是 embeddings，以及 tokenizer 为什么必须和 model 匹配”。

## 来源

- Hugging Face LLM Course Chapter 1/5：GPT-2 使用 BPE，BERT 使用 WordPiece。
- Hugging Face LLM Course Chapter 6：[`Byte-Pair Encoding tokenization`](https://huggingface.co/learn/llm-course/en/chapter6/5) / [`WordPiece tokenization`](https://huggingface.co/learn/llm-course/en/chapter6/6) 机制说明。
