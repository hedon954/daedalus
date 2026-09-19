# Concept Map

记录跨章节概念、机制问题和薄弱点。每个概念都应该能回到课程章节、实验观察或迁移任务。

## Core Concepts

| Concept | 学习来源 | 机制问题 | 已验证证据 | 薄弱点 |
| --- | --- | --- | --- | --- |
| pipeline | task string 如何展开成 tokenizer、model、config 和 postprocess？ | 已跑通 sentiment pipeline；用户已完成 Chapter 2 | 默认模型选择仍需固定 | 按需复习，不阻塞 Chapter 3 |
| tokenizer | 文本如何变成 `input_ids`、`attention_mask`、special tokens 和可训练 labels？BPE / WordPiece / Unigram 的差异是什么？ | 已观察 tokenizer 输出和 decode；已记录 [`bpe-vs-wordpiece.md`](../topics/lora-feedback-loop/notes/04-lesson-lab/bpe-vs-wordpiece.md)；用户确认 Chapter 6 已读完 | 真实 Qwen3 template、长度分布、truncation、EOS 与 loss mask 证据待脱敏 | 博文 2 验收 |
| sequence classification | 文本分类任务如何从 `text + label` 反推到 tokenizer、collator、model head、metric 和 Trainer？ | 用户已完成 `sequence_classification.ipynb`；已观察 batch keys、loss、logits shape、argmax、id2label；已闭卷重写最小 flow；已生成 [`04-sequence-classification-derivation.md`](../topics/lora-feedback-loop/guides/04-lesson-lab/04-sequence-classification-derivation.md) | 小样本训练需避免未 shuffle 的标签偏斜；需确认 `id2label` 与数据集 label 语义一致 | 已通过，待迁移 |
| task heads | 不同任务如何改变 head、logits shape、loss 和 postprocess？ | Chapter 1/5 已完成 Causal LM、sequence classification、token classification、question answering、summarization；已记录 [`06-token-classification-derivation.md`](../topics/lora-feedback-loop/guides/04-lesson-lab/06-token-classification-derivation.md)、[`07-question-answering-derivation.md`](../topics/lora-feedback-loop/guides/04-lesson-lab/07-question-answering-derivation.md)、[`08-summarization-derivation.md`](../topics/lora-feedback-loop/guides/04-lesson-lab/08-summarization-derivation.md) 及 summarization 运行 note | QA 严肃评估需补完整 span postprocess 与 EM/F1；Translation / ASR / Image classification 已跳过或延后 | 已够进入架构归纳 |
| architecture families | encoder-only、decoder-only、encoder-decoder 分别适合什么任务？ | 已完成 Causal LM、sequence classification、token classification、question answering、summarization，可分别映射到 decoder-only、encoder-only、encoder-decoder；用户已复述 encoder-only 依赖完整上下文理解，见 [`2026-07-07-chapter1-6-encoder-only-recap.md`](../topics/lora-feedback-loop/notes/04-lesson-lab/2026-07-07-chapter1-6-encoder-only-recap.md)；用户已复述 decoder-only 是续写当前 input 而不是从 input 摘取，见 [`2026-07-07-chapter1-6-decoder-only-recap.md`](../topics/lora-feedback-loop/notes/04-lesson-lab/2026-07-07-chapter1-6-decoder-only-recap.md)；用户已辨析 summarization 可由多类架构实现，但 encoder-decoder 最贴合 source -> target 任务形态，见 [`2026-07-07-chapter1-6-encoder-decoder-recap.md`](../topics/lora-feedback-loop/notes/04-lesson-lab/2026-07-07-chapter1-6-encoder-decoder-recap.md)；最终总结见 [`2026-07-07-chapter1-6-architecture-selection-summary.md`](../topics/lora-feedback-loop/notes/04-lesson-lab/2026-07-07-chapter1-6-architecture-selection-summary.md) | 已通过，待迁移到 pipeline / model class 选择 | 已通过 |
| attention mechanisms | full attention、LSH attention、local attention、axial positional encodings 分别解决什么长序列问题？ | Chapter 1/6 已完成 guided walkthrough；用户确认全课程已读完 | 需要用户在课程总览博文中复述各自牺牲和保留了什么 | 按需 review |
| LLM inference | LLM 如何从 prompt 逐 token 生成？推理成本、延迟和显存主要花在哪里？ | 已有 Chapter 1/8 inference guide；用户确认 Chapter 1 已完成 | prefill/decode 与 KV cache 可在性能实践时复习 | 已越过，不阻塞 Chapter 3 |
| seq2seq generation | Encoder-decoder 任务如何从 source text 生成 target text？ | 用户已完成 `summarization.ipynb`：BillSum `ca_test` 加载、T5 `summarize:` prefix、seq2seq labels、ROUGE 评估、`model.generate` 推理和 checkpoint 加载；已记录 [`2026-07-03-summarization-lab-recap.md`](../topics/lora-feedback-loop/notes/04-lesson-lab/2026-07-03-summarization-lab-recap.md) | Translation 已按用户决定跳过/延后；如后续需要补做，只用来对比同构 seq2seq 流程下的 source/target 与指标差异 | 已够进入架构归纳 |
| pipeline internals | `pipeline("text-classification")` 如何展开成 tokenizer、model forward、softmax/argmax、id2label？ | 已跑通过 pipeline smoke test 和 sequence classification inference | 先完成 Chapter 1/5 task labs，再手写 pipeline 等价流程 | 延后 |
| causal language modeling | GPT-2 为什么训练 next token prediction？`labels = input_ids.copy()` 为什么不是让模型复制输入？ | 已完成 DistilGPT2 quick training run | shifted loss 机制仍需可视化 | 候选 mechanism deep dive |
| Trainer / full loop | Trainer 如何封装 dataloader、forward、loss、backward、optimizer、scheduler 与 evaluation？手写循环又暴露什么？ | 用户已完成 3/4 机制学习，并拆解 shuffle、`**batch`、forward/backward/optimizer/scheduler/zero-grad 顺序；详见 [`17-chapter3-finetuning-pretrained-model.md`](../topics/lora-feedback-loop/guides/04-lesson-lab/17-chapter3-finetuning-pretrained-model.md) | 完整训练运行结果、evaluation 与 Accelerate 证据按需补齐 | 已够推进，待运行复习 |
| learning curves | train/validation loss 与 accuracy 的不同组合如何暴露收敛、过拟合、欠拟合或训练震荡？ | 用户确认 Chapter 3 已读完；全章 guide 已提供诊断矩阵与单变量实验纪律 | 需要用户能从真实曲线提出原因假设、预期信号与下一次单变量实验 | 已读，待实验 |
| Hub model sharing | Hub Git 仓库、`from_pretrained()`、`push_to_hub()` 与 model card 如何共同支持版本化和复用？ | 用户确认 Chapter 4 已读完 | 尚未验收实际上传、版本选择与 model card 实践 | 已读，待实践 |
| local / remote dataset loading | 非 Hub 数据如何由文件格式、`data_files`、split mapping 与 `field` 转成 DatasetDict？ | 用户确认 Chapter 5 已读完；Agent 已验证 GitHub URL、raw URL、本地 `.json.gz`，排障见 [`18-chapter5-2-local-remote-dataset-loading.md`](../topics/lora-feedback-loop/guides/04-lesson-lab/18-chapter5-2-local-remote-dataset-loading.md) | 不再阻塞阅读；在 27k 数据复盘中验证实际 source、schema、split 与 session leakage | 博文 2、4 验收 |
| SFT objective | `system + context + output` 如何变成 next-token 监督，为什么能学到条件行为而不只是背答案？ | 用户确认完成真实 Qwen3 suggestion Full SFT 与 LoRA；Chapter 11 已读完 | 需要闭卷推导 causal shift、assistant-only loss、teacher forcing 和多答案边界 | 博文 1 验收 |
| Full SFT vs LoRA | 全参更新和低秩更新分别允许参数移动到哪里？任务距离、数据覆盖与优化自由度如何共同决定选择？ | Qwen3 1.7B / 27k 两种训练方式为用户确认事实 | 精确配置、结果、公平对照与因果解释尚未归档 | 博文 3 验收 |
| dataset policy engineering | suggestion 数据如何从样本集合变成可解释、可控制、可反馈的 action policy？ | 已形成四版候选 schema；历史讨论覆盖 Plan3/Plan0 与 action 分布 | 真实 27k 的 marginal/combination/eligibility、去重、split 和 counterfactual 证据待补 | 博文 4 验收 |
| model evaluation | 如何把 training objective、held-out generation、业务 decision quality 与端到端系统质量分开？ | Chapter 11 已读；真实评测存在但尚未脱敏归档 | 需要明确 eval 分母、路由条件、rubric、Judge 校准、checkpoint 和 failure slices | 博文 5、6 验收 |
| SFT vs GRPO | 示范 token likelihood 与 group-relative reward optimization 分别改变什么分布？ | 用户确认 Chapter 12 已读完 | 尚无运行或闭卷解释证据；需要讨论 reward hacking、KL 与 verifiable task 边界 | 博文 8 验收 |

## Mechanism Questions

- assistant-only loss 下，context token 不直接计 loss，为什么仍影响梯度？
- chat template、tokenizer、truncation、packing 与 loss mask 如何共同决定模型真正看到的训练分布？
- 如何证明 Full/LoRA 分数差来自更新空间，而不是学习率、数据序列化、checkpoint 或 eval 漂移？
- action marginal、combination 与 eligibility 如何共同暴露 teacher policy bias？
- SFT feedback loop 如何避免把 held-out failure 反复写回训练集造成测试集拟合？
- suggestion list 是否适合 GRPO；什么 reward 能被验证，什么偏好无法安全压缩成标量？

## Transfer Links

- 将 Causal LM / Trainer 训练链路迁移到闲鱼买家 Agent suggestion next action SFT / LoRA feedback loop。
- 真实迁移证据边界见 [`05-qwen3-suggestion-sft-practice-evidence.md`](../topics/lora-feedback-loop/guides/06-practice-transfer/05-qwen3-suggestion-sft-practice-evidence.md)。
- 掌握度通过 [`01-sft-blog-series-mastery-plan.md`](../topics/lora-feedback-loop/guides/08-review-loop/01-sft-blog-series-mastery-plan.md) 和 review question bank 验收。
