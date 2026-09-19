# 2026-06-30 Token Classification Lab 复述与完成证据

### 用户当前复述

用户已经能按任务契约解释 token classification 的主链路：

```text
sequence 中每个 word 都需要一个 NER tag
-> WNUT 提供 tokens 和 ner_tags，二者在 word-level 一一对应
-> tokens 不是模型 tokenizer 之后的 token，而更接近 words
-> tokenizer 会把 word 拆成 subword，并插入 special tokens
-> 因此需要 label alignment
-> special tokens 和非 word-start subword 设置为 -100，不参与 loss
-> 训练后用 pipeline 验证 NER 输出
```

### 需要校正的点

1. “输出最后就是一个 `num_labels` 大小的分类器”可以更精确地说成：token classification head 对每个 token hidden state 输出一个 `num_labels` 维 logits 向量，所以整体 `logits.shape == [batch_size, seq_len, num_labels]`。
2. 这里的 `seq_len` 是 tokenizer、special tokens、padding / truncation 后的模型序列长度，不是原始 `tokens` 的 word 数量。
3. `-100` 不是直接“告诉 Trainer”，而是作为 labels 进入模型 loss；PyTorch cross entropy 默认用 `ignore_index=-100` 忽略这些位置。Trainer 只是负责把 batch 传给 model。
4. “非 word-start-token 设置为 -100”是本 lab 采用的对齐策略，不是唯一策略；也可以把同一个 word 的标签复制到所有 subword 上，但当前策略更容易避免重复计算一个原始词的 loss。

### 当前结论

用户已经抓住本 lab 的关键：token classification 的难点不是训练 API，而是 tokenizer 改变了标注坐标系。

### 完成证据

Notebook：`demo/hugging-face-course-learning/transformer-work/token_classification.ipynb`

用户已跑通：

```text
flaitenberger/wnut_17
-> tokenizer + label alignment
-> DataCollatorForTokenClassification
-> AutoModelForTokenClassification
-> max_steps=50 小步训练
-> trainer.save_model("my_awesome_wnut_model")
-> tokenizer.save_pretrained("my_awesome_wnut_model")
-> pipeline("ner", model="my_awesome_wnut_model")
```

本地保存目录已包含可加载 artifact：

```text
config.json
model.safetensors
tokenizer.json
tokenizer_config.json
training_args.bin
```

Pipeline 验证句子：

```text
The Golden State Warriors are an American professional basketball team based in San Francisco.
```

观察到模型能走完 NER postprocess，并输出 `B-location` 预测，包括 `golden`、`san`、`francisco`。当前分数较低且预测质量不作为重点；本 lab 的验收目标是跑通 per-token classification 链路，并能解释 label alignment 与 `[batch_size, seq_len, num_labels]`。

### 下一步

Lab 3/8 Token classification 收口。下一步进入 Lab 4/8 Question answering，观察 start/end logits 和 span extraction。
