# 2026-06-23 MPS Training Speed Inspection

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
