# Chapter 1/6 Transformer Architectures：从任务反推架构

## 学习现场

当前不补 Translation / ASR / Image classification。先用已经完成的文本任务观察，读 Hugging Face LLM Course Chapter 1/6：

- Causal LM / text generation
- Text classification
- Token classification
- Extractive question answering
- Summarization

本节目标不是背 `encoder-only`、`decoder-only`、`encoder-decoder` 三个名词，而是看到一个任务时能反推：

```text
这个任务需要理解整段输入？
还是需要按顺序继续生成？
还是需要把一个输入序列转换成另一个输出序列？
```

## 先读出的三条主线

### Encoder-only

阅读时抓一个问题：模型能不能同时看见输入中的左右上下文？

如果任务主要是理解现有输入，并在输入之上做分类、标注或抽取，优先映射到 encoder-only：

- Text classification：整句 -> 一个标签。
- Token classification：整句 -> 每个 token 一个标签。
- Extractive QA：question + context -> context 中的 start/end span。

观察锚点：

```text
input_ids / attention_mask
-> encoder contextual representations
-> task head
-> logits / span positions
```

### Decoder-only

阅读时抓一个问题：模型是不是只能看见当前位置之前的 token，然后预测下一个 token？

如果任务主要是继续生成文本，优先映射到 decoder-only：

- Causal LM / text generation：前文 -> next token。

观察锚点：

```text
input_ids
-> causal / masked self-attention
-> language modeling head
-> next-token logits
-> generate
```

### Encoder-decoder

阅读时抓一个问题：输入和输出是不是两个序列，而且输出要依赖完整输入？

如果任务是把一个输入文本转换成另一个输出文本，优先映射到 encoder-decoder：

- Summarization：长文 -> 摘要。
- Translation：源语言句子 -> 目标语言句子，当前已跳过/延后。

观察锚点：

```text
source input_ids
-> encoder reads source
-> decoder autoregressively generates target
-> generated ids
-> decoded text
```

### 能做和更适合不是一回事

架构分类不是能力绝对边界。现代模型经常能跨架构完成任务，但课程里的三分类是在给默认任务接口和工程选择建立直觉。

以 summarization 为例：

```text
extractive summary:
  从原文里挑句子 / 片段
  -> encoder-only 可以做，因为它主要是理解和选择

abstractive summary:
  读完整原文后生成新摘要
  -> encoder-decoder 最自然，因为 encoder 负责读完整输入，decoder 负责生成输出

prompted summary:
  把原文和“请总结”放进 prompt，让模型续写
  -> decoder-only 也能做，尤其是现代 LLM
```

所以更准确的判断不是“别的架构不能做”，而是：

```text
这个任务最自然的接口是什么？
输出是标签 / span，还是一个新的 token 序列？
如果要生成新序列，它是否强依赖完整输入？
```

## 读的时候做一个小判断表

| 任务 | 输入 | 输出 | 更像哪类架构 | 判断理由 |
| --- | --- | --- | --- | --- |
| Causal LM | 前文 tokens | 下一个 token / 后续文本 | decoder-only | 只依赖前文继续生成 |
| Text classification | 一段文本 | 一个类别 | encoder-only | 需要理解完整输入后分类 |
| Token classification | 一段 tokens | 每个 token 的类别 | encoder-only | 每个 token 的判断依赖整句上下文 |
| Extractive QA | question + context | context 中答案 span | encoder-only | 在完整 context 里定位答案 |
| Summarization | 长文本 | 短生成文本 | encoder-decoder | 先理解完整输入，再生成新序列 |

## 本节不要急着做的事

- 不补 Translation / ASR / Image classification，除非后续明确要补 seq2seq 或多模态直觉。
- 不把 Chapter 2 pipeline internals 提前做完；这节还要读 attention mechanisms。
- 不把 Agent 解释写成用户 notes。notes 要等用户读完、复述或运行观察后再记。

## Attention mechanisms 阅读入口

Chapter 1/6 后半段还没有结束。官方继续讲的是：标准 attention 对长文本很贵，所以出现了一些更高效的 attention / position encoding 变体。

先抓住一个公式级直觉：

```text
full attention:
  每个 token 都看每个 token
  attention matrix 约是 seq_len x seq_len
  成本随长度近似 O(n^2) 增长
```

这解释了为什么长文本会变贵：长度翻倍，attention 矩阵不是翻倍，而是大约变成四倍。

接着读三个变体时，不需要记细节公式，先问：

```text
它到底减少了哪些 token-to-token 连接？
它保留了什么能力？
它牺牲了什么全局信息？
```

当前最小理解目标：

| 机制 | 先抓住什么 | 要问的取舍 |
| --- | --- | --- |
| LSH attention | 只让相近的 query/key 互相注意 | 怎么用近似检索减少全连接计算？会不会漏掉重要远距离关系？ |
| Local attention | 主要看附近窗口，少数 token 可有 global attention | 局部窗口为什么够用？什么时候需要全局 token？ |
| Axial positional encodings | 把很长的位置编码矩阵拆成更小的因子 | 这是省位置编码参数/内存，不是直接改变 token 注意谁 |

### LSH attention 慢动作解释

先不要从 `hash function` 这个词开始理解。从 full attention 的浪费开始：

```text
full attention:
  对每个 query q
  都拿它和所有 key k 做相似度比较
  得到一整行 QK^T 分数
  softmax 后，大多数很小的分数几乎没有贡献
```

LSH attention 的想法是：

```text
既然最后真正有用的是“和 q 最像的少数 k”
那就不要把 q 和所有 k 都比一遍
先用 locality-sensitive hashing 把相似向量分到同一桶
然后只在同一桶里做 attention
```

这里的 “close” 不是 token 在句子里的位置近，而是向量空间里内容相似。它和 local attention 的区别很重要：

```text
local attention:
  位置近，所以互相看

LSH attention:
  向量相似，所以互相看
```

一个粗糙例子：

```text
full attention:
  q1 要和 k1, k2, k3, k4, k5, k6, k7, k8 都比较

LSH attention:
  hash(q1) -> bucket A
  bucket A 里只有 k3, k7
  q1 只和 k3, k7 比较
```

这不是精确搜索，而是近似搜索。哈希可能把真正相关的 token 分错桶，所以实践里会用多轮 hash，再把结果合起来，降低漏掉重要关系的概率。

最小记忆：

```text
full attention = 每个 token 问所有 token
LSH attention = 先按相似度粗分组，只问同组 token
收益 = 少算很多 pair
代价 = 近似，有可能漏掉本该看的远处信息
```

### Local attention 慢动作解释

Local attention 的起点比 LSH 更直观：

```text
很多语言理解任务里
一个 token 最常依赖的不是整篇文章所有 token
而是它附近的一小段上下文
```

所以 local attention 不再让每个 token 看全局，而是只看附近窗口：

```text
full attention:
  token i 可以看 1..n 的所有 token

local attention:
  token i 只看 i-window .. i+window 的附近 token
```

一个粗糙例子，假设窗口大小是 2：

```text
token 5 在 full attention 里看：
  1, 2, 3, 4, 5, 6, 7, 8, 9

token 5 在 local attention 里只看：
  3, 4, 5, 6, 7
```

它减少计算的方式不是“哈希找相似内容”，而是“砍掉远距离连接”：

```text
full attention:
  每个 token 看 n 个 token
  总成本约 n x n

local attention:
  每个 token 看固定窗口 w 个 token
  总成本约 n x w
```

如果 `w` 比 `n` 小很多，计算就会轻很多。

关键取舍是：局部关系保留得好，远距离依赖容易丢。

```text
适合：
  主要依赖附近上下文的任务
  很长文本里需要降低成本的场景

风险：
  开头的信息可能影响结尾
  跨段落、跨章节的关系可能被窗口切断
```

所以一些模型会加少量 `global attention`：让特定 token 能看全局，或者被全局看见。

```text
local attention:
  大多数 token 只看附近

local + global attention:
  大多数 token 看附近
  少数重要 token 负责跨远距离传递信息
```

这些 global token 通常不是模型“自己发现”的，而是由模型设计或任务预处理指定的。常见确定方式：

```text
1. 特殊汇总 token
   例如 [CLS] / <s> / BOS
   让它看全局，用来汇总整段输入

2. 任务关键位置
   例如 QA 里的 question tokens
   让问题 token 看全局，帮助它们从长 context 中找答案

3. 结构边界 token
   例如段落标题、分隔符、章节开头
   让这些 token 成为跨段落信息中转站

4. 固定间隔的中转 token
   每隔一段放一个 global token
   像长文本里的路由节点，让远距离信息能分段传过去

5. 任务规则指定
   比如分类任务让 [CLS] 全局，检索任务让 query 部分全局
```

所以 `重要` 不是抽象玄学，而是由任务接口决定：

```text
谁需要整篇信息？
谁负责输出最终判断？
谁能把远处信息传给附近窗口？
```

最小记忆：

```text
local attention = 只看附近窗口
收益 = 把 O(n^2) 近似降到 O(n x window)
代价 = 远距离信息可能断掉
补救 = 给少数 token global attention
```

### Axial positional encodings 慢动作解释

先把它和前两个机制分开：

```text
LSH attention / local attention:
  改的是 token 之间“谁看谁”

axial positional encodings:
  改的是“位置编号怎么表示”
```

Transformer 本身不知道 token 顺序，所以要给每个位置一个位置向量。最直接的 learned positional embedding 是：

```text
position 0 -> 一个 hidden_size 维向量
position 1 -> 一个 hidden_size 维向量
...
position n -> 一个 hidden_size 维向量
```

如果最大长度很大，这张表会变得很大：

```text
max_length x hidden_size
```

例如：

```text
4096 positions x 512 dims
```

Axial positional encodings 的想法是：不要把 4096 个位置当成一条超长的一维表，而是把它拆成多个轴。

比如 4096 可以拆成：

```text
64 x 64 = 4096
```

于是一个一维位置可以改写成二维坐标：

```text
position 0    -> (row 0, col 0)
position 1    -> (row 0, col 1)
...
position 65   -> (row 1, col 1)
...
```

然后分别给 row 和 col 学位置向量，再拼起来或加起来：

```text
position embedding(position)
  = row_embedding[row]
  +/concat col_embedding[col]
```

参数量直觉从：

```text
4096 x hidden_size
```

变成类似：

```text
64 x dim_row + 64 x dim_col
```

也就是用两个短轴的位置表组合出很多长序列位置。

一个粗糙类比：

```text
普通位置编码：
  给每个门牌号单独做一张身份证

axial 位置编码：
  用“楼层号 + 房间号”组合出门牌号
```

它的重点不是减少 attention pair，而是让长序列的位置表示更省参数/内存。

最小记忆：

```text
axial positional encodings = 把长的一维位置编号拆成多个轴
收益 = 更省位置 embedding 参数/内存，适合长序列
代价 = 位置表达被拆成组合形式，设计上要选好 axial shape / dims
```

## 后续提醒队列

这些事项已经写入 `.daedalus/todo.md` 的 `Reminder Queue`，恢复学习时要提醒用户做：

- Chapter 1/6 架构映射：把已做过的 Causal LM / classification / QA / summarization 映射到三类架构。
- Chapter 1/6 attention mechanisms：理解 full attention 的 O(n^2) 成本，以及 LSH / local attention / axial positional encodings 的取舍。
- Chapter 2 前后 pipeline 拆解：手写 `pipeline("text-classification")` 的 tokenizer -> model -> postprocess 等价流程。
- Trainer 机制观察：打印 batch keys / shape / device，并手动运行 `model(**batch)` 查看 loss/logits。

## 通过标准

读完 Chapter 1/6 后，用户至少能不用教程回答：

```text
1. 为什么 classification / token classification / extractive QA 更像 encoder-only？
2. 为什么 Causal LM 更像 decoder-only？
3. 为什么 summarization 更像 encoder-decoder？
4. 如果看到一个新任务，我先问哪三个问题来选架构？
```

## 来源

- [Hugging Face LLM Course Chapter 1/6: Transformer Architectures](https://huggingface.co/learn/llm-course/en/chapter1/6)
