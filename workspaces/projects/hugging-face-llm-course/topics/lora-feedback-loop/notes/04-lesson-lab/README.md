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
