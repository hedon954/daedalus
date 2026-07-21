# Chapter 1/5 Task Lab Sweep：用重复建立任务直觉

## 学习现场

用户原先希望先把 Hugging Face LLM Course Chapter 1/5 `How Transformers solve tasks` 这一节提到的 task labs 都完成，再进入 Chapter 2 `Behind the pipeline`。2026-07-07 路线收窄：已完成 5 个文本主线任务后，Translation / ASR / Image classification 跳过或延后，先进入 Chapter 1/6 `Transformer Architectures` 做架构归纳。

这个判断成立：当前阶段用户不是缺少“更高效的路线”，而是缺少跨任务的手感。前期机械式重复可以服务于模式识别：

```text
raw input
  -> tokenizer / processor / feature extractor
  -> model backbone
  -> task-specific head / decoder
  -> logits / generated ids / spans
  -> loss / metric / postprocess
  -> human-readable output
```

每个 lab 都不要追求完整效果，而是追求“我知道这个任务的输入输出形状和模型头为什么不同”。

## 官方页面覆盖的 Labs

Chapter 1/5 页面里所有 `Ready to try your hand...` 对应的任务实验一共 8 个。它们用同一套 Transformer 视角串起不同输入、架构、head、loss 和 postprocess：

1. Text generation / causal language modeling：GPT-2 / decoder-only / next-token prediction。
2. Text classification：BERT / encoder-only / sequence classification head。
3. Token classification：BERT / encoder-only / per-token classification head。
4. Question answering：BERT / encoder-only / span start-end logits。
5. Summarization：T5 / encoder-decoder / seq2seq generation。
6. Translation：T5 / encoder-decoder / seq2seq generation。
7. Automatic speech recognition：Whisper / audio encoder + autoregressive decoder。
8. Image classification：ViT / image patches + `[CLS]` representation + classification head。

注意：本 topic 的主线仍是 LoRA / fine-tuning feedback loop。这里不是要转成 audio / CV topic，而是用这些 labs 快速建立“不同任务怎么改输入、head、loss、postprocess”的直觉。

## 当前完成情况

| Lab | 状态 | 当前证据 | 下一步 |
| --- | --- | --- | --- |
| Text generation / Causal LM | done | `causal_language_model.ipynb`，已跑通 DistilGPT2 quick training run、perplexity、Hub 上传 | 后续只在对比 decoder-only 时回看 |
| Text classification | done | `sequence_classification.ipynb`，已跑通训练、pipeline load、forward/logits probe、闭卷复现 | 后续只在 encoder classification 对比时回看 |
| Token classification | done | `token_classification.ipynb`，已跑通 WNUT 数据加载、label alignment、小步训练、模型保存、`pipeline("ner")`；已记录用户复述 | 后续只在 per-token logits 对比时回看 |
| Question answering | done | `question_answering.ipynb`，已跑通 SQuAD 数据加载、字符答案到 token span 对齐、DistilBERT QA 小样本训练、Hub push，并观察到独立 start/end argmax 的空 span 风险 | 后续只在 QA postprocess / eval 对比时回看 |
| Summarization | done | `summarization.ipynb` 已跑通 BillSum / T5 / ROUGE / Seq2SeqTrainer / checkpoint 推理；已记录 [`2026-07-03-summarization-lab-recap.md`](../../notes/04-lesson-lab/2026-07-03-summarization-lab-recap.md) | 后续作为 encoder-decoder / seq2seq 代表任务回看 |
| Translation | skipped/deferred | 用户决定当前不做 | 如后续补做，只作为 summarization 的 seq2seq 对照 |
| Automatic speech recognition | skipped/deferred | 用户决定当前不做 | 非文本多模态任务不阻塞本 topic |
| Image classification | skipped/deferred | 用户决定当前不做 | 非文本多模态任务不阻塞本 topic |

## 每个 lab 的统一验收模板

每个 lab 都要回答同一组问题，不追求大而全：

```text
1. 任务输入是什么？
2. 监督信号 / 目标输出是什么？
3. tokenizer / processor 把输入变成了哪些 tensor？
4. batch 里有哪些 key？shape 是什么？
5. model class 是什么？它对应哪种架构：encoder / decoder / encoder-decoder？
6. task-specific head 或 decoder 输出了什么？
7. logits / generated ids / span 如何被 postprocess 成人类可读结果？
8. loss 或 metric 在比较什么？
9. 这个任务和上一个任务最大的结构差异是什么？
```

## 统一实验策略

前期允许机械重复，但不允许无意识烧时间：

- 使用小数据集或 `select(range(...))`，优先观察链路。
- 训练用 `max_steps` 控制成本。
- 默认 `push_to_hub=False`，确认 artifact 后再手动 push。
- Notebook 可以作为学习证据，但必须补最小观察输出：batch keys、shape、loss/logits 或 generated output。
- 不强求每个 lab 都完整 fine-tune 到好效果；先证明任务链路。
- 对 ASR / image classification 这类非文本任务，先做 inference 或极小样本观察；如果环境成本过高，只记录跳过理由。

## 当前排障：Question Answering 数据集 ID

在 `datasets==5.0.0` + `huggingface_hub==1.19.0` 组合下，课程旧写法：

```python
load_dataset("squad", split="train[:5000]")
```

会报：

```text
HfUriError: Invalid HF URI 'hf://datasets/squad@.../.huggingface.yaml'.
Repository id must be 'namespace/name', got 'squad'.
```

原因不是 SQuAD 数据集缺失，而是新版本 `huggingface_hub` 的 `hf://` 解析不再接受单段 repo id。`HfApi.dataset_info("squad")` 会解析到同一个数据集仓库 `rajpurkar/squad`，commit hash 与报错中的 revision 一致。当前 notebook 应使用：

```python
squad = load_dataset("rajpurkar/squad", split="train[:5000]")
```

已验证 `load_dataset("rajpurkar/squad", split="train[:2]")` 能生成 `id/title/context/question/answers` 字段。继续 lab 时，先打印一个样本和 batch shape，再进入 tokenizer offset mapping、`start_positions/end_positions`、`start_logits/end_logits` 和 span extraction。

如果下一格出现：

```text
AttributeError: 'DatasetDict' object has no attribute 'train_test_split'
```

先检查 `type(squad)`。`load_dataset(..., split="train[:5000]")` 返回单个 `Dataset`，可以调用 `.train_test_split()`；`load_dataset(...)` 不带 `split` 会返回已经包含 `train/validation` 的 `DatasetDict`，此时不能再对整个对象 split。为了避免 Jupyter 旧内存状态污染，推荐在 notebook 里分开命名：

```python
squad_raw = load_dataset("rajpurkar/squad", split="train[:5000]")
squad = squad_raw.train_test_split(test_size=0.2)
```

## 已完成：Token Classification

Token classification 和 text classification 最像，重复成本低，但能暴露一个重要差异：

```text
Text classification:
  one sequence -> one label
  logits.shape == [batch_size, num_labels]

Token classification:
  one sequence -> one label per token
  logits.shape == [batch_size, seq_len, num_labels]
```

单项 guide：[`06-token-classification-derivation.md`](06-token-classification-derivation.md)。本 sweep 文件只记录横向路线；token classification 的任务契约、label alignment、排障和验收细节都放在独立 guide 中。

已跑通并观察：

```text
dataset sample
-> tokenizer + label alignment
-> collated batch
-> model forward
-> logits shape
-> token-level predictions
```

特别关注：

- 为什么 label 需要和 token 对齐。
- 为什么 subword token 让 label alignment 变复杂。
- 为什么 `attention_mask` 不等于 label mask。
- 为什么 token classification 的输出比 sequence classification 多一个 `seq_len` 维度。

## 下一步进入 Chapter 1/6 的理由

Chapter 2 `Behind the pipeline` 不是取消，而是后移。当前先进入 Chapter 1/6 `Transformer Architectures`，因为已经完成的 5 个文本任务足够支撑一个更重要的归纳：

```text
任务不是只换 pipeline 名字
而是围绕输入理解方式、attention 可见范围、head / decoder 和输出形态选择架构
```

也就是说，先把已跑过的任务映射到 encoder-only、decoder-only、encoder-decoder，再拆 pipeline，会更有感觉。Translation / ASR / Image classification 后续只有在需要补 seq2seq 或多模态直觉时再恢复。

## 来源

- [Hugging Face LLM Course Chapter 1/5: How Transformers solve tasks](https://huggingface.co/learn/llm-course/en/chapter1/5)
- [Causal language modeling task guide](https://huggingface.co/docs/transformers/tasks/language_modeling#causal-language-modeling)
- [Text classification task guide](https://huggingface.co/docs/transformers/tasks/sequence_classification)
- [Token classification task guide](https://huggingface.co/docs/transformers/tasks/token_classification)
- [Question answering task guide](https://huggingface.co/docs/transformers/tasks/question_answering)
- [Summarization task guide](https://huggingface.co/docs/transformers/tasks/summarization)
- [Translation task guide](https://huggingface.co/docs/transformers/tasks/translation)
- [Automatic speech recognition task guide](https://huggingface.co/docs/transformers/tasks/asr)
- [Image classification task guide](https://huggingface.co/docs/transformers/tasks/image_classification)
