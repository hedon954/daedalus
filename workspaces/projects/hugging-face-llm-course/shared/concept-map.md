# Concept Map

记录跨章节概念、机制问题和薄弱点。每个概念都应该能回到课程章节、实验观察或迁移任务。

## Core Concepts

| Concept | 学习来源 | 机制问题 | 已验证证据 | 薄弱点 |
| --- | --- | --- | --- | --- |
| pipeline | task string 如何展开成 tokenizer、model、config 和 postprocess？ | 已跑通 sentiment pipeline；用户已完成 Chapter 2 | 默认模型选择仍需固定 | 按需复习，不阻塞 Chapter 3 |
| tokenizer | 文本如何变成 `input_ids`、`attention_mask` 和可 batch 的张量？BPE / WordPiece 的切词差异是什么？ | 已观察 tokenizer 输出和 decode；已记录 [`bpe-vs-wordpiece.md`](../topics/lora-feedback-loop/notes/04-lesson-lab/bpe-vs-wordpiece.md) | padding/truncation 与 batch 关系仍需练习 | 待补 lesson lab |
| sequence classification | 文本分类任务如何从 `text + label` 反推到 tokenizer、collator、model head、metric 和 Trainer？ | 用户已完成 `sequence_classification.ipynb`；已观察 batch keys、loss、logits shape、argmax、id2label；已闭卷重写最小 flow；已生成 [`04-sequence-classification-derivation.md`](../topics/lora-feedback-loop/guides/04-lesson-lab/04-sequence-classification-derivation.md) | 小样本训练需避免未 shuffle 的标签偏斜；需确认 `id2label` 与数据集 label 语义一致 | 已通过，待迁移 |
| task heads | 不同任务如何改变 head、logits shape、loss 和 postprocess？ | Chapter 1/5 已完成 Causal LM、sequence classification、token classification、question answering、summarization；已记录 [`06-token-classification-derivation.md`](../topics/lora-feedback-loop/guides/04-lesson-lab/06-token-classification-derivation.md)、[`07-question-answering-derivation.md`](../topics/lora-feedback-loop/guides/04-lesson-lab/07-question-answering-derivation.md)、[`08-summarization-derivation.md`](../topics/lora-feedback-loop/guides/04-lesson-lab/08-summarization-derivation.md) 及 summarization 运行 note | QA 严肃评估需补完整 span postprocess 与 EM/F1；Translation / ASR / Image classification 已跳过或延后 | 已够进入架构归纳 |
| architecture families | encoder-only、decoder-only、encoder-decoder 分别适合什么任务？ | 已完成 Causal LM、sequence classification、token classification、question answering、summarization，可分别映射到 decoder-only、encoder-only、encoder-decoder；用户已复述 encoder-only 依赖完整上下文理解，见 [`2026-07-07-chapter1-6-encoder-only-recap.md`](../topics/lora-feedback-loop/notes/04-lesson-lab/2026-07-07-chapter1-6-encoder-only-recap.md)；用户已复述 decoder-only 是续写当前 input 而不是从 input 摘取，见 [`2026-07-07-chapter1-6-decoder-only-recap.md`](../topics/lora-feedback-loop/notes/04-lesson-lab/2026-07-07-chapter1-6-decoder-only-recap.md)；用户已辨析 summarization 可由多类架构实现，但 encoder-decoder 最贴合 source -> target 任务形态，见 [`2026-07-07-chapter1-6-encoder-decoder-recap.md`](../topics/lora-feedback-loop/notes/04-lesson-lab/2026-07-07-chapter1-6-encoder-decoder-recap.md)；最终总结见 [`2026-07-07-chapter1-6-architecture-selection-summary.md`](../topics/lora-feedback-loop/notes/04-lesson-lab/2026-07-07-chapter1-6-architecture-selection-summary.md) | 已通过，待迁移到 pipeline / model class 选择 | 已通过 |
| attention mechanisms | full attention、LSH attention、local attention、axial positional encodings 分别解决什么长序列问题？ | Chapter 1/6 已进入 attention mechanisms；官方指出标准 attention 是 O(n^2)，长文本会成为计算瓶颈 | 需要用户复述 full attention 的成本，以及 LSH / local / axial positional encodings 各自牺牲和保留了什么 | 当前 lesson |
| LLM inference | LLM 如何从 prompt 逐 token 生成？推理成本、延迟和显存主要花在哪里？ | 已有 Chapter 1/8 inference guide；用户确认 Chapter 1 已完成 | prefill/decode 与 KV cache 可在性能实践时复习 | 已越过，不阻塞 Chapter 3 |
| seq2seq generation | Encoder-decoder 任务如何从 source text 生成 target text？ | 用户已完成 `summarization.ipynb`：BillSum `ca_test` 加载、T5 `summarize:` prefix、seq2seq labels、ROUGE 评估、`model.generate` 推理和 checkpoint 加载；已记录 [`2026-07-03-summarization-lab-recap.md`](../topics/lora-feedback-loop/notes/04-lesson-lab/2026-07-03-summarization-lab-recap.md) | Translation 已按用户决定跳过/延后；如后续需要补做，只用来对比同构 seq2seq 流程下的 source/target 与指标差异 | 已够进入架构归纳 |
| pipeline internals | `pipeline("text-classification")` 如何展开成 tokenizer、model forward、softmax/argmax、id2label？ | 已跑通过 pipeline smoke test 和 sequence classification inference | 先完成 Chapter 1/5 task labs，再手写 pipeline 等价流程 | 延后 |
| causal language modeling | GPT-2 为什么训练 next token prediction？`labels = input_ids.copy()` 为什么不是让模型复制输入？ | 已完成 DistilGPT2 quick training run | shifted loss 机制仍需可视化 | 候选 mechanism deep dive |
| Trainer / full loop | Trainer 如何封装 dataloader、forward、loss、backward、optimizer、scheduler 与 evaluation？手写循环又暴露什么？ | 用户已完成 3/4 机制学习，并拆解 shuffle、`**batch`、forward/backward/optimizer/scheduler/zero-grad 顺序；详见 [`17-chapter3-finetuning-pretrained-model.md`](../topics/lora-feedback-loop/guides/04-lesson-lab/17-chapter3-finetuning-pretrained-model.md) | 完整训练运行结果、evaluation 与 Accelerate 证据按需补齐 | 已够推进，待运行复习 |
| learning curves | train/validation loss 与 accuracy 的不同组合如何暴露收敛、过拟合、欠拟合或训练震荡？ | 用户确认 Chapter 3 已读完；全章 guide 已提供诊断矩阵与单变量实验纪律 | 需要用户能从真实曲线提出原因假设、预期信号与下一次单变量实验 | 已读，待实验 |
| Hub model sharing | Hub Git 仓库、`from_pretrained()`、`push_to_hub()` 与 model card 如何共同支持版本化和复用？ | 用户确认 Chapter 4 已读完 | 尚未验收实际上传、版本选择与 model card 实践 | 已读，待实践 |
| local / remote dataset loading | 非 Hub 数据如何由文件格式、`data_files`、split mapping 与 `field` 转成 DatasetDict？ | 用户已进入 Chapter 5/2；Agent 已在同一 `.venv` 验证 GitHub URL、最终 raw URL、本地 `.json.gz` 均得到 train 442 / test 48，排障见 [`18-chapter5-2-local-remote-dataset-loading.md`](../topics/lora-feedback-loop/guides/04-lesson-lab/18-chapter5-2-local-remote-dataset-loading.md) | 需要用户亲自检查 loader name、source、split、rows/columns 和 nested field 的对应关系 | 当前 lesson |

## Mechanism Questions

- `ForCausalLMLoss` 如何 shift logits / labels？
- BPE / WordPiece 的切词策略差异如何影响 tokenizer 与 model 的匹配？
- `pipeline("text-generation")` 如何调用 tokenizer、model.generate 和 decode？
- `Trainer` 如何处理 MPS、dataloader、optimizer 和 scheduler？
- Chapter 1/6 中，架构选择到底由任务输出形态决定，还是由 attention 可见范围和生成方式共同决定？

## Transfer Links

- 将 Causal LM / Trainer 训练链路迁移到闲鱼买家 Agent suggestion next action SFT / LoRA feedback loop。
