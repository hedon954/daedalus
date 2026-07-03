# Concept Map

记录跨章节概念、机制问题和薄弱点。每个概念都应该能回到课程章节、实验观察或迁移任务。

## Core Concepts

| Concept | 学习来源 | 机制问题 | 已验证证据 | 薄弱点 |
| --- | --- | --- | --- | --- |
| pipeline | task string 如何展开成 tokenizer、model、config 和 postprocess？ | 已跑通 sentiment pipeline | 默认模型选择仍需固定 | 待补 Chapter 2 观察 |
| tokenizer | 文本如何变成 `input_ids`、`attention_mask` 和可 batch 的张量？BPE / WordPiece 的切词差异是什么？ | 已观察 tokenizer 输出和 decode；已记录 [`bpe-vs-wordpiece.md`](../topics/lora-feedback-loop/notes/04-lesson-lab/bpe-vs-wordpiece.md) | padding/truncation 与 batch 关系仍需练习 | 待补 lesson lab |
| sequence classification | 文本分类任务如何从 `text + label` 反推到 tokenizer、collator、model head、metric 和 Trainer？ | 用户已完成 `sequence_classification.ipynb`；已观察 batch keys、loss、logits shape、argmax、id2label；已闭卷重写最小 flow；已生成 [`04-sequence-classification-derivation.md`](../topics/lora-feedback-loop/guides/04-lesson-lab/04-sequence-classification-derivation.md) | 小样本训练需避免未 shuffle 的标签偏斜；需确认 `id2label` 与数据集 label 语义一致 | 已通过，待迁移 |
| task heads | 不同任务如何改变 head、logits shape、loss 和 postprocess？ | Chapter 1/5 已完成 Causal LM、sequence classification、token classification、question answering、summarization；已记录 [`06-token-classification-derivation.md`](../topics/lora-feedback-loop/guides/04-lesson-lab/06-token-classification-derivation.md)、[`07-question-answering-derivation.md`](../topics/lora-feedback-loop/guides/04-lesson-lab/07-question-answering-derivation.md)、[`08-summarization-derivation.md`](../topics/lora-feedback-loop/guides/04-lesson-lab/08-summarization-derivation.md) 及 summarization 运行 note | 仍需 translation 对比；QA 严肃评估需补完整 span postprocess 与 EM/F1 | 当前 lesson |
| seq2seq generation | Encoder-decoder 任务如何从 source text 生成 target text？ | 用户已完成 `summarization.ipynb`：BillSum `ca_test` 加载、T5 `summarize:` prefix、seq2seq labels、ROUGE 评估、`model.generate` 推理和 checkpoint 加载；已记录 [`2026-07-03-summarization-lab-recap.md`](../topics/lora-feedback-loop/notes/04-lesson-lab/2026-07-03-summarization-lab-recap.md) | Translation 仍需观察同构流程下的任务提示、source/target language 和指标差异 | 当前 lesson |
| pipeline internals | `pipeline("text-classification")` 如何展开成 tokenizer、model forward、softmax/argmax、id2label？ | 已跑通过 pipeline smoke test 和 sequence classification inference | 先完成 Chapter 1/5 task labs，再手写 pipeline 等价流程 | 延后 |
| causal language modeling | GPT-2 为什么训练 next token prediction？`labels = input_ids.copy()` 为什么不是让模型复制输入？ | 已完成 DistilGPT2 quick training run | shifted loss 机制仍需可视化 | 候选 mechanism deep dive |
| Trainer | `Trainer.train()` 如何串起 dataloader、forward、loss、backward、optimizer？ | 已迁入 `guides/04-lesson-lab/03-trainer-train-under-the-hood.md` | 仍需用户打印 batch/loss/logits | 下一步 lesson lab |

## Mechanism Questions

- `ForCausalLMLoss` 如何 shift logits / labels？
- BPE / WordPiece 的切词策略差异如何影响 tokenizer 与 model 的匹配？
- `pipeline("text-generation")` 如何调用 tokenizer、model.generate 和 decode？
- `Trainer` 如何处理 MPS、dataloader、optimizer 和 scheduler？

## Transfer Links

- 将 Causal LM / Trainer 训练链路迁移到闲鱼买家 Agent suggestion next action SFT / LoRA feedback loop。
