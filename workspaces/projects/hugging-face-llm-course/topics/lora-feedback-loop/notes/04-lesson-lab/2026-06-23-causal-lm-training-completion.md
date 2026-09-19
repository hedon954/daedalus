# 2026-06-23 Causal LM Training Completion

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
