# Question Answering Derivation：从字符答案推导 token span

## 本节定位

Question answering 是 Chapter 1/5 task lab sweep 的第四个 lab。它仍然是 encoder-only BERT 类模型，但监督信号不再是整句标签或每个 token 的类别，而是答案在 context 里的起止位置。

本节的核心不是 `Trainer`，而是理解 `preprocess_function` 为什么要把 SQuAD 的字符级答案转成 token 级 span：

```text
输入：question + context
原始目标：answers.text + answers.answer_start，即答案在 context 字符串里的字符区间
tokenizer：把 question/context 拼成一条 token 序列，并记录每个 token 对应的原文字符区间
训练目标：start_positions / end_positions，即答案开始和结束 token 的索引
模型输出：start_logits / end_logits，每个 token 各有一个“适合作为答案起点/终点”的分数
后处理：取 start/end 分数最高的 token span，再 decode 回文本答案
```

## 和前两个分类任务的差异

```text
Sequence classification:
  one sequence -> one label
  logits.shape == [batch_size, num_labels]

Token classification:
  one sequence -> one label per token
  logits.shape == [batch_size, seq_len, num_labels]

Question answering:
  question + context -> two token positions
  start_logits.shape == [batch_size, seq_len]
  end_logits.shape == [batch_size, seq_len]
```

QA 的 head 不是给每个 token 分实体类别，而是给每个 token 两种位置分数：它能不能做答案起点，它能不能做答案终点。

## 为什么有些训练需要 `compute_metrics`

`compute_metrics` 不是训练 loss 的来源。只要 batch 里有模型认识的 label 字段，模型 forward 就能自己算 loss：

```text
Sequence classification: labels -> cross entropy loss
Token classification: labels -> token-level cross entropy loss, 忽略 -100
Question answering: start_positions / end_positions -> start/end loss
```

`Trainer.train()` 优化的是这个 loss，所以很多最小训练脚本不写 `compute_metrics` 也能训练。

`compute_metrics` 的职责是在 `evaluate()` 或训练中的 eval step 里，把模型输出翻译成人类关心的指标：

```text
model outputs + labels -> accuracy / F1 / seqeval / exact match
```

为什么有些任务会写，有些不写，主要看 metric 是否容易从 logits 直接算出来：

| Task | Loss 是否需要 `compute_metrics` | Metric 难度 | 常见做法 |
| --- | --- | --- | --- |
| Text classification | 不需要 | 简单：`argmax(logits)` 对比 label | 常写 `compute_metrics` 算 accuracy/F1 |
| Token classification | 不需要 | 中等：过滤 `-100`，再转 label name | 常写 `compute_metrics` 算 seqeval |
| Causal LM | 不需要 | 通常直接看 eval loss/perplexity | 可以不写，手动用 eval loss 算 perplexity |
| Question answering | 不需要 | 较复杂：start/end logits -> 合法 span -> decode text -> EM/F1 | 最小训练常不写；严肃评估需要 postprocess 再算 SQuAD metrics |

所以 QA 里“不写 `compute_metrics`”不代表没有评估，只代表 `Trainer` 默认最多给你 eval loss。要得到 SQuAD 的 exact match / F1，需要额外把 `start_logits/end_logits` 后处理成文本答案，再和 `answers.text` 比较。

## `preprocess_function` 做了什么

### 1. 清理 question

```python
questions = [q.strip() for q in examples["question"]]
```

这一步只是去掉问题前后的空白，避免无意义空格进入 tokenizer。

### 2. Tokenize question/context pair

```python
inputs = tokenizer(
    questions,
    examples["context"],
    max_length=384,
    truncation="only_second",
    return_offsets_mapping=True,
    padding="max_length",
)
```

这里 tokenizer 的输入是 pair：

```text
sequence 0: question
sequence 1: context
```

关键参数：

- `max_length=384`：固定模型输入长度。
- `truncation="only_second"`：只截断第二段，也就是 context；不要把 question 截掉。
- `return_offsets_mapping=True`：为每个 token 返回它在原始字符串里的字符起止位置。
- `padding="max_length"`：直接 padding 到固定长度，后面 `DefaultDataCollator` 不再额外 padding。

### 3. 取出 offset mapping

```python
offset_mapping = inputs.pop("offset_mapping")
```

`offset_mapping` 是 preprocessing 的中间工具，不直接喂给模型。它回答的问题是：

```text
token i 对应原始字符串里的哪一段字符？
```

例如某个 context token 对应：

```text
offset[i] == (423, 435)
```

表示这个 token 覆盖 context 字符串的 `[423, 435)` 字符区间。

### 4. 原始答案是字符坐标

```python
start_char = answer["answer_start"][0]
end_char = answer["answer_start"][0] + len(answer["text"][0])
```

SQuAD 给的是：

```text
answer_start: 答案在 context 字符串里的起始字符位置
answer.text: 答案文本
```

所以 `end_char` 是根据 `start_char + len(answer_text)` 算出来的。此时监督信号还在“字符坐标系”。

### 5. 找到 context 在 token 序列里的范围

```python
sequence_ids = inputs.sequence_ids(i)
```

`sequence_ids(i)` 会告诉第 `i` 个样本中每个 token 属于哪一段：

```text
None -> special token / padding
0    -> question
1    -> context
```

所以这两段循环是在找 context 的 token 边界：

```python
idx = 0
while sequence_ids[idx] != 1:
    idx += 1
context_start = idx

while sequence_ids[idx] == 1:
    idx += 1
context_end = idx - 1
```

为什么只在 context 里找？因为答案必须来自 context，不能来自 question，也不能来自 `[CLS]` / `[SEP]` / padding。

### 6. 处理答案被截断的情况

```python
if offset[context_start][0] > end_char or offset[context_end][1] < start_char:
    start_positions.append(0)
    end_positions.append(0)
```

如果 context 太长被截断，答案可能已经不在保留下来的 context token 范围里。这时官方示例把标签设成 `(0, 0)`，通常对应 `[CLS]` 位置，表示这个切片里没有可学习的答案 span。

### 7. 字符坐标转 token 坐标

如果答案还在当前 context 范围里，就向右找答案起点 token：

```python
idx = context_start
while idx <= context_end and offset[idx][0] <= start_char:
    idx += 1
start_positions.append(idx - 1)
```

含义是：找到最后一个 `token_start <= answer_start_char` 的 token，它就是答案起点 token。

再向左找答案终点 token：

```python
idx = context_end
while idx >= context_start and offset[idx][1] >= end_char:
    idx -= 1
end_positions.append(idx + 1)
```

含义是：找到第一个 `token_end >= answer_end_char` 的 token，它就是答案终点 token。

最后把这两个 token index 放回模型输入：

```python
inputs["start_positions"] = start_positions
inputs["end_positions"] = end_positions
```

## 当前 Notebook 的坑位

### 1. 错把大多数答案标成 `[CLS]`

判断答案是否在保留下来的 context 窗口内时，应该检查 context 的最后一个 token 是否仍覆盖答案起点：

```python
if offset[context_start][0] > end_char or offset[context_end][1] < start_char:
    start_positions.append(0)
    end_positions.append(0)
```

如果误写成：

```python
offset[context_start][1] < start_char
```

含义就变成“第一个 context token 的结束位置是否早于答案起点”。对绝大多数样本来说，答案不会出现在 context 第一个 token 里，所以这个条件几乎总是 true，训练标签就会被批量写成 `(0, 0)`。模型最后输出 `[CLS]` 并不是随机坏掉，而是在学习你给它的错误监督信号。

### 2. Inference 要加载同一个训练目录

训练时如果使用：

```python
TrainingArguments(output_dir="my_awesome_qa_model", ...)
```

推理时也应该加载同一个目录：

```python
tokenizer = AutoTokenizer.from_pretrained("my_awesome_qa_model")
model = AutoModelForQuestionAnswering.from_pretrained("my_awesome_qa_model")
```

否则你可能以为自己在测刚训练出的模型，实际加载的是另一个旧目录。

### 3. 空输出通常是 `start > end`

最小推理代码里如果直接写：

```python
answer_start_index = outputs.start_logits.argmax()
answer_end_index = outputs.end_logits.argmax()
predict_answer_tokens = inputs.input_ids[0, answer_start_index : answer_end_index + 1]
```

`start_logits.argmax()` 和 `end_logits.argmax()` 是两个独立选择，可能出现 `answer_start_index > answer_end_index`。这时切片为空，`tokenizer.decode(...)` 会得到空字符串，看起来像“什么都没输出”。

严肃 QA 后处理不能只取两个独立 argmax，而要在候选 start/end 中找合法 span：

```text
end >= start
span length 不要过长
span 最好落在 context token 范围内
score = start_logit[start] + end_logit[end]
```

当前 lab 先用 top-k 合法 span 观察即可；后续再补完整 SQuAD EM/F1 postprocess。

### 4. 官方示例输出不是 golden output

Hugging Face task guide 里的 inference 输出只能当作“代码形状示例”，不能当作本地训练的确定性答案。即使代码和数据集名称相同，结果也可能不同：

- `AutoModelForQuestionAnswering.from_pretrained("distilbert/distilbert-base-uncased")` 会在 base DistilBERT 上新建 QA head；`qa_outputs.weight/bias` 是随机初始化，不是一个已经会 QA 的 head。
- `train_test_split(test_size=0.2)` 没有固定 `seed` 时，训练/测试切分可能不同。
- `TrainingArguments` 没有开启 full deterministic；MPS/GPU、库版本、batch 顺序和随机初始化都可能让小数据训练结果漂移。
- 当前只用 `train[:5000]` 小样本训练 3 epoch，泛化到 BLOOM 这条非 SQuAD 风格手写 context 时不稳定是正常的。
- 官方 inference 代码用独立 `argmax(start_logits)` 和 `argmax(end_logits)`，不是完整 QA postprocess；如果 `start > end`，本地会得到空字符串。如果选出合法但过长的 span，也可能得到一串不够精确的答案。

更稳定的学习目标不是复现官网那一句输出，而是观察：

```text
preprocess 后 label span 能 decode 回原始 answers.text
训练后 start/end logits 开始偏向 context 中的数字或名词短语
后处理必须约束合法 span
```

## 必须观察

在 notebook 里不要只跑完整训练。先取一个样本打印：

```python
sample = squad["train"][0]
print(sample["question"])
print(sample["answers"])
print(sample["context"][sample["answers"]["answer_start"][0] : sample["answers"]["answer_start"][0] + len(sample["answers"]["text"][0])])
```

再在 `preprocess_function` 里临时打印一个样本的：

```text
sequence_ids
offset_mapping 前若干项
context_start / context_end
start_char / end_char
start_positions / end_positions
tokenizer.decode(input_ids[start_positions:end_positions+1])
```

验收标准：decode 出来的 token span 应该能还原到 `answers.text[0]`，允许有 tokenizer 空格或大小写差异。

## 来源

- [Hugging Face Transformers question answering task guide](https://huggingface.co/docs/transformers/tasks/question_answering)
