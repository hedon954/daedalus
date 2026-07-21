# 2026-06-23 Data Collator / Trainer Warning Inspection

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
