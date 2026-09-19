# 2026-06-26 Sequence Classification Pipeline Load Error

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
