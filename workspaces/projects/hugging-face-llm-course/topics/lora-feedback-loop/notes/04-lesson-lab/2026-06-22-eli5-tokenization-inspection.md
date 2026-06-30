# 2026-06-22 ELI5 Tokenization Inspection

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
