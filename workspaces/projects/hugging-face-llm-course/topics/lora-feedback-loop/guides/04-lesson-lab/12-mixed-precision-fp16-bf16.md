# Mixed Precision：`fp16`、`bf16` 和 XPU

## `fp16=True` 是什么

`fp16` 是 floating point 16-bit，意思是用 16 位浮点数参与训练。默认训练通常用 `fp32`，也就是 32 位浮点数。

```text
fp32:
  数字更精细、更稳
  占显存/内存更多
  速度通常更慢

fp16:
  数字更省空间
  在支持的 GPU 上通常更快
  数值范围更小，更容易 overflow / underflow
```

在 Hugging Face `TrainingArguments` 里：

```python
fp16=True
```

表示启用半精度或混合精度训练。它主要是性能/显存优化，不是模型结构的一部分，也不是 summarization 任务必须要开的开关。

## 当前 lab 的默认建议

当前 lab 的目标是先跑通 seq2seq 链路：

```text
text -> tokenizer -> model.generate -> decode -> ROUGE
```

所以本地排障时建议先关掉：

```python
fp16=False
bf16=False
```

尤其在 Mac/MPS 环境里，`fp16=True` 未必像 CUDA/NVIDIA GPU 那样稳定或有明显收益。等链路跑通后，如果换到支持混合精度更成熟的 GPU，再打开它做速度优化。

## `bf16` 和 XPU

```text
fp16: 16-bit float，省显存/快，但数值范围小
bf16: bfloat16，同样 16-bit，但指数范围更接近 fp32，通常更稳
fp32: 32-bit float，默认稳妥选择
```

官方注释里的：

```python
# change to bf16=True for XPU
```

意思是：如果你在 Intel XPU 后端上训练，通常应该用 `bf16=True`，而不是 `fp16=True`。这里的 XPU 不是“所有 GPU”的泛称，在 PyTorch 语境里主要指 Intel GPU / accelerator 后端。

当前本地环境检查结果：

```text
torch 2.12.0
mps available True
cuda available False
xpu available False
```

所以本机是 Apple/MPS 路径，不是 XPU 路径。不要同时开 `fp16=True` 和 `bf16=True`。

## 来源

- [Hugging Face mixed precision training](https://huggingface.co/docs/transformers/mixed_precision_training)
- [PyTorch XPU documentation](https://docs.pytorch.org/docs/stable/notes/get_start_xpu.html)
