# Sequence Classification：从任务契约反推代码

## 学习现场

用户已完成 `transformer-work/sequence_classification.ipynb`，大致理解官方 recipe 的流程，但仍觉得整体印象模糊，只能按教程抄，无法自己推导。

本 guide 的目标不是再解释一遍每行 API，而是把 sequence classification 变成一套可反推的任务契约：

```text
text
  -> tokenizer
  -> dynamic padding batch
  -> sequence classification model
  -> logits
  -> label / metric
```

## 核心诊断

现在的卡点不是“代码没跑通”，而是还没有建立这条推导链：

```text
任务需要什么输出？
-> 模型 forward 必须产出什么？
-> forward 需要什么输入 tensor？
-> batch 怎么构造？
-> 单条样本应该先怎么 preprocess？
-> 原始 dataset 需要提供哪些字段？
```

抄教程是正向执行：

```text
load dataset -> tokenizer -> collator -> metric -> model -> Trainer
```

自己推导要反过来：

```text
classification objective -> logits/loss -> model inputs -> batch -> tokenized sample -> raw data
```

## 任务契约

Sequence classification 的任务契约是：

```text
输入：一段文本
输出：一个类别
```

在 IMDb 实验中：

```text
输入：movie review text
输出：0/1 sentiment label
```

所以模型最终不需要为每个 token 都输出一个类别，也不需要生成下一段文本。它只需要对整段 sequence 输出一个分类分布：

```text
logits.shape == [batch_size, num_labels]
```

IMDb 是二分类，因此：

```text
num_labels = 2
logits.shape == [batch_size, 2]
```

## 从输出反推每个对象

### 1. 为什么需要 `id2label` / `label2id`

模型内部只认识数字类别：

```text
0, 1
```

人需要读懂语义：

```text
NEGATIVE, POSITIVE
```

所以需要：

```text
id2label = {0: "NEGATIVE", 1: "POSITIVE"}
label2id = {"NEGATIVE": 0, "POSITIVE": 1}
```

它们不负责训练算法，只负责让模型 config、推理输出和 Hub artifact 带上可读标签语义。

### 2. 为什么需要 `AutoModelForSequenceClassification`

base DistilBERT 只负责把 token 序列编码成 hidden states。分类任务还需要一个 classification head，把整段文本的表示变成类别 logits。

因此要用：

```python
AutoModelForSequenceClassification.from_pretrained(
    checkpoint,
    num_labels=2,
    id2label=id2label,
    label2id=label2id,
)
```

它的 forward 契约可以先记成：

```text
input_ids + attention_mask + labels
  -> loss + logits
```

其中：

```text
labels.shape == [batch_size]
logits.shape == [batch_size, num_labels]
loss 是 batch 上的 classification loss
```

### 3. 为什么 tokenizer 只做 `truncation=True`

原始文本是字符串，模型不能直接吃字符串。tokenizer 负责：

```text
text -> input_ids / attention_mask
```

这里使用：

```python
tokenizer(examples["text"], truncation=True)
```

`truncation=True` 是因为 DistilBERT 有最大输入长度。超过模型上限的文本必须截断，否则模型无法处理。

这里暂时不在 preprocess 阶段做全量 padding，因为不同 review 长度差异很大。过早 padding 到最大长度会浪费大量计算。

### 4. 为什么需要 `DataCollatorWithPadding`

Dataset 中每条样本 token 长度不同：

```text
sample A: 42 tokens
sample B: 183 tokens
sample C: 17 tokens
```

但模型 batch 需要规整 tensor：

```text
input_ids.shape == [batch_size, seq_len]
attention_mask.shape == [batch_size, seq_len]
```

所以 collator 在“组成 batch 的一刻”动态 padding：

```text
当前 batch 最长是 183
-> 这个 batch 内全部 pad 到 183
```

这比把整个 dataset 都 pad 到模型最大长度更省显存和计算。

### 5. 为什么需要 `compute_metrics`

模型 forward 给的是 logits，不是最终 accuracy。

所以评测要把：

```text
logits -> argmax -> predicted label id
```

再和真实 label 比较：

```python
predictions = np.argmax(predictions, axis=1)
accuracy.compute(predictions=predictions, references=labels)
```

如果 logits shape 是 `[batch_size, 2]`，`axis=1` 的含义就是在两个类别维度上取分数最高的类。

### 6. 为什么最后交给 `Trainer`

到这里，组件已经齐了：

```text
model：知道怎么 forward、loss、save
training_args：知道 batch size、lr、epoch、eval/save 策略
train/eval dataset：提供样本
data_collator：把样本组 batch
compute_metrics：把 logits 转 metric
processing_class/tokenizer：保存和推理时需要同一套 tokenizer
```

`Trainer` 的职责不是定义任务本身，而是把训练循环接起来：

```text
sample -> collator -> batch -> model forward -> loss
-> backward -> optimizer step -> eval -> save / push
```

## `TrainingArguments` 是训练控制面板

`TrainingArguments` 不定义模型结构，也不定义样本怎么 tokenize。它更像 Trainer 的训练控制面板，回答这些问题：

```text
训练多久？
每次吃多少数据？
学习率多大？
什么时候评估？
什么时候保存？
要不要把模型推到 Hub？
```

当前官方 recipe 使用：

```python
training_args = TrainingArguments(
    output_dir="my_awesome_imdb_beat-model",
    learning_rate=2e-5,
    per_device_train_batch_size=16,
    per_device_eval_batch_size=16,
    num_train_epochs=2,
    weight_decay=0.01,
    eval_strategy="epoch",
    save_strategy="epoch",
    load_best_model_at_end=True,
    push_to_hub=True,
)
```

### 参数含义

`output_dir` 是本地输出目录。checkpoint、最终模型和训练状态会写到这里；如果 `push_to_hub=True`，它还会和 Hugging Face Hub repo 发生关联。

`learning_rate=2e-5` 是优化器更新参数时的步长。fine-tuning BERT / DistilBERT 这类预训练模型时，常见量级是 `1e-5` 到 `5e-5`。太大可能破坏预训练能力，太小会让学习变慢。

`per_device_train_batch_size=16` 表示每个设备每个 training step 处理 16 条训练样本。单机 M 芯片上，可以先近似理解为：

```text
每 step 训练 16 条 review
```

`per_device_eval_batch_size=16` 表示评估时每个 batch 也是 16 条。评估没有 backward，通常比训练便宜，但完整扫一遍 test set 仍然会花时间。

`num_train_epochs=2` 表示完整扫训练集 2 遍。IMDb train 约 25000 条样本，如果 batch size 是 16：

```text
steps_per_epoch = ceil(25000 / 16) ≈ 1563
total_steps ≈ 1563 * 2 = 3126
```

所以官方 recipe 对学习实验来说偏重；它更像一个完整示例，不是最小观察 run。

`weight_decay=0.01` 是正则化，用来抑制权重过大，降低过拟合风险。它主要服务于泛化稳定性，不是加速参数。

`eval_strategy="epoch"` 表示每个 epoch 结束后完整评估一次。IMDb test 约 25000 条样本，所以每次 eval 也要扫很多 batch。

`save_strategy="epoch"` 表示每个 epoch 结束后保存 checkpoint。保存本身有 IO 成本；如果结合 Hub push，还可能带来上传成本。

`load_best_model_at_end=True` 表示训练结束后加载评估表现最好的 checkpoint，而不是最后一步的模型。正式实验最好显式补上：

```python
metric_for_best_model="accuracy"
```

这样 best model 的选择标准更清楚。

`push_to_hub=True` 表示训练产物会推到 Hugging Face Hub。学习阶段建议先关闭，训练完成、确认结果后再手动：

```python
trainer.push_to_hub()
```

这样可以把训练耗时和上传耗时分开观察。

### 训练量怎么估算

训练慢不只由样本数决定，还和序列长度、训练步数、模型规模有关：

```text
训练成本 ≈ steps * batch_size * avg_sequence_length * model_size
```

当前 recipe 慢，主要是这些因素叠加：

```text
完整 IMDb train：约 25000 条
完整 IMDb test：约 25000 条
2 epochs：约 3126 个训练 step
每个 epoch eval/save/push：额外评估、保存、上传成本
IMDb review 较长：很多样本可能接近 512 tokens
```

可调训练量有三层：

```text
样本数：select(range(...))
训练步数：max_steps / num_train_epochs
序列长度：max_length
```

### 推荐一：机制观察 run

目标是快速看懂链路，不追求最终效果。

```python
def preprocess_function(examples):
    return tokenizer(examples["text"], truncation=True, max_length=256)

tokenized_imdb = imdb.map(preprocess_function, batched=True)

small_train = tokenized_imdb["train"].shuffle(seed=42).select(range(1000))
small_eval = tokenized_imdb["test"].shuffle(seed=42).select(range(300))

training_args = TrainingArguments(
    output_dir="tmp-imdb-debug",
    learning_rate=2e-5,
    per_device_train_batch_size=16,
    per_device_eval_batch_size=32,
    max_steps=50,
    eval_strategy="steps",
    eval_steps=25,
    save_strategy="no",
    load_best_model_at_end=False,
    push_to_hub=False,
    logging_steps=10,
    dataloader_pin_memory=False,
    report_to="none",
)
```

这里最关键的是：

```python
max_steps=50
```

`max_steps` 为正数时会覆盖 `num_train_epochs`。这适合当前阶段：先让用户观察 `batch -> forward -> loss/logits -> metric`，不要被完整训练耗时打断学习节奏。

### 推荐二：小型学习 run

目标是看到 accuracy 变化，同时保留 eval / save / best model 的体验。

```python
def preprocess_function(examples):
    return tokenizer(examples["text"], truncation=True, max_length=256)

tokenized_imdb = imdb.map(preprocess_function, batched=True)

small_train = tokenized_imdb["train"].shuffle(seed=42).select(range(5000))
small_eval = tokenized_imdb["test"].shuffle(seed=42).select(range(1000))

training_args = TrainingArguments(
    output_dir="imdb-small-run",
    learning_rate=2e-5,
    per_device_train_batch_size=16,
    per_device_eval_batch_size=32,
    max_steps=300,
    eval_strategy="steps",
    eval_steps=100,
    save_strategy="steps",
    save_steps=100,
    save_total_limit=2,
    load_best_model_at_end=True,
    metric_for_best_model="accuracy",
    push_to_hub=False,
    logging_steps=20,
    dataloader_pin_memory=False,
    report_to="none",
)
```

这档适合在理解主链路后运行一次，观察：

```text
training loss 有没有下降
eval accuracy 有没有变化
checkpoint 如何保存
best model 如何选择
```

### 推荐三：接近官方完整 run

目标是得到一个更完整的 IMDb fine-tuned model。

```python
training_args = TrainingArguments(
    output_dir="my_awesome_imdb_beat-model",
    learning_rate=2e-5,
    per_device_train_batch_size=16,
    per_device_eval_batch_size=32,
    num_train_epochs=2,
    weight_decay=0.01,
    eval_strategy="epoch",
    save_strategy="epoch",
    save_total_limit=2,
    load_best_model_at_end=True,
    metric_for_best_model="accuracy",
    push_to_hub=False,
    dataloader_pin_memory=False,
    report_to="none",
)
```

仍然建议先 `push_to_hub=False`，等训练结果确认后再手动 push。

### 当前建议

当前阶段使用“机制观察 run”。先缩小样本数、限制 `max_length`、使用 `max_steps`、关闭 save 和 Hub push。等可以自己解释 TrainingArguments 每个参数改变了什么，再逐步恢复 eval、save、best model 和 Hub artifact。

## 保存模型与 `pipeline` 加载排障

如果运行：

```python
classifier = pipeline("sentiment-analysis", model="my_awesome_imdb_beat-model")
```

报错：

```text
ValueError: Unrecognized model in my_awesome_imdb_beat-model.
Should have a `model_type` key in its config.json.
```

优先按这个方向理解：

```text
pipeline 收到一个字符串 model="my_awesome_imdb_beat-model"
-> 如果当前目录下存在同名本地目录，就按本地模型目录加载
-> AutoConfig 试图读取 config.json，判断模型架构
-> 目录不是完整 Transformers model artifact
-> 无法识别 model_type，加载失败
```

一个可加载的 Transformers 模型目录通常至少需要：

```text
config.json              # 里面要有 model_type，例如 "distilbert"
model.safetensors        # 或 pytorch_model.bin
tokenizer.json           # 或 tokenizer 相关文件
tokenizer_config.json
```

当前错误常见原因：

```text
训练还没跑到保存点。
save_strategy="no"，但没有手动 trainer.save_model()。
训练被中断，output_dir 只创建了空目录。
notebook 当前工作目录和预期不同，保存/加载看的不是同一个路径。
push_to_hub=True 只是配置了 Hub 行为，不等于本地目录已经有完整可加载 artifact。
```

先用这个 probe 判断目录是否完整：

```python
from pathlib import Path
import json

save_dir = Path("my_awesome_imdb_beat-model")

print(save_dir.resolve())
print([p.name for p in save_dir.iterdir()])

config_path = save_dir / "config.json"
print(config_path.exists())

if config_path.exists():
    config = json.loads(config_path.read_text())
    print(config.get("model_type"))
    print(config.get("architectures"))
```

如果目录是空的，或者没有 `config.json` / `model.safetensors` / tokenizer 文件，就不要用它做 `pipeline` 的 `model`。

### 修复方式一：直接用内存中的模型

如果刚刚训练完，还在同一个 notebook kernel 里，最小修复是直接把 `trainer.model` 交给 pipeline：

```python
from transformers import pipeline

classifier = pipeline(
    "text-classification",
    model=trainer.model,
    tokenizer=tokenizer,
)

classifier("I've been waiting for a Hugging Face course my whole life.")
```

这绕过了本地目录加载，适合快速验证 forward / logits / label。

### 修复方式二：显式保存后再从目录加载

如果想验证“保存 artifact -> 重新加载 -> 推理”的链路，就显式保存：

```python
save_dir = "my_awesome_imdb_beat-model"

trainer.save_model(save_dir)
tokenizer.save_pretrained(save_dir)

classifier = pipeline(
    "text-classification",
    model=save_dir,
    tokenizer=save_dir,
)

classifier("I've been waiting for a Hugging Face course my whole life.")
```

这一步要确认 `save_dir` 里至少有：

```text
config.json
model.safetensors
tokenizer.json
tokenizer_config.json
```

### 修复方式三：从 checkpoint 加载

如果训练过程已经保存了 checkpoint，可以直接加载 checkpoint：

```python
checkpoint_dir = "my_awesome_imdb_beat-model/checkpoint-100"

classifier = pipeline(
    "text-classification",
    model=checkpoint_dir,
    tokenizer=tokenizer,
)
```

前提是 checkpoint 目录里有完整模型权重和 config。tokenizer 可以来自原 tokenizer，也可以来自保存了 tokenizer 文件的同一个目录。

### 修复方式四：从 Hub repo 加载

如果已经成功：

```python
trainer.push_to_hub()
```

并且 Hub 上存在完整模型 repo，则用完整 repo id：

```python
classifier = pipeline(
    "text-classification",
    model="hedonwang/my_awesome_imdb_beat-model",
)
```

不要把“本地目录名”和“Hub repo id”混为一谈。`model="my_awesome_imdb_beat-model"` 会优先受当前工作目录影响；`model="username/repo_name"` 才是明确从 Hub repo 加载。

## 最小观察点

回到 notebook 时，只加这些 probe，不急着深入 Trainer 源码。

### 原始样本

```python
sample = imdb["train"][0]
print(sample.keys())
print(sample["label"])
print(sample["text"][:300])
```

要回答：

```text
原始 dataset 提供了哪些字段？
哪个字段是输入？
哪个字段是监督信号？
```

### Tokenized sample

```python
sample_tok = tokenizer(sample["text"], truncation=True)
print(sample_tok.keys())
print(len(sample_tok["input_ids"]))
print(tokenizer.decode(sample_tok["input_ids"][:50]))
```

要回答：

```text
tokenizer 新增了什么？
为什么 label 没有被 tokenizer 改掉？
```

### Collated batch

```python
model_input_keys = ["input_ids", "attention_mask", "label"]

features = [
    {k: v for k, v in tokenized_imdb["train"][i].items() if k in model_input_keys}
    for i in range(3)
]

batch = data_collator(features)

print(batch.keys())
print(batch["input_ids"].shape)
print(batch["attention_mask"].shape)
print(batch["labels"].shape)
```

要回答：

```text
collator 的输入是几条 dict-like sample。
collator 的输出是模型可吃的 tensor batch。
```

如果直接写：

```python
features = [tokenized_imdb["test"][i] for i in range(3)]
batch = data_collator(features)
```

可能会报：

```text
ValueError: too many dimensions 'str'
Perhaps your features (`text` in this case) have excessive nesting
```

原因是 `tokenized_imdb["test"][i]` 里可能还保留了原始 `text` 字符串字段。`data_collator` 的工作是把样本整理成 tensor batch，而字符串 `text` 不能被转成模型输入 tensor。

所以手动 probe 时要只保留模型需要的字段：

```text
input_ids
attention_mask
label
```

`DataCollatorWithPadding` 会把 `label` 整理成 batch 里的 `labels`。

正式训练时，`Trainer` 通常会根据模型 forward signature 移除无关列；但手动调用 `data_collator(features)` 时，这层自动清理不会替你发生。

### Model forward

```python
small_batch = {k: v.to(model.device) for k, v in batch.items()}
outputs = model(**small_batch)

print(outputs.loss)
print(outputs.logits.shape)
print(outputs.logits[:3])
```

要回答：

```text
为什么 logits 第二维是 2？
为什么传 labels 时 outputs 里会有 loss？
```

这里的目标不是训练，而是绕开 `Trainer` 和 `pipeline`，直接观察模型本体的 forward 契约：

```text
batch 里的 input_ids / attention_mask / labels
  -> model(**batch)
  -> outputs.loss + outputs.logits
```

其中：

```text
outputs.logits.shape == [batch_size, num_labels]
```

IMDb 二分类里，`num_labels=2`，所以如果当前 batch 有 3 条样本，就应该看到：

```text
outputs.logits.shape == [3, 2]
```

这一步要建立的直觉是：

```text
logits 是模型对每个类别打的原始分数。
loss 是模型拿 logits 和 labels 比较后算出的训练信号。
```

如果没有传 `labels`：

```python
outputs = model(input_ids=batch["input_ids"], attention_mask=batch["attention_mask"])
```

通常只能得到 logits，没有 supervised loss。因为模型不知道正确答案是什么。

### Manual prediction

```python
pred_ids = outputs.logits.argmax(dim=-1)
print(pred_ids)
print([model.config.id2label[i.item()] for i in pred_ids])
```

要回答：

```text
pipeline 最终输出的 label，本质上是 logits argmax 再查 id2label。
```

这一步的目标是把 `pipeline` 的后处理拆开。模型只输出 logits，例如：

```text
sample 0 logits: [-1.2, 2.4]
sample 1 logits: [ 1.8,-0.7]
```

每一行对应一条样本，每一列对应一个 label：

```text
column 0 -> NEGATIVE
column 1 -> POSITIVE
```

`argmax(dim=-1)` 就是在每条样本的类别维度上找最高分：

```text
[-1.2, 2.4] -> 1 -> POSITIVE
[ 1.8,-0.7] -> 0 -> NEGATIVE
```

所以 `pipeline("text-classification")` 大致做的是：

```text
text
  -> tokenizer
  -> model forward
  -> logits
  -> argmax / softmax
  -> id2label
  -> {"label": "...", "score": ...}
```

`argmax` 负责选类别，`softmax` 负责把 logits 变成类似置信度的分数。

### 如果 4 / 5 不知道做什么

先把 4 / 5 当成两个调试镜头：

```text
4. Model forward：看模型本体吃什么、吐什么。
5. Manual prediction：看 pipeline 最后的 label 是怎么从 logits 变出来的。
```

最小代码可以直接贴到 notebook 里：

```python
model_input_keys = ["input_ids", "attention_mask", "label"]

features = [
    {k: v for k, v in tokenized_imdb["test"][i].items() if k in model_input_keys}
    for i in range(3)
]

batch = data_collator(features)

batch = {k: v.to(model.device) for k, v in batch.items()}
outputs = model(**batch)

print("batch keys:", batch.keys())
print("labels:", batch["labels"])
print("loss:", outputs.loss)
print("logits shape:", outputs.logits.shape)
print("logits:", outputs.logits)

pred_ids = outputs.logits.argmax(dim=-1)
pred_labels = [model.config.id2label[i.item()] for i in pred_ids]

print("id2label:", model.config.id2label)
print("pred ids:", pred_ids)
print("pred labels:", pred_labels)
print("gold labels:", [model.config.id2label[i.item()] for i in batch["labels"]])
```

如果 `pred ids` 是 `0`，但 `pred labels` 显示为 `POSITIVE`，说明当前 `model.config.id2label` 里是：

```text
0 -> POSITIVE
```

这时必须先确认它是否和数据集真实 label 语义一致：

```python
print(model.config.id2label)
print(model.config.label2id)
print(tokenized_imdb["test"].features["label"])
print(tokenized_imdb["test"][0]["label"])
print(tokenized_imdb["test"][0]["text"][:500])
```

如果数据集约定和 `model.config.id2label` 相反，pipeline 输出的文字标签就会反，哪怕模型的数字分类是对的。

看完后只回答三句话：

```text
batch 里有什么？
model forward 输出了什么？
logits 如何变成最终 label？
```

## 反推练习

做完上面的 probe 后，盖住教程，只保留这个骨架，自己补全代码：

```python
checkpoint = "distilbert/distilbert-base-uncased"

# 1. dataset contract

# 2. tokenizer contract

# 3. preprocess contract

# 4. dynamic batch contract

# 5. metric contract

# 6. model contract

# 7. trainer contract

# 8. inference contract
```

每补一段，都问四个问题：

```text
它的输入是什么？
它的输出是什么？
下游谁依赖它？
删掉它会坏在哪里？
```

如果这四个问题答不出来，就说明还在 recipe copying；如果能答出来，就开始进入机制掌握。

## 当前边界

这次先不展开：

- `Trainer.train()` 的完整源码。
- DistilBERT encoder 内部 attention 细节。
- Hub 上传和 model card 的生产规范。

当前只要求建立 sequence classification 的主链路：

```text
raw text + label
  -> tokenized sample
  -> padded batch
  -> sequence classification logits/loss
  -> metric / inference label
```

## 官方来源

- [Hugging Face Transformers Text classification](https://huggingface.co/docs/transformers/tasks/sequence_classification)
- [Hugging Face Transformers DataCollatorWithPadding](https://huggingface.co/docs/transformers/v5.12.0/en/main_classes/data_collator#transformers.DataCollatorWithPadding)
