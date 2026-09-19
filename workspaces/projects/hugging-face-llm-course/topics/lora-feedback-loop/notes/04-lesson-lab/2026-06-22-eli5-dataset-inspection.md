# 2026-06-22 ELI5 Dataset Inspection

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
