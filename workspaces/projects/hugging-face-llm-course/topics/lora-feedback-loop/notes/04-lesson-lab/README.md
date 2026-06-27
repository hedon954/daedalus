# 04 Lesson Lab：运行证据

本文件记录用户亲自运行 demo 的证据、观察和下一步调试问题。

## 2026-06-16 Transformers Pipeline Smoke Test

### 运行目标

确认 Hugging Face Transformers 的最小 pipeline 能在当前 demo 环境中运行，并验证 Cursor / Pyright 配置与运行时环境的差异。

### 运行环境

- Demo：`demo/hugging-face-course-learning`
- Python env：`.venv`
- 运行方式：`uv run transformer-lib/pipeline.py`
- 关键依赖：`transformers==5.12.0`，`torch>=2.12.0`

### 运行结果

用户已跑通 sentiment/text classification pipeline。

观察到的输出：

```text
[{'label': 'POSITIVE', 'score': 0.9982948899269104}]
```

### 重要现象

1. `pipeline("sentiment-analysis")` 运行时可成功，但 BasedPyright 报 `reportCallIssue`。
2. 原因是 Transformers runtime 支持 `sentiment-analysis` alias，但类型 overload 中 canonical task 是 `text-classification`。
3. demo 已改为 `pipeline("text-classification")`，保持运行结果等价，同时减少静态类型误报。
4. 运行时出现 Hugging Face Hub unauthenticated warning；当前 smoke test 可忽略，后续大量下载或 gated model 再配置 `HF_TOKEN`。

### 当前结论

第一个 HF pipeline 已跑通。下一步不急着进入 LoRA，而是要把 pipeline 的机制拆开：

- task name 如何映射到默认 model。
- model/tokenizer/config 如何被下载和缓存。
- pipeline 输入如何经过 tokenizer、model、postprocess。
- 为什么不指定 model 在生产中不推荐。

### 下一步动作

- 把当前脚本从“能跑”改成“可观察”：打印 pipeline 的 task、model name、tokenizer、config、缓存/下载行为。
- 固定 model name，避免依赖 Transformers 默认模型选择。
- 将运行命令和输出沉淀到 demo README 或 runbook。

## 2026-06-23 Chapter 1/5 Text Classification Tokenizer Note

- 当前问题：BPE 和 WordPiece 的区别是什么？
- 独立笔记：[`bpe-vs-wordpiece.md`](bpe-vs-wordpiece.md)
- 当前结论：BPE 更像频率驱动的 merge rules；WordPiece 更像词表驱动的最长匹配，并在训练时更重视片段之间的相对绑定强度。

## 2026-06-26 Sequence Classification Lab Completion

### 运行目标

完成 Hugging Face Transformers text / sequence classification recipe，建立从 IMDb 文本分类数据到 DistilBERT fine-tuning 和 pipeline 推理的整体体感。

### 运行环境

- Notebook：`demo/hugging-face-course-learning/transformer-work/sequence_classification.ipynb`
- Dataset：`stanfordnlp/imdb`
- Base model：`distilbert/distilbert-base-uncased`
- 关键组件：`AutoTokenizer`、`DataCollatorWithPadding`、`AutoModelForSequenceClassification`、`TrainingArguments`、`Trainer`、`pipeline`

### 用户观察

用户已完成第二个 lab，对大致流程有一些理解，但整体仍是模糊印象；当前主要卡点不是“代码是否能跑”，而是仍然只能按教程抄，无法自己推导出这条链路。

### 当前结论

Sequence classification 的主链路应被理解为：

```text
raw text + label
  -> tokenizer(text, truncation=True)
  -> DataCollatorWithPadding 动态组成 batch
  -> AutoModelForSequenceClassification 输出 [batch_size, num_labels] logits
  -> compute_metrics 把 logits argmax 后与 labels 比较
  -> Trainer 驱动 train / eval / save / push
  -> pipeline 或手动 forward 做推理
```

当前退出标准不是“能复现官方代码”，而是能从任务契约反推每个对象为什么存在、输入输出是什么、删掉会坏在哪里。

### 下一步动作

- 按 [`../../guides/04-lesson-lab/04-sequence-classification-derivation.md`](../../guides/04-lesson-lab/04-sequence-classification-derivation.md) 复查 notebook。
- 补 raw sample、tokenized sample、collated batch、model forward、logits -> label 的实际打印观察。
- 盖住教程，用 guide 里的 8 段 skeleton 重新写一遍最小 sequence classification flow。

## 2026-06-26 Sequence Classification Pipeline Load Error

### 运行目标

在 sequence classification 训练后，用 `pipeline` 加载 fine-tuned model 并做一次 sentiment/text-classification 推理。

### 用户观察

运行：

```python
from transformers import pipeline

classifier = pipeline("sentiment-analysis", model="my_awesome_imdb_beat-model")
classifier("I've been waiting for a Hugging Face course my whole life.")
```

报错：

```text
ValueError: Unrecognized model in my_awesome_imdb_beat-model.
Should have a `model_type` key in its config.json.
```

### 当前诊断

`pipeline` 会把字符串形式的 `model` 当作本地目录或 Hub repo 来解析。当前本地存在 `transformer-work/my_awesome_imdb_beat-model`，但该目录为空，不是一个完整的 Transformers model artifact，因此无法从 `config.json` 识别 `model_type`。

一个可加载目录至少需要能提供模型 config、权重和 tokenizer 文件，例如：

```text
config.json
model.safetensors
tokenizer.json
tokenizer_config.json
```

### 下一步动作

- 如果只是快速验证推理，用 `pipeline("text-classification", model=trainer.model, tokenizer=tokenizer)`。
- 如果要验证保存/加载链路，先执行 `trainer.save_model(save_dir)` 和 `tokenizer.save_pretrained(save_dir)`，再用 `pipeline(..., model=save_dir, tokenizer=save_dir)`。
- 相关排障已补入 [`../../guides/04-lesson-lab/04-sequence-classification-derivation.md`](../../guides/04-lesson-lab/04-sequence-classification-derivation.md)。

### 2026-06-27 更新

用户报告该问题已跑通。当前不继续追新实验，下一步转入 sequence classification derivation review：补 raw sample、tokenized sample、collated batch、model forward、logits -> label 的实际观察，并尝试盖住教程重写最小 flow。

## 2026-06-27 Sequence Classification Derivation Review

### 用户当前复述

用户已经能解释前三段链路：

- Dataset：加载原始数据集，提供 train / test，每条数据包含 `text` 和 `label`；`label` 服务于最终分类目标。
- Tokenizer：把 dataset 中的 `text` 转为 `input_ids` 和 `attention_mask`，因为模型不能直接处理自然语言字符串。
- Collator：把多条变长样本通过 padding 填充到一样长，从而可以 batch 训练。

### 需要校正的点

Tokenizer 不是把文字“向量化”。更准确的链路是：

```text
text
  -> tokenizer
  -> tokens
  -> token ids
  -> model embedding layer
  -> embedding vectors
```

`input_ids` 是 embedding table 的查表索引；真正的向量表示发生在模型内部的 embedding layer。

### 当前卡点

用户暂时不知道 4 / 5 要做什么：

```text
4. 手动跑 model(**batch)：观察 outputs.loss 和 outputs.logits.shape。
5. 手动 argmax(logits) 再查 id2label：理解 pipeline 的 label 如何从模型输出得到。
```

### 下一步动作

- 按 guide 的 `Model forward` / `Manual prediction` 小节，把 4 / 5 的 probe 加到 notebook。
- 只回答三个问题：`batch` 里有什么，`model forward` 输出了什么，`logits` 如何变成最终 `label`。
- 相关说明已补入 [`../../guides/04-lesson-lab/04-sequence-classification-derivation.md`](../../guides/04-lesson-lab/04-sequence-classification-derivation.md)。

### Collator probe 报错

用户在手动运行：

```python
features = [tokenized_imdb["test"][i] for i in range(3)]
batch = data_collator(features=features)
```

时遇到：

```text
ValueError: too many dimensions 'str'
Perhaps your features (`text` in this case) have excessive nesting
```

诊断：`tokenized_imdb["test"][i]` 里仍保留原始 `text` 字符串字段。手动调用 `data_collator` 时，它会尝试把 features 中的字段整理成 tensor；`text` 不是模型输入 tensor，因此报错。

修复：手动 probe 时只保留 `input_ids`、`attention_mask`、`label`，再交给 `DataCollatorWithPadding`。guide 中的 Collated batch 和 4/5 probe 代码已同步修正。

### Model forward / Manual prediction 观察

用户按 guide 跑通 4 / 5 probe，观察到：

```text
batch keys: dict_keys(['input_ids', 'attention_mask', 'labels'])
labels: tensor([0, 0, 0], device='mps:0')
loss: tensor(0.5448, device='mps:0', grad_fn=<NllLossBackward0>)
logits shape: torch.Size([3, 2])
logits: tensor([[ 0.3006, -0.3274],
        [-0.0302, -0.0735],
        [ 0.1679, -0.1778]], device='mps:0', grad_fn=<LinearBackward0>)
pred ids: tensor([0, 0, 0], device='mps:0')
pred labels: ['POSITIVE', 'POSITIVE', 'POSITIVE']
gold labels: ['POSITIVE', 'POSITIVE', 'POSITIVE']
```

当前解释：

- `batch keys` 表明模型真正吃到的是 `input_ids`、`attention_mask`、`labels`，不再包含原始 `text`。
- `labels` 是真实类别 id；当前 3 条样本的真实类别 id 都是 `0`。
- `loss=0.5448` 是模型把 `logits` 和 `labels` 对比后算出的分类损失；`grad_fn=<NllLossBackward0>` 说明这个 loss 仍连接着反向传播计算图。
- `logits shape=[3, 2]` 表示 batch 有 3 条样本，每条样本有 2 个类别分数。
- 每行 logits 里第 0 列都大于第 1 列，所以 `argmax` 得到 `pred ids=[0,0,0]`。
- `pred labels` 和 `gold labels` 都是 `POSITIVE`，说明在当前 `model.config.id2label` 下，这 3 条样本预测正确。

需要继续确认：当前输出显示 label id `0` 被映射为 `POSITIVE`。这必须和数据集真实 label 语义保持一致；否则可能出现数字预测对了，但文字标签反了的问题。guide 已补充 `id2label` / dataset label sanity check。

### 当前 checkpoint

用户确认已理解 sequence classification 的 forward / logits / loss / label mapping 链路：

```text
batch
  -> model(**batch)
  -> loss + logits
  -> argmax(logits)
  -> pred ids
  -> id2label
  -> human-readable labels
```

本轮 lesson 不再继续扩展新 API。下一步是闭卷重写最小 sequence classification flow，验证是否能脱离教程按任务契约推导代码。

## 2026-06-22 ELI5 Dataset Inspection

### 运行目标

观察 causal LM recipe 的第一步：原始 ELI5 数据集长什么样，为什么后续需要 flatten、抽取 `answers.text`、再 tokenizer。

### 运行环境

- Demo：`demo/hugging-face-course-learning`
- 运行方式：`uv run transformer-work/casual_lm/inspect_dataset.py`

### 运行结果

用户已跑通数据集观察脚本。

观察到：

```text
DatasetDict({
    train: Dataset({
        features: ['q_id', 'title', 'selftext', 'category', 'subreddit', 'answers', 'title_urls', 'selftext_urls'],
        num_rows: 4000
    })
    test: Dataset({
        features: ['q_id', 'title', 'selftext', 'category', 'subreddit', 'answers', 'title_urls', 'selftext_urls'],
        num_rows: 1000
    })
})
```

关键样本结构：

- 每条样本是一个 Reddit 问题。
- `title` / `selftext` 是问题文本。
- `answers` 是嵌套结构，包含多个回答的 `text`、`score`、`a_id` 等。
- `answers.text` 是 list，不是单个字符串。

### 当前结论

ELI5 原始数据不是“干净的一段训练文本”，而是“问题 + 多个回答 + 元数据”的结构化样本。

因此后续代码的必要性变清楚了：

- `flatten()`：把嵌套字段摊平，方便访问 `answers.text`。
- `" ".join(x)`：把多个回答文本合成一段可训练文本。
- `tokenizer(...)`：把文本转成 token ids。
- `remove_columns`：删掉 Trainer/collator 不需要的原始元数据字段。

### 下一步动作

- 运行 `inspect_tokenization.py`，观察文本如何变成 `input_ids` 和 `attention_mask`。
- 对比 tokenizer 前后的字段变化。
- 手动 decode 一小段 `input_ids`，确认 token ids 仍能还原成人类可读文本。

## 2026-06-22 ELI5 Tokenization Inspection

### 运行目标

观察 tokenizer 如何把 `answers.text` 里的自然语言文本转换成模型可读的 token ids，并确认 token ids 可以 decode 回文本。

### 运行环境

- Demo：`demo/hugging-face-course-learning`
- Notebook：`transformer-work/casual_lm/inspect_tokenizer.ipynb`

### 运行结果

用户已观察到 tokenization 后的数据结构：

- 原始字段仍然保留时，dataset 增加了 `input_ids` 和 `attention_mask`。
- 使用 `remove_columns` 后，样本只剩下训练需要的 token 字段。
- `tokenizer.decode(input_ids)` 可以还原出拼接后的回答文本。

观察到的重要 warning：

```text
Token indices sequence length is longer than the specified maximum sequence length for this model
```

这说明某些拼接后的回答 token 数超过模型上下文上限 `1024`。当前阶段这不是 tokenizer 错误，而是在提醒：这些长序列不能直接喂给 DistilGPT2，需要后续 `group_texts` 按 `block_size` 切成模型可接受的固定长度片段。

### 当前结论

tokenizer 只完成了这一步：

```text
自然语言文本 -> input_ids / attention_mask
```

它还没有完成 causal LM 训练需要的：

```text
长 token stream -> fixed-size blocks -> labels
```

因此下一步必须观察 `group_texts`：

- 为什么要把 token 拼成长序列。
- 为什么要按 `block_size=128` 切块。
- 为什么 `labels = input_ids.copy()`。
- 为什么超过 1024 的 warning 会在切块后被解决。

### 下一步动作

- 运行或编写 `inspect_lm_blocks.py` / notebook。
- 打印切块前后的 token 长度。
- 打印第一条 block 的 `input_ids`、`labels`。
- decode 第一条 block，确认 block 仍然是连续文本片段。

## 2026-06-22 LM Block Inspection

### 运行目标

观察 `group_texts` 如何把 tokenizer 产生的长短不一 token 序列，改造成 causal LM 训练需要的固定长度 block，并生成 `labels`。

### 运行环境

- Demo：`demo/hugging-face-course-learning`
- Notebook：`transformer-work/casual_lm/inspect_tokenizer.ipynb`
- 参数：`block_size = 128`

### 运行结果

用户已观察到 `group_texts` 后第一条样本：

- `input_ids` 长度为 `128`。
- `labels` 长度为 `128`。
- `input_ids == labels` 为 `True`。
- `tokenizer.decode(input_ids)` 后是一段连续文本片段。

### 当前结论

`group_texts` 是 causal LM 数据构造的关键转折点：

```text
tokenized samples
  -> concatenate into one token stream
  -> split into fixed-size blocks
  -> labels = input_ids.copy()
```

这一步解决两个问题：

- 训练样本长度统一，方便 batch 和模型处理。
- 把普通 token 序列变成 next-token prediction 的监督样本。

`labels == input_ids` 并不表示模型在“复制输入”。对于 `AutoModelForCausalLM`，loss 计算时模型内部会 shift：

```text
输入位置:  token_0 token_1 token_2 ...
预测目标:          token_1 token_2 token_3 ...
```

所以表面上 labels 和 input_ids 一样，实际训练目标仍然是预测下一个 token。

### 下一步动作

- 观察 `DataCollatorForLanguageModeling(mlm=False)` 对 batch 做了什么。
- 确认 GPT2/DistilGPT2 为什么需要设置 `tokenizer.pad_token = tokenizer.eos_token`。
- 进入 model forward：看 `AutoModelForCausalLM` 输出 logits 的 shape。

## 2026-06-23 Data Collator / Trainer Warning Inspection

### 运行目标

观察进入 data collator / Trainer 后出现的运行 warning，并区分哪些是训练语义相关提示，哪些只是设备后端能力提示。

### 运行结果

用户观察到两个 warning：

```text
The tokenizer has new PAD/BOS/EOS tokens that differ from the model config and generation config.
The model config and generation config were aligned accordingly...
Updated tokens: {'pad_token_id': 50256}.
```

```text
'pin_memory' argument is set as true but not supported on MPS now, device pinned memory won't be used.
```

### 当前结论

第一个 warning 来自 `tokenizer.pad_token = tokenizer.eos_token` 这一类设置。GPT2 / DistilGPT2 默认没有独立的 `pad_token`，但 batch padding 需要一个 padding token id。当前运行把 tokenizer 的 `pad_token_id` 设置为 `50256`，也就是 GPT2 的 `eos_token_id`，Transformers 随后把 model config / generation config 同步到 tokenizer 的值。

这不是错误，而是提醒：tokenizer、model config、generation config 原本的特殊 token 配置不完全一致，库已自动对齐。当前 causal LM 学习阶段可以接受；后续做正式训练或生成评测时，应显式保存并检查 tokenizer / model config，避免训练和推理阶段使用不同的 special token 配置。

第二个 warning 是 PyTorch 在 Apple MPS 后端上的设备能力提示。`pin_memory=True` 主要服务于 CPU 到 CUDA GPU 的数据拷贝优化；MPS 当前不支持 pinned memory，因此该优化不会生效。它不影响训练语义，也不表示 batch、labels 或 loss 有问题。

### 下一步动作

- 在 notebook 或脚本里打印 `tokenizer.pad_token`、`tokenizer.pad_token_id`、`model.config.pad_token_id`，确认三者对齐。
- 观察 collator 输出 batch 的 keys、shape 和 padding 位置。
- 进入 model forward，打印 `loss` 与 `logits.shape`，确认 causal LM 训练链路已经从样本构造走到模型计算。

## 2026-06-23 MPS Training Speed Inspection

### 运行目标

判断当前 Causal LM 训练在 Apple Silicon 上偏慢时，应该优先优化硬件使用、训练参数，还是学习实验规模。

### 运行结果

当前环境检查结果：

```text
torch 2.12.0
mps available True
mps built True
```

用户观察到训练进度约为：

```text
299/3924, 2.51 it/s, Epoch 0.23/3, ETA about 24 min
```

### 当前结论

当前慢的主要原因不是 MPS 不可用，而是课程默认训练参数更接近“完整跑一遍 recipe”：

- `num_train_epochs` 默认约等于 3。
- `per_device_train_batch_size` 未显式设置，使用 Trainer 默认值。
- `lm_dataset["train"]` 已经被 `group_texts` 扩展到上万条 block。
- 当前学习目标是理解 causal LM pipeline，不需要先完整训练 3 epoch。

M 芯片能通过 MPS 提供 GPU 加速，但它不是 CUDA 训练卡。对本 topic 的学习节奏来说，优先级应该是：

```text
先缩小实验规模，快速观察链路
再做 MPS 参数微调
最后再迁移到云 GPU / 百炼类平台做更完整训练
```

### 下一步动作

- 快速学习 run：使用小数据子集、`max_steps`、关闭 push/save，先把 `dataset -> collator -> model forward -> loss -> train` 跑通。
- MPS 参数 run：确认 `trainer.args.device` 为 `mps`，尝试 `bf16=True` / `fp16=True`，并设置 `dataloader_pin_memory=False`。
- 正式训练 run：再恢复更大数据量、更长训练、完整 eval 和 artifact 保存。

## 2026-06-23 Causal LM Training Completion

### 运行目标

完成一次小步 Causal LM fine-tuning，从 ELI5 文本数据进入 `Trainer.train()`，并观察训练结果、评估指标、Hub 上传和生成表现。

### 运行环境

- Notebook：`demo/hugging-face-course-learning/transformer-work/casual_language_model.ipynb`
- Base model：`distilbert/distilgpt2`
- Training args：`max_steps=200`，`per_device_train_batch_size=16`，`eval_strategy="steps"`，`eval_steps=50`，`save_strategy="no"`，`dataloader_pin_memory=False`

### 运行结果

训练完成：

```text
TrainOutput(
  global_step=200,
  training_loss=3.9759389686584474,
  train_runtime=140.8586,
  train_samples_per_second=22.718,
  train_steps_per_second=1.42,
  epoch=0.3058103975535168
)
```

评估完成：

```text
Perplexity: 47.81
```

模型已上传到 Hugging Face Hub：

```text
https://huggingface.co/hedonwang/my_awesome_eli5_clm-model
commit: 147c2fc05c36740c96eea2ab572ee1c4c3cece70
```

生成测试使用 prompt：

```text
Somatic hypermutation allows the immune system to
```

生成结果能延续生物学相关开头，但很快漂移到 NES / console 的重复片段，说明当前小步训练已经跑通链路，但生成质量仍受训练步数、数据规模、采样策略、上下文主题一致性和 base model 能力约束。

### 当前结论

这次 notebook 跑通的是完整的 Causal LM fine-tuning 最小闭环：

```text
ELI5 raw records
  -> flatten nested answers
  -> tokenize answer text
  -> concatenate token streams
  -> split into fixed-size blocks
  -> create labels for next-token prediction
  -> collate batches
  -> fine-tune AutoModelForCausalLM
  -> evaluate perplexity
  -> push artifact
  -> run text generation
```

训练 loss / perplexity 只能说明模型在 held-out token prediction 上的平均不确定性；它不能直接保证生成内容事实正确、结构稳定或不会重复。生成结果里的主题漂移是后续学习 decoding、eval 和 task-specific fine-tuning 时必须正视的问题。

### 下一步动作

- 解释 `Trainer.train()` 背后实际发生的 forward / loss / backward / optimizer step。
- 修正推理 cell：从 Hub reload model 后需要重新构造 `inputs = tokenizer(prompt, return_tensors="pt").input_ids`，并注意 device 对齐。
- 进入 Chapter 2 的 pipeline 拆解：把 `pipeline("text-generation")` 展开成 tokenizer、model.generate、decode 三步。
