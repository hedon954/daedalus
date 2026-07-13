# Chapter 3：从预训练模型到可诊断的微调闭环

## 这章真正要学什么

Chapter 2 解决的是“如何把输入交给预训练模型并得到输出”，Chapter 3 开始回答：

> 如何用自己的有标签数据改变模型参数，并证明这种改变确实改善了目标任务？

整章使用 BERT + GLUE/MRPC 句对分类作为受约束的设计案例。不要只记住 MRPC recipe；需要抽取的是一条可迁移到其他监督微调任务的主线：

```text
任务契约
-> 原始 dataset 与 split
-> tokenizer / preprocessing
-> collator / DataLoader
-> model forward -> loss
-> backward -> optimizer -> scheduler
-> evaluation metrics
-> learning curves / error diagnosis
-> checkpoint 与可复现证据
```

本章用两种方式实现同一件事：

```text
Trainer API：用声明式配置交给框架编排
Pure PyTorch loop：把训练职责逐项展开
Accelerate：在保留训练循环的同时抽象设备与分布式执行
```

学完后的标准不是“代码跑通”，而是能指出每层的输入、输出、隐藏职责和失败证据。

## 章节地图

| Section | 核心问题 | 应留下的证据 |
| --- | --- | --- |
| 3/1 Introduction | fine-tuning 相比 inference 新增了什么？ | 能画出完整训练闭环 |
| 3/2 Processing the data | raw sample 如何变成形状一致、模型可消费的 batch？ | sample keys、tokenized keys、batch shape、label mapping |
| 3/3 Trainer API | Trainer 替我们编排了哪些职责？ | training/eval loss、metric、checkpoint、关键参数 |
| 3/4 Full training loop | 高层 API 隐藏了哪些 PyTorch 步骤？ | forward、backward、optimizer、scheduler、zero-grad 顺序 |
| 3/4 Accelerate | 如何把设备与分布式细节移出业务训练循环？ | `prepare()` 前后对象、device、实际进程/硬件 |
| 3/5 Learning Curves | 如何区分学到了、过拟合、欠拟合和训练不稳定？ | train/eval curves、诊断结论、下一实验变量 |
| 3/6 Check | 能否把各节重新连成因果链？ | 闭卷复述 |
| 3/7 Quiz | 能否识别监督微调的关键判断？ | 用户自行答题，不归档答案 |

## 3/2：数据处理不是格式转换，而是定义训练问题

### 从 DatasetDict 开始检查

官方示例加载 `glue` / `mrpc`，得到 train、validation、test 三个 split。第一步不要急着 tokenize，而要确认：

```text
每行字段是什么？
哪个字段是输入？
哪个字段是监督信号？
label id 对应什么业务语义？
各 split 是否真的承担 train / selection / final test 的职责？
```

MRPC 的输入是 `sentence1 + sentence2`，label 表示两句话是否语义等价。`features` 中的 `ClassLabel` 才是 `0/1` 到语义名称的事实源，不能凭直觉猜。

### 句对必须作为句对 tokenize

这两种操作不等价：

```python
tokenizer(sentence1)
tokenizer(sentence2)
```

```python
tokenizer(sentence1, sentence2, truncation=True)
```

BERT 句对输入大致形成：

```text
[CLS] sentence1 [SEP] sentence2 [SEP]
```

并可能带有 `token_type_ids`，用于区分两个 segment。不是所有架构都有这个字段；DistilBERT 就可能没有。正确原则不是“永远手写 `token_type_ids`”，而是让匹配的 tokenizer 根据 checkpoint 契约生成模型所需字段。

### `Dataset.map(..., batched=True)` 的作用

```python
def tokenize_function(examples):
    return tokenizer(
        examples["sentence1"],
        examples["sentence2"],
        truncation=True,
    )

tokenized_datasets = raw_datasets.map(tokenize_function, batched=True)
```

这里有三层机制：

1. `map()` 保留 Dataset / DatasetDict 结构，并把 preprocessing 返回的新字段加入各 split。
2. `batched=True` 让 tokenizer 一次处理一批样本，发挥 fast tokenizer 的 Rust 实现与批处理优势。
3. preprocessing 此时只做 truncation，不做全数据集 padding。

不要把 `batched=True` 理解为“已经生成训练 batch”。它只是批量执行预处理；真正的训练 batch 由 DataLoader + collator 在迭代时组成。

### Dynamic padding 为什么放到 collator

若预处理时把所有样本 pad 到整个数据集的最大长度，短样本会产生大量无效 token 计算。动态 padding 推迟到组 batch 时：

```text
dataset 中保留不同长度的 token ids
-> DataLoader 取出本 batch 样本
-> DataCollatorWithPadding
-> 只 pad 到当前 batch 的最大长度
```

因此 batch 中常见形状为：

```text
input_ids:      [batch_size, batch_max_seq_len]
attention_mask: [batch_size, batch_max_seq_len]
token_type_ids: [batch_size, batch_max_seq_len]  # 仅某些模型
labels:         [batch_size]
```

代价与边界：

- 不同 batch 的 sequence length 会变化，这是预期行为。
- 动态 padding 通常减少无效计算，但 TPU 更偏好固定 shape；硬件约束可能改变最优策略。
- 若需要 Tensor Core 友好的维度，可进一步考虑 `pad_to_multiple_of`，但应以实际 profiling 为准。

### 3/2 最小观察实验

先预测：句对 tokenizer 会新增什么字段？8 条不同长度样本经过 collator 后，第二维等于什么？

必须打印：

```python
print(raw_datasets)
print(raw_datasets["train"].features)
print(raw_datasets["train"][0])
print(tokenized_datasets["train"].column_names)

samples = tokenized_datasets["train"][:8]
print([len(x) for x in samples["input_ids"]])
batch = data_collator(samples)
print({k: (v.shape, v.dtype) for k, v in batch.items()})
```

通过标准：能解释 `map`、tokenizer、collator 各自在哪个时间点工作，且不把 preprocessing batch 与 training batch 混为一谈。

## 3/3：Trainer 是训练控制面，不是训练机制本身

`Trainer` 把许多机械职责组合起来：

```text
TrainingArguments
model
train/eval datasets
data collator
processing class
compute_metrics
-> Trainer
-> train / evaluate / predict / save / log
```

### `TrainingArguments` 控制什么

至少要把参数分成几组，而不是把它看成一个巨大参数表：

| 维度 | 典型参数 | 控制的问题 |
| --- | --- | --- |
| 产物 | `output_dir`, save strategy | checkpoint 写到哪里、何时保存 |
| 优化 | learning rate、epochs、weight decay | 参数如何更新、更新多久 |
| batch | per-device batch size、gradient accumulation | 单步显存与有效 batch size |
| evaluation | `eval_strategy`, `eval_steps` | 何时在 validation 上测量 |
| logging | logging steps、report target | 训练过程留下多少可诊断证据 |
| precision | fp16 / bf16 | 速度、显存与数值稳定性 |
| scheduler | scheduler type、warmup | learning rate 如何随 step 改变 |

硬件参数必须以当前设备能力为准。不要因为课程展示 `fp16=True` 就在 MPS、CPU 或不支持的 GPU 上机械照搬。

### 只有 training loss 不等于知道模型效果

只调用 `trainer.train()` 可能只报告训练 loss。若想判断泛化能力，需要同时配置：

```text
eval_dataset
+ eval_strategy
+ compute_metrics
```

`compute_metrics` 接收 predictions 与 label ids。分类任务通常先对 logits 做 `argmax(axis=-1)`，再计算 accuracy / F1。注意：

- loss 是优化目标的连续信号；
- accuracy / F1 是任务层指标；
- 两者不能互相替代；
- MRPC 类别与数据分布决定应关注 accuracy、F1 或两者，不能只选看起来最大的数。

### 每次实验要重新初始化模型

如果复用已经训练过的 model 再调用 `train()`，得到的是续训，不是可比较的新实验。改变关键超参数做 A/B 实验时，应从同一 checkpoint 重新实例化 model，并尽量控制 seed、split 与其他变量。

### Trainer 最小观察实验

先预测：`predict()` 的 logits 和 labels 各是什么 shape？只设置 `eval_dataset` 而没有 `eval_strategy` 与 `compute_metrics` 会看到什么？

必须保留：

```text
训练参数快照
train loss / eval loss
accuracy / F1
runtime / samples per second
checkpoint 路径
模型与 tokenizer 的 checkpoint 名称
```

不要只截终端最后一行；结果必须能回答“训练了什么、怎样训练、效果如何、能否复现”。

## 3/4：手写训练循环是在揭开 Trainer 的抽象

### Trainer 自动做掉的前处理

进入纯 PyTorch 循环后，需要显式处理：

```text
删除模型不接受的字符串列
label -> labels
Dataset 输出格式 -> torch tensors
构造 train/eval DataLoader
训练集 shuffle，验证集不 shuffle
传入 collate_fn 做动态 padding
```

第一批 batch 必须先试跑 model forward：

```python
outputs = model(**batch)
print(outputs.loss)
print(outputs.logits.shape)
```

它是训练前最便宜的契约检查：字段名、shape、label、head 的类别数和 forward 能否闭合，应在长时间训练前暴露。

### 一步训练的因果顺序

```python
model.train()
for batch in train_dataloader:
    batch = {k: v.to(device) for k, v in batch.items()}
    outputs = model(**batch)
    loss = outputs.loss
    loss.backward()
    optimizer.step()
    lr_scheduler.step()
    optimizer.zero_grad()
```

逐步理解：

1. `model.train()` 打开 dropout 等训练行为。
2. forward 产生 logits，并在提供 `labels` 时计算 loss。
3. `backward()` 沿计算图把梯度累积到参数的 `.grad`。
4. `optimizer.step()` 使用当前梯度更新参数。
5. `scheduler.step()` 推进学习率计划。
6. `zero_grad()` 清除本步梯度，否则下一步会继续累积。

“梯度累积”正是有意不在每个 micro-batch 后 step/zero 的策略，因此不能把 `zero_grad()` 当无意义样板代码。

### optimizer 与 scheduler 的关系

AdamW 决定如何把梯度变成参数更新；scheduler 决定每一步使用多大的 learning rate。训练总步数通常是：

```text
num_epochs * len(train_dataloader)
```

但加入 gradient accumulation、drop-last、分布式切分或 resume 后，真实 optimizer steps 可能不再等同于看到的 micro-batch 数。迁移到自己的训练脚本时必须校准“scheduler 的 step 到底对应什么”。

### evaluation loop 的关键切换

```text
model.eval()
torch.no_grad()
forward
logits -> predictions
metric.add_batch(...)
metric.compute()
```

`eval()` 改变 dropout 等模块行为；`no_grad()` 停止构建反向图、减少显存与计算。两者职责不同，都需要。

### Accelerate 抽象了什么

Accelerate 没有消灭训练循环，而是接管设备与分布式相关机械工作：

```python
accelerator = Accelerator()
train_dl, eval_dl, model, optimizer = accelerator.prepare(
    train_dataloader, eval_dataloader, model, optimizer
)

# loss.backward() -> accelerator.backward(loss)
```

核心变化：

- `prepare()` 包装 model、optimizer、dataloader，使它们匹配当前执行环境；
- batch 不再需要手动 `.to(device)`；
- backward 经过 Accelerate，以便正确处理 mixed precision / distributed setup；
- 启动方式与环境配置仍决定实际用了几个进程和什么硬件，不能因为 import 了 Accelerate 就宣称完成分布式训练。

通过标准：能把 Trainer 的一个配置项或行为映射到手写 loop 中的具体对象与语句。

## 3/5：Learning curves 是训练系统的反馈，不是装饰图

### loss 与 accuracy 为什么形状不同

loss 是连续信号。模型即使还没改变最终类别，只要对正确类别的置信度有所改善，loss 就可能下降。

accuracy 是离散决策。只有预测跨过类别边界，正确样本数才改变，所以曲线常呈台阶状。不要因 accuracy 暂时平坦就立即判断“模型完全没学”。

### 诊断矩阵

| 现象 | 更可能的解释 | 下一步只改变一个变量 |
| --- | --- | --- |
| train/eval loss 都下降且接近 | 健康学习并逐渐收敛 | 观察是否已到收益平台 |
| train loss 继续下降，eval loss 上升 | 过拟合 | early stopping、weight decay、dropout、更多/更好数据 |
| train/eval loss 都高且早早平台 | 欠拟合或数据/优化问题 | 检查数据，再尝试训练更久、容量或 learning rate |
| loss 高频震荡、出现尖峰 | learning rate、batch、梯度或脏数据问题 | 先查 batch/data，再降低 LR、增大 batch、clip grad |
| loss 下降但业务指标不升 | 优化目标与决策指标未同步，或阈值/类别问题 | 看 confusion matrix、per-class metric 与样本错误 |
| validation 很好但线上失败 | split 泄漏、分布偏移或指标不代表产品目标 | 重做 split 与业务 eval set |

### 对官网建议保持批判性

课程给的是诊断候选，不是单一因果答案。同一种曲线可能由多种原因造成：

- “欠拟合”可能是模型太小，也可能是 label 错、preprocessing 错或 learning rate 不合适；
- “震荡”可能来自学习率，也可能来自小 batch、异常样本、数值精度或日志粒度；
- 增大模型、训练更久、增加 batch 都有成本，不应一次全改。

官网 erratic-curves 示例的文字说“降低 learning rate”，但展示的 diff 从 `1e-5` 改成 `1e-4`，数值方向其实是升高。学习时应遵循机制与实验设计，不把示例代码当 golden output：若诊断是 LR 过高，应设计降低 LR 的单变量对照实验。

### 最小实验纪律

每次调参记录：

```text
hypothesis：我认为哪条曲线说明了什么？
single change：只改变哪个变量？
expected signal：若判断正确，哪条曲线应该怎样变化？
observed result：实际曲线和最终指标如何？
decision：保留、回滚，还是提出下一个假设？
```

这一步是从“会训练”走向“会做训练实验”的分界。

## Trainer、手写循环与 Accelerate 怎么选

| 选择 | 适合 | 优势 | 风险 |
| --- | --- | --- | --- |
| Trainer | 标准监督任务、快速可靠 baseline | 少样板、内置 eval/save/log/distributed | 容易不知道默认值和隐藏行为 |
| Pure PyTorch | 非标准 loss、特殊 batch/step、机制学习 | 控制强、每一步可观察 | 易漏 eval mode、zero-grad、device、checkpoint 等细节 |
| Accelerate + loop | 需要自定义 loop 且要跨设备/分布式 | 保留控制力，减少硬件样板 | 仍需理解分布式 metric、进程与保存语义 |

对当前 `lora-feedback-loop` topic，推荐顺序：

```text
先用 Trainer 建可重复 baseline
-> 用最小手写 loop 解释隐藏职责
-> 回到 Trainer / PEFT 做 LoRA 实验
-> 只有需要自定义训练行为时才扩大手写 loop
```

这不是“高层 API 不专业”。专业判断是知道抽象隐藏了什么、何时足够、何时必须打开。

## 迁移到 LoRA / suggestion-next-action 任务

Chapter 3 的 BERT 分类只教监督微调骨架。迁移到当前业务任务时，以下部分可以忠实复用：

```text
明确 train/validation/test split
可批处理的 preprocessing
collator 负责 batch shape
训练参数与 checkpoint 产物化
train/eval 分离
learning curves 与单变量实验
baseline -> changed model -> compare
```

以下部分不能机械照搬：

- MRPC 是二分类，suggestion next action 是结构化生成；model head、labels shape、loss 和 metric 都不同。
- `accuracy/F1` 不能衡量建议是否推动购买、安全、相关且语义完整；需要业务 rubric、格式合规率和人工/模型评审的一致性校准。
- 全参数 BERT fine-tuning 不等于 LoRA；LoRA 冻结大部分原参数，只训练低秩 adapter，但 optimizer/backward/eval 的训练闭环仍然成立。
- 动态 padding、precision、batch size 和 gradient accumulation 的选择必须按生成模型长度分布与实际硬件重新 profiling。

## 本章建议的学习顺序

当前只推进一个 lesson：先做 3/2 数据处理观察，不同时启动 Trainer 与 full loop。

```text
Checkpoint A — Data contract
能解释 MRPC raw row、label mapping、sentence-pair tokenization、dynamic padding

Checkpoint B — Trainer baseline
能跑通训练/eval，并保存参数、metric、checkpoint 证据

Checkpoint C — Full loop derivation
能闭卷写出 forward -> backward -> optimizer -> scheduler -> zero-grad

Checkpoint D — Learning curve diagnosis
能基于 train/eval 曲线提出单变量实验，而不是拍脑袋调参

Checkpoint E — Transfer
能指出迁移到 LoRA 生成任务时哪些环节不变、哪些任务契约必须重做
```

## Chapter 3 通过标准

不看教程，能够回答：

1. `Dataset.map(batched=True)` 与 DataLoader batch 有什么区别？
2. 为什么 padding 通常推迟到 collator？什么时候可能不这样做？
3. tokenizer 为什么可能返回或不返回 `token_type_ids`？
4. Trainer 替你完成了哪些数据、优化、评估、保存与设备职责？
5. `loss.backward()`、`optimizer.step()`、`scheduler.step()`、`zero_grad()` 的顺序和作用是什么？
6. `model.eval()` 与 `torch.no_grad()` 有什么区别？
7. Accelerate 的 `prepare()` 与 `backward()` 抽象了什么，又没有证明什么？
8. 为什么 loss 下降时 accuracy 可能不变？
9. 如何从曲线区分过拟合、欠拟合与训练不稳定？
10. 从 MRPC/BERT 迁移到 LoRA 结构化生成时，哪些机制保留，哪些契约必须重建？

## 来源

- [Chapter 3/1: Introduction](https://huggingface.co/learn/llm-course/en/chapter3/1)
- [Chapter 3/2: Processing the data](https://huggingface.co/learn/llm-course/en/chapter3/2)
- [Chapter 3/3: Fine-tuning a model with the Trainer API](https://huggingface.co/learn/llm-course/en/chapter3/3)
- [Chapter 3/4: A full training loop](https://huggingface.co/learn/llm-course/en/chapter3/4)
- [Chapter 3/5: Understanding Learning Curves](https://huggingface.co/learn/llm-course/en/chapter3/5)
- [Chapter 3/6: Fine-tuning, Check!](https://huggingface.co/learn/llm-course/en/chapter3/6)
- [Chapter 3/7: End-of-chapter Certificate](https://huggingface.co/learn/llm-course/en/chapter3/7)
