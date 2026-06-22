# 02 Source Scout：Hugging Face LLM Course 材料选择

## Learning Navigation

- Final artifact：手机品类二手买家 Agent suggestion next action 的 baseline + LoRA 对照实验。
- Current stage：02-source-scout / 02-repo-scout。
- Current gap：需要选择最小课程、文档和示例组合，不通读、不泛学。
- Evidence needed：材料清单、每个材料服务的实验环节、暂不阅读清单。
- After this：进入 `03-socratic-coach`，定义输入输出 schema、rubric、baseline 和数据集问题路线图。

## Confirmed From 01

- Fine-tuning 是最后选择；生产上先做 prompt/context/tooling baseline。
- 第一轮品类：手机。
- 核心质量标准：推动购买任务往前、多帮用户想一步，同时满足安全性、语义完整性和相关性。
- 学习策略：实践 + 理论闭环；每个实验步骤都要解释背后的训练链路。
- 工程策略：第一阶段就采用 script-first / config-first / artifact-first；实验规模可以小，但不能用 notebook demo 标准降低可复现、可迁移、可审计要求。

## Source Scout Goal

选择能支持下面实验闭环的最小材料集合：

```text
phone-shopping scenario dataset
  -> prompt/context baseline
  -> LoRA fine-tuning
  -> fixed eval set + rubric
  -> error analysis
  -> feedback data
  -> retrain + compare
```

## Primer

如果训练环境、模型选择、LoRA、PEFT、数据集和 eval 的概念还不清楚，先读：

- [`01-finetuning-lab-primer.md`](01-finetuning-lab-primer.md)

如果需要确认业务 schema、输出格式、评测 rubric 和工程目录，读：

- [`02-schema-and-engineering-shape.md`](02-schema-and-engineering-shape.md)

如果需要确认第一轮默认模型、训练环境、数据规模和材料入口，读：

- [`03-first-lab-default-plan.md`](03-first-lab-default-plan.md)

## Candidate Material Buckets

| Bucket | 作用 | 选择标准 |
| --- | --- | --- |
| Hugging Face LLM Course | 建立 tokenizer/model/datasets/training/eval 的课程主线 | 只读支撑当前 lab 的章节 |
| Transformers docs/examples | 模型加载、推理、Trainer 或 generation baseline | 只选一个可运行入口 |
| Datasets docs | 构造、切分、保存训练集和测试集 | 必须服务数据闭环 |
| PEFT / LoRA docs | adapter 配置、训练、保存、加载 | 必须能解释 LoRA 改了什么 |
| TRL docs | 后续 DPO/RLHF 入口 | 本阶段只标记，不提前展开 |
| 阿里云百炼 / Model Studio | 第二阶段候选训练、评测、部署平台 | 先确认 SFT/DPO/数据集/部署能力，不作为第一轮唯一入口 |
| OpenAI / external model optimization docs | 校准 fine-tuning 适用边界 | 只用于 critical lens |

## Engineering Shape Required

材料选择必须优先支持下面的工程形态：

- Script-first：训练、评测、反馈集构造、报告生成都有可重复执行的脚本入口。
- Config-first：模型名、数据路径、训练参数、评测集、输出目录都由配置文件控制。
- Artifact-first：训练集、测试集、baseline 输出、LoRA 输出、eval 结果、错误分析和再训练报告都落盘，并能通过 run id 串起来。
- Migration-ready：第一阶段本地/Colab 能跑，第二阶段迁移到百炼时尽量复用同一份数据 schema、rubric、评测集和报告结构。

不选择只能在 notebook cell 顺序里成立、依赖手工复制文件、路径写死或无法复现实验产物的材料作为主入口。

## Do Not Read Yet

- 完整 RLHF / DPO pipeline。
- 大规模分布式训练。
- 模型架构论文全集。
- 与手机 suggestion next action 无关的 leaderboard、benchmark 和新模型新闻。
- DDIA 正式 topic；DDIA 继续作为 ambient reading。

## User Questions Before Completion

1. 第一轮训练环境是什么：本地 Mac、Colab、云 GPU，还是先用 CPU/toy model 证明 pipeline？
2. 第一轮模型选择标准是什么：中文能力、体积小、Hugging Face 示例多、LoRA 支持好，还是部署成本低？
3. 你希望 source scout 优先找“能最快跑起来的示例”，还是“理论链路讲得最清楚的材料”？

## User Answers

- 第一阶段优先目标：先构建完整 pipeline，后续再考虑迁移到云上和真实业务效果。
- 训练环境约束：可以接受 Colab / 云 GPU，但需要控制成本。
- 数据集策略：先由 Agent 生成候选样本，用户审核修改。
- 训练平台候选：用户所在工作场景主要可能使用阿里云百炼平台，需要评估它是否适合作为第二阶段迁移目标。
- 工程形态确认：从第一阶段开始按生产工程标准设计，采用 script-first / config-first / artifact-first，而不是用 demo 标准放过自己。
- Schema 初稿：买家上下文包含购买任务、候选商品、品类、约束条件、购买阶段、已知/未知/卡点、对话上下文和可选知识注入；target 收敛为 `{"suggestions":[{"s":"短建议","r":"短原因"}]}`，最多 3 条；`s` 是用户点击后直接发送给买家 Agent 的用户口吻指令，`r` 解释当前为什么要这样做。
- 第一轮默认方案：`Qwen/Qwen2.5-0.5B-Instruct` 起步，`Qwen/Qwen2.5-1.5B-Instruct` 作为升级候选；Colab/低成本单卡云 GPU 训练；初始数据规模 train 80 / eval 20 / holdout 20；PEFT LoRA + TRL SFTTrainer 优先。

## Source Scout Decision

本阶段已收敛为下面的默认材料组合：

| 环节 | 默认材料 |
| --- | --- |
| 课程主线 | Hugging Face LLM Course 中 Transformers / Datasets / training 相关章节 |
| baseline 与模型加载 | Transformers docs/examples |
| 数据集构造 | Datasets docs |
| LoRA 训练 | PEFT LoRA docs |
| SFT 执行 | TRL SFTTrainer docs |
| 第一轮模型 | Qwen2.5 0.5B/1.5B Instruct model cards |
| 第二阶段迁移 | 阿里云百炼 Model Studio 文档 |

`02-repo-scout` 剩余动作不再继续泛搜材料；下一步应进入 `03-socratic-coach`，把默认方案转成可执行的问题路线图、数据生成标准和 demo 设计。
