# Todo Path Board

Todo 是动态路径看板。学习证据变化、阶段完成、学习路径需要收窄或扩展时，都要同步更新它和 [`outcome-map.md`](outcome-map.md)。

> Agent 负责拆解、指导、排障和验收；用户负责关键实践、观察和手写笔记。不要把 Agent 自动完成的事项伪装成用户已经掌握。

## North Star

- Project：`hugging-face-llm-course`
- Topic：`lora-feedback-loop` - LoRA 微调与数据反馈闭环
- 最终产物：闲鱼二手买家 Agent suggestion next action production-shaped mini LoRA lab、训练闭环 runbook、实验对比报告、业务迁移笔记。
- 最小 lesson lab：HF Course 1/2 基础机制观察；后续迁移为 `suggestion dataset -> train -> eval -> error analysis -> feedback data -> retrain -> compare`。
- 业务迁移目标：让用户从 AI Agent 应用层进入模型训练闭环层，提升下一份 AI 开发岗位竞争力。

## Current Path

当前 active topic 已迁移为 course-learning：`01-need-aligner`、`02-syllabus-mapper`、`03-concept-roadmap` 已完成，当前处于 `04-lesson-lab`。

## Now

- 当前问题：用户希望先完成 Chapter 1/5 `How Transformers solve tasks` 里的 task labs，用机械式重复建立 Transformer 任务直觉。
- 为什么现在做它：不建立课程概念路线，后面 LoRA / SFT / DPO 仍会变成换 recipe 抄代码。
- 完成后解锁：能横向比较不同任务的 input、processor/tokenizer、head、loss、logits/generated output 和 postprocess，再回到 Chapter 2 拆 pipeline。

## Current Cursor

Current Cursor 是恢复定位器，不是完成证明。恢复或判断阶段状态时，必须用当前代码、测试、运行输出或用户已验证观察重新校准。

- Course frontier：已进入 `demo/hugging-face-course-learning`；用户完成 `transformer-work/casual_language_model.ipynb` 的 `max_steps=200` Causal LM quick training run。
- Latest lesson evidence：用户已完成 `transformer-work/question_answering.ipynb`，修正 `rajpurkar/squad` 数据集入口和 QA span label 边界判断，跑通 DistilBERT QA 小样本训练、Hub push，并观察到 `start_logits/end_logits` 独立 argmax 可能产生空 span；已记录 QA 后处理不是 golden output 的边界。
- Already wired：project/topic 已通过 daedalus lifecycle 初始化；task-card / outcome-map 已填入确认过的主线目标。
- Current open decision：Chapter 2 `Behind the pipeline` 后移；当前继续 Chapter 1/5 task lab sweep，下一步做 summarization lab。
- Current QA boundary：QA lab 已完成基础观察；若后续追求可靠指标，需要补完整合法 span postprocess 与 SQuAD EM/F1，而不是复现 task guide 的单次示例输出。
- Do not suggest：不要恢复旧仓库学习阶段命名；不要把 DDIA 升级为第二个 active topic。

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
- [ ] Chapter 1/5 task lab sweep：完成本节 8 个 task labs，形成跨任务输入/输出/head/postprocess 对比。
- [x] Lab 1/8 Text generation / Causal LM：已完成 DistilGPT2 quick training run、perplexity、Hub 上传。
- [x] Lab 2/8 Text classification：已完成 DistilBERT sequence classification、pipeline load、forward/logits probe、闭卷复现。
- [x] Lab 3/8 Token classification：已观察 label alignment、token-level logits 任务契约、模型保存和 `pipeline("ner")` postprocess；用户已完成复述。
- [x] Lab 4/8 Question answering：已观察 `offset_mapping`、`start_positions/end_positions`、`start_logits/end_logits`、空 span 后处理问题和官网示例输出不可作为 golden output 的边界。
- [ ] Lab 5/8 Summarization：观察 encoder-decoder generation。
- [ ] Lab 6/8 Translation：观察 seq2seq translation 与 summarization 的共性差异。
- [ ] Lab 7/8 Automatic speech recognition：观察 audio input、processor、generated transcript；先做轻量 pipeline / tiny sample。
- [ ] Lab 8/8 Image classification：观察 image processor、pixel_values、image logits；先做轻量 pipeline / tiny sample。
- [ ] Pipeline internals：手写 `pipeline("text-classification")` 的 tokenizer -> model -> postprocess 等价流程。
- [ ] Trainer batch 观察：在 notebook 中打印 `batch.keys()`、`input_ids/attention_mask/labels` shape 和 device。
- [ ] Trainer forward 观察：手动运行 `model(**batch)`，打印 `outputs.loss` 和 `outputs.logits.shape`。
- [ ] HF Course 1/2：完成 Transformer Models 与 Using Transformers 的基础学习。
- [ ] Causal LM recipe：解释并观察 dataset -> tokenizer -> blocks -> labels -> Trainer。
- [ ] Pipeline 机制：解释 task -> model/tokenizer/config -> inference -> postprocess。
- [ ] Baseline 固化：固定 model name，不依赖默认 pipeline model。
- [ ] 用户审核：用户至少审核 5-10 条样本，校准 keep/revise/weak/reject 口径。

## Stage Exit Criteria

- [x] 可以解释本任务为什么值得进入 active learning。
- [x] 可以说清最终产物和最小 demo。
- [x] 可以用验收标准判断是否进入 `02-syllabus-mapper`。
- [x] 用户 review 本次初始化内容，并确认 fine-tuning suitability、第一轮品类和质量标准。

## Done

- [x] 01-need-aligner：完成 topic 目标、业务任务、fine-tuning suitability、第一轮品类和核心质量标准确认。
- [x] 02-syllabus-mapper：完成材料入口、schema、工程形态、默认模型、训练环境和数据规模收敛。
- [x] 06-practice-transfer input：数据集与问题路线产物已迁入 course-learning 阶段目录。
- [x] 第一批候选样本：生成 20 条手机品类 suggestion next action 候选样本。
- [x] 第一批候选样本 v0.2：根据产品形态反馈，重生成短句数组版本。
- [x] 第一批候选样本 v0.3：根据 `{s,r}` target schema 生成当前有效版本。
- [x] 第一批候选样本 v0.4：根据“suggestion 是发给买家 Agent 的用户指令”修正当前有效版本。
- [x] Transformers pipeline smoke test：用户已跑通最小分类 pipeline。
- [x] 学习路径调整：先完成 HF Course 1/2，建立 Transformers 基础，再进入 LoRA。

## Canceled
