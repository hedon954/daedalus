# Concept Map

记录跨章节概念、机制问题和薄弱点。每个概念都应该能回到课程章节、实验观察或迁移任务。

## Core Concepts

| Concept | 学习来源 | 机制问题 | 已验证证据 | 薄弱点 |
| --- | --- | --- | --- | --- |
| pipeline | task string 如何展开成 tokenizer、model、config 和 postprocess？ | 已跑通 sentiment pipeline | 默认模型选择仍需固定 | 待补 Chapter 2 观察 |
| tokenizer | 文本如何变成 `input_ids`、`attention_mask` 和可 batch 的张量？ | 已观察 tokenizer 输出和 decode | padding/truncation 与 batch 关系仍需练习 | 待补 lesson lab |
| causal language modeling | GPT-2 为什么训练 next token prediction？`labels = input_ids.copy()` 为什么不是让模型复制输入？ | 已完成 DistilGPT2 quick training run | shifted loss 机制仍需可视化 | 候选 mechanism deep dive |
| Trainer | `Trainer.train()` 如何串起 dataloader、forward、loss、backward、optimizer？ | 已迁入 `guides/04-lesson-lab/03-trainer-train-under-the-hood.md` | 仍需用户打印 batch/loss/logits | 下一步 lesson lab |

## Mechanism Questions

- `ForCausalLMLoss` 如何 shift logits / labels？
- `pipeline("text-generation")` 如何调用 tokenizer、model.generate 和 decode？
- `Trainer` 如何处理 MPS、dataloader、optimizer 和 scheduler？

## Transfer Links

- 将 Causal LM / Trainer 训练链路迁移到闲鱼买家 Agent suggestion next action SFT / LoRA feedback loop。
