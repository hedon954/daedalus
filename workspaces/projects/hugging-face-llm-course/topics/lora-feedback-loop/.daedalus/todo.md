# Todo Path Board

Todo 是动态路径看板。学习证据变化、阶段完成、学习路径需要收窄或扩展时，都要同步更新它和 [`outcome-map.md`](outcome-map.md)。

> Agent 负责拆解、指导、排障和验收；用户负责关键实践、观察和手写笔记。不要把 Agent 自动完成的事项伪装成用户已经掌握。

## North Star

- Project：`hugging-face-llm-course`
- Topic：`lora-feedback-loop` - LoRA 微调与数据反馈闭环
- 最终产物：SFT 博文系列、Qwen3 Full/LoRA 脱敏实验复盘、训练数据 preflight、评测与反馈闭环说明。
- 最小 review：从一条 suggestion 样本闭卷推导 `chat template -> tokens/labels -> loss -> parameter update`，再用真实实验验证。
- 业务迁移目标：让用户从 AI Agent 应用层进入模型训练闭环层，提升下一份 AI 开发岗位竞争力。

## Current Path

课程阅读已由用户于 2026-08-27 确认为 0–12 章全部完成；已有 lesson labs 和真实 Qwen3 Full/LoRA 迁移。专题已从 `08-review-loop` 中途搁置，不再占用 daily active 槽。接回后继续用博文而不是继续阅读来检验掌握度。

## Now

- 当前状态：`parked`。用户决定先释放 active 槽，启动下一题。
- 接回命令：`daedalus topic activate lora-feedback-loop`
- 接回后的问题：能否把课程知识和真实工作经验写成可验证、可反驳、可迁移的 SFT 技术文章。
- 完成后解锁：进入 capstone/closeout 判断，确认哪些结论真正掌握、哪些需要回补、哪些可以进入 knowledge-base 或公开博客。

## Current Cursor

Current Cursor 是恢复定位器，不是完成证明。恢复或判断阶段状态时，必须用当前代码、测试、运行输出或用户已验证观察重新校准。

- Reading frontier：Hugging Face LLM Course 当前 0–12 章全部阅读完毕（用户确认）。
- Local evidence：DistilGPT2、sequence classification、token classification、QA、summarization 等 labs 已保留；具体掌握边界见 `review/mastery-map.md`。
- Real-work evidence：用户确认用 Qwen3 1.7B、约 27k 数据执行过 Full SFT 与 LoRA；仅归档脱敏事实，精确结果仍待补。
- Current review：博文 1《SFT 到底学了什么：从 next-token loss 到行为蒸馏》。
- Lifecycle：`parked`；恢复点仍是 `08-review-loop` 博文 1。
- Do not suggest：不要再把“第一次跑 LoRA”或 Chapter 5/2 当作默认下一步；不要把未确认的 0.6B 历史实验与 1.7B 实验合并。不要在未 `activate` 前继续推进本 topic。

## Reminder Queue

这些事项是后续必须提醒用户亲自做的观察，不要因为 Agent 已解释过就标成完成。

- [x] Chapter 1/6 架构映射：用户已能用“理解 encoder、生成 decoder；extract 是 encoder，generate new token 是 decoder；强依赖完整 input 的生成可用 decoder 续写或 encoder + decoder 先理解后生成”归纳三类架构。
- [x] Chapter 1/6 attention mechanisms：完成 Agent-guided walkthrough；LSH / local / axial positional encodings guide 已留作回看。
- [x] Chapter 1/8 与 Chapter 2：用户确认已学完并正式进入 Chapter 3；未深挖项转为按需复习，不阻塞课程游标。
- [x] Chapter 3/1 Introduction：全章路线已预读，并生成独立 Chapter 3 guide。
- [x] Chapter 3/2、3/3 阅读进度：用户已读完并推进到 3/4。
- [ ] Chapter 3/2、3/3 实验回看：区分 preprocessing/training batch，保留 Trainer 的 loss、metric 与 checkpoint 证据；不阻塞当前阅读游标。
- [x] Chapter 3/4 full loop 机制：已拆解 dataset/DataLoader、shuffle、`**batch`、device、forward/loss、backward、optimizer、scheduler 与 zero-grad 顺序。
- [ ] Chapter 3/4 运行证据：完整训练结果、evaluation 与 Accelerate 观察按需补齐，不阻塞当前游标。
- [x] Chapter 3 剩余阅读：用户确认 3/5-3/7 已读完；曲线诊断实验保留待验收。
- [x] Chapter 4 阅读：用户确认整章已读完；Hub 上传/model card 实践保留待验收。
- [x] Chapter 5/1 Introduction：用户确认已读完。
- [x] Chapter 5–12 阅读：用户于 2026-08-27 确认全部读完；实践和掌握度在 review 中分别验收。
- [ ] 博文 1：写出中心论点、机制图和 SFT 最小伪代码。
- [ ] 博文 2：补一个真实、脱敏的 training sample preflight。
- [ ] 博文 3：补 Qwen3 1.7B Full/LoRA 实验矩阵和结果。

## Critical Checkpoint

Critical Checkpoint 只记录会影响后续判断的关键取舍，不写成聊天日志。

- Source constraint：Hugging Face/PEFT 示例通常默认数据、任务和评测已清楚；本 topic 必须把闲鱼买家 Agent 的输入、输出、偏好标准和评测边界显式化。
- Faithful imitation：必须忠实保留训练闭环的因果链，而不是只跑一次 fine-tune。
- Engineering standard：第一阶段可以低成本、小数据、小模型，但不能用 demo 标准放过工程结构；训练、评测、反馈构造和报告生成必须脚本化、配置化、产物化。
- Dataset difficulty：数据集不能只覆盖典型清晰 case；必须加入 ambiguous、boundary、negative/unsafe 样本，让模型学会补证据、暂停推进和处理风险冲突。
- Product shape：suggestion next action 是用户点击后直接发给买家 Agent 的自然指令，不是菜单标签；target 为最多 3 条 `{s, r}`，其中 `s` 是用户口吻指令，`r` 是短原因。
- Simplified / improved / discarded：简化模型规模和数据规模；暂时丢弃生产级平台和完整 RLHF。
- Transfer risk：迁移到真实业务前必须重新验证数据质量、评测一致性、成本、隐私和线上回归风险。

## Gaps Blocking Next Stage

- [x] 学习目标：LoRA / fine-tuning feedback loop 能力补齐。
- [x] 最小 demo：闲鱼二手买家 Agent suggestion next action 训练反馈闭环 lab。
- [x] 业务迁移目标：AI Agent 工程师模型层和训练层判断力。
- [x] 数据集任务：贴近 AI Agent 的 suggestion next action 生成。
- [x] Fine-tuning suitability：已形成 Agent 调研版判断，结论是先做 prompt/context baseline，再用 LoRA 判断稳定策略和格式偏好是否改善。
- [x] 第一轮品类：手机。
- [x] 核心质量标准：推动购买任务往前、多帮用户想一步，同时满足安全性、语义完整性和相关性。
- [x] 材料入口：确认课程/文档/示例组合。
- [x] 实验优先级：第一阶段先证明完整 pipeline。
- [x] 训练环境约束：可接受 Colab / 云 GPU，但需要成本可控。
- [x] 数据策略：Agent 生成候选样本，用户审核修改。
- [x] 企业平台候选：阿里云百炼可作为第二阶段迁移目标。
- [x] 工程标准：从第一阶段开始采用 script-first / config-first / artifact-first，不用 demo 标准放过自己。
- [x] Schema 草案：买家上下文、suggestion 输出结构、购买阶段枚举、rubric 和工程目录已形成草案。
- [x] 实验边界：默认模型、训练环境和第一批样例数据规模已确认。
- [x] 下一阶段入口：生成 `06-practice-transfer` 问题路线图、数据生成标准和 demo 设计前置问题。
- [x] 第一批样本：生成 20 条候选数据，供用户校准审核口径。
- [x] 难度覆盖：确认数据集需要覆盖 typical / ambiguous / boundary / negative-unsafe case。
- [x] 输出形态修正：确认 target 为 `{ "suggestions": [{ "s": "建议", "r": "原因" }] }`，最多 3 条。
- [x] v0.2 样本：按短句数组重生成 20 条候选样本。
- [x] v0.3 样本：按 `{s,r}` schema 生成候选样本。
- [x] v0.4 样本：按“用户口吻发给买家 Agent 的指令”重生成候选样本。
- [x] 运行证据：用户已跑通 Transformers `pipeline` smoke test。
- [x] 数据观察：用户已跑通 ELI5 `inspect_dataset.py`，确认原始数据结构。
- [x] Tokenizer 观察：用户已观察 `input_ids` / `attention_mask` / decode，并发现长序列 warning。
- [x] LM block 观察：用户已观察 `group_texts` 后 `input_ids` / `labels` 均为 128，且 `labels == input_ids`。
- [x] Collator / Trainer warning 观察：用户已观察 special token config 自动对齐 warning 与 MPS `pin_memory` warning，并确认它们不是训练失败。
- [x] Causal LM quick training run：用户已完成 `max_steps=200` 的 DistilGPT2 fine-tuning，得到 `training_loss≈3.98`、`perplexity≈47.81`，并上传到 Hugging Face Hub。
- [x] Trainer 拆解 guide：已迁入 `guides/04-lesson-lab/03-trainer-train-under-the-hood.md`。
- [x] Sequence classification lab：用户已完成 `transformer-work/sequence_classification.ipynb`，但需要做 derivation review。
- [x] Sequence classification pipeline reload：用户报告已跑通保存/加载或内存模型推理链路。
- [x] course-learning 迁移：project/topic state 已切换为 `course-learning` / `course-learning-topic`，并补齐 course shared maps 与新阶段目录。
- [x] Concept roadmap：已生成 `guides/03-concept-roadmap/README.md`，把 HF Course 1/2 的关键问题收束为课程概念路线。
- [x] Lesson lab：已生成 `guides/04-lesson-lab/README.md`，并迁移 Trainer batch / forward / loss / backward 观察入口。
- [x] Sequence classification derivation review：已补 raw/tokenized/collated/forward/logits 观察，并理解 loss/logits/argmax/id2label 链路。
- [x] Sequence classification closed-book rewrite：盖住教程，按任务契约重写最小 flow，并验收通过。
- [x] Chapter 1/5 task lab sweep：完成文本主线 5/8；剩余 Translation / ASR / Image classification 已按用户决定跳过/延后，不再阻塞 Chapter 1/6。
- [x] Lab 1/8 Text generation / Causal LM：已完成 DistilGPT2 quick training run、perplexity、Hub 上传。
- [x] Lab 2/8 Text classification：已完成 DistilBERT sequence classification、pipeline load、forward/logits probe、闭卷复现。
- [x] Lab 3/8 Token classification：已观察 label alignment、token-level logits 任务契约、模型保存和 `pipeline("ner")` postprocess；用户已完成复述。
- [x] Lab 4/8 Question answering：已观察 `offset_mapping`、`start_positions/end_positions`、`start_logits/end_logits`、空 span 后处理问题和官网示例输出不可作为 golden output 的边界。
- [x] Lab 5/8 Summarization：已观察 encoder-decoder generation、T5 task prefix、seq2seq labels、ROUGE、`compute_metrics` decode 边界和 `model.generate` 推理链路。
- [x] Lab 6/8 Translation：按用户决定跳过/延后；如后续补做，只作为 summarization 的 seq2seq 对照。
- [x] Lab 7/8 Automatic speech recognition：按用户决定跳过/延后；不作为当前 Transformers language foundation 阻塞项。
- [x] Lab 8/8 Image classification：按用户决定跳过/延后；不作为当前 Transformers language foundation 阻塞项。
- [x] Chapter 1/6 Transformer Architectures：三类架构映射和 attention mechanisms 已完成到可推进，后续弱点可回看 guide。
- [x] Chapter 1/7 Ungraded quiz：用户已越过并进入 Chapter 1/8。
- [x] Chapter 1/8 Deep dive into Text Generation Inference with LLMs：用户确认 Chapter 1 已完成。
- [x] HF Course 1/2：用户于 2026-07-13 确认完成 Transformer Models 与 Using Transformers。
- [x] Chapter 3/1 Introduction：已生成全章机制与实验路线 guide。
- [x] Chapter 3/2 Processing the data：用户确认已读完；观察证据待回看验收。
- [x] Chapter 3/3 Trainer API：用户确认已读完；训练与评估产物待回看验收。
- [x] Chapter 3/4 Full training loop：用户确认完成机制学习；运行与 Accelerate 证据按需补齐。
- [x] Chapter 3/5-3/7：用户确认已读完；learning curves 实验待回看。
- [x] Chapter 4 Sharing models and tokenizers：用户确认已读完；Hub 实践待回看。
- [x] Chapter 5/1 Introduction：用户确认已读完。
- [x] Hugging Face LLM Course 0–12：全部阅读完毕（用户确认）。
- [x] 真实 Qwen3 practice transfer：1.7B、约 27k、Full SFT 与 LoRA 均已执行（用户确认）。
- [ ] SFT 机制：通过博文 1 闭卷解释 dataset -> template -> tokens/labels -> loss -> update。
- [ ] Full/LoRA：通过博文 3 补齐公平实验矩阵、最佳 checkpoint 和结果解释。
- [ ] Dataset/Eval：通过博文 4–5 补齐 27k 数据画像、split、rubric 和 failure slices。
- [ ] Feedback loop：确认是否完成反馈数据再训练与独立 held-out 对比。

## Stage Exit Criteria

- [x] 可以解释本任务为什么值得进入 active learning。
- [x] 可以说清最终产物和最小 demo。
- [x] 可以用验收标准判断是否进入 `02-syllabus-mapper`。
- [x] 用户 review 本次初始化内容，并确认 fine-tuning suitability、第一轮品类和质量标准。

## Done

- [x] Course reading：用户于 2026-08-27 确认当前 Hugging Face LLM Course 0–12 章全部读完。
- [x] Real-work transfer：用户确认 Qwen3 1.7B、约 27k、Full SFT 与 LoRA 两种训练均已执行。
- [x] 01-need-aligner：完成 topic 目标、业务任务、fine-tuning suitability、第一轮品类和核心质量标准确认。
- [x] 02-syllabus-mapper：完成材料入口、schema、工程形态、默认模型、训练环境和数据规模收敛。
- [x] 06-practice-transfer input：数据集与问题路线产物已迁入 course-learning 阶段目录。
- [x] 第一批候选样本：生成 20 条手机品类 suggestion next action 候选样本。
- [x] 第一批候选样本 v0.2：根据产品形态反馈，重生成短句数组版本。
- [x] 第一批候选样本 v0.3：根据 `{s,r}` target schema 生成当前有效版本。
- [x] 第一批候选样本 v0.4：根据“suggestion 是发给买家 Agent 的用户指令”修正当前有效版本。
- [x] Transformers pipeline smoke test：用户已跑通最小分类 pipeline。
- [x] 学习路径调整：先完成 HF Course 1/2，建立 Transformers 基础，再进入 LoRA。
- [x] Chapter 1/5 scope cut：完成 5 个文本主线任务实验后，按用户决定跳过/延后 Translation / ASR / Image classification，进入 Chapter 1/6。
- [x] Chapter 1/6 architecture families summary：完成 encoder-only、decoder-only、encoder-decoder 的任务形态归纳。
- [x] Chapter 1/6 attention mechanisms walkthrough：完成 full attention / LSH attention / local attention / axial positional encodings 的慢动作解释。

## Canceled

- Chapter 1/5 Translation lab：2026-07-07 用户决定当前不做；后续如需要补 seq2seq 对照再恢复。
- Chapter 1/5 Automatic speech recognition lab：2026-07-07 用户决定当前不做；非文本多模态实验不阻塞本 topic。
- Chapter 1/5 Image classification lab：2026-07-07 用户决定当前不做；非文本多模态实验不阻塞本 topic。
