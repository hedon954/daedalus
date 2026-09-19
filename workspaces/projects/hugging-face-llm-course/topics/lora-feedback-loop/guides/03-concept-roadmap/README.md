# HF Course 1/2 Concept Roadmap

本 guide 是迁移到 course-learning 后的第一份正式课程阶段指南。课程实验入口统一收敛到 `04-lesson-lab`。

## Current Course Scope

- 课程：Hugging Face LLM Course
- 当前范围：Chapter 1 Transformer Models + Chapter 2 Using Transformers
- 用户已完成：Transformers pipeline smoke test、ELI5 dataset inspection、tokenizer / group_texts / labels / collator / Trainer warning 观察、DistilGPT2 Causal LM quick training run。
- 当前目标：把“我跟着教程跑通了”重构成“我知道每一步在训练链路里承担什么职责”。

## Concept Questions

| Concept | 要回答的问题 | 观察证据 | 下一步 |
| --- | --- | --- | --- |
| `pipeline` | task string 如何展开成 tokenizer、model、config、postprocess？ | 已跑通 sentiment pipeline；观察到默认模型 warning | 进入 `04-lesson-lab` 时固定 model name，打印 pipeline 内部组件 |
| tokenizer | 文本为什么要先变成 `input_ids` / `attention_mask`？ | 已观察 tokenizer 输出、decode 和长序列 warning | 补一页 lesson lab：单条文本、batch、padding、truncation |
| Causal LM | 为什么 GPT-2 预训练目标是预测 next token？ | 已完成 ELI5 Causal LM quick training run | 用 logits shape 和 shifted labels 解释 loss |
| `group_texts` | 为什么要拼接再切成固定 block？输入输出类型是什么？ | 已观察 block size = 128，`labels == input_ids` | 将观察整理进 `notes/04-lesson-lab` |
| data collator | collator 在 batch 进入 model 前做了什么？ | 已理解 `mlm=False`、pad/eos warning | 打印一个 batch 的 keys、shape、device |
| `Trainer.train()` | batch 如何经过 forward、loss、backward、optimizer step？ | 已迁移到 `04-lesson-lab/03-trainer-train-under-the-hood.md` | 当前正式入口是 `guides/04-lesson-lab/README.md` |

## Mechanism Deep Dive Candidates

只在 lesson lab 观察仍解释不清时进入 `05-mechanism-deep-dive`。

- `ForCausalLMLoss` 如何 shift logits / labels。
- `pipeline("text-generation")` 如何调用 `model.generate`。
- `Trainer` 如何组装 dataloader、loss、accelerator、optimizer 和 scheduler。

## Transfer Thread

这些概念最终要迁移到闲鱼买家 Agent suggestion next action 的 LoRA/SFT feedback loop：

- tokenizer / batch：决定数据样本如何变成训练输入。
- labels / loss：决定模型到底被训练去预测什么。
- collator / padding：决定结构化 `{s,r}` target 如何批处理。
- Trainer / artifacts：决定训练过程如何脚本化、配置化、产物化。
- eval：决定 prompt/context baseline 与 LoRA/SFT 的比较是否可信。

## Next Lesson Lab

下一步生成：

- [`../04-lesson-lab/README.md`](../04-lesson-lab/README.md)

目标是继续执行 `Trainer.train()` 拆解 lesson lab：

```text
batch -> model(**batch) -> logits/loss -> loss.backward -> optimizer.step
```

用户需要亲自运行或复查 notebook 中的打印结果；Agent 不能把自己的解释写成用户已观察证据。
