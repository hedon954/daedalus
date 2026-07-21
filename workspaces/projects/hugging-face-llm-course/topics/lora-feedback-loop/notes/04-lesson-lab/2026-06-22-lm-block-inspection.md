# 2026-06-22 LM Block Inspection

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
