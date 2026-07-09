# Syllabus Map

把课程 syllabus 转换为 daedalus 学习路线图。不要盲目照搬课程目录；要标明必学、可跳过、延后和迁移关系。

## Course

- URL：https://huggingface.co/learn/llm-course/en
- 学习策略：先完成 Chapter 1/2 的基础机制，再迁移到 LoRA/SFT feedback loop。
- 主线章节：1. Transformer Models；2. Using Transformers；3. Fine-tuning a pretrained model。
- 旁路章节：部署优化、认证考试、非当前任务必要的章节先延后。
- 暂不学习：完整 RLHF / DPO / 大规模分布式训练，等第一轮 SFT/LoRA 闭环跑通后再进入。

## Chapter Map

| Chapter / Section | 概念目标 | 依赖 | 必做练习 | 状态 | 证据 |
| --- | --- | --- | --- | --- | --- |
| Chapter 1 Transformer Models | 建立 pipeline、Transformer、Causal LM 和 LLM inference 直觉 | 无 | Text generation / Causal LM quick run；Chapter 1/5 文本任务观察；Chapter 1/6 架构归纳；Chapter 1/8 inference 主线 | in-progress | Chapter 1/5 已完成 5 个文本主线 lab；Chapter 1/6 架构与 attention mechanisms 已完成到可推进；Chapter 1/7 quiz 已越过；当前进入 Chapter 1/8 |
| Chapter 2 Using Transformers | 理解 tokenizer、model、batch、Trainer 的基本组合 | Chapter 1 | tokenizer / Trainer lesson lab | planned | 待补 |

## Stop Rules

- 不因为课程有章节就默认全量学习。
- 不把看完文字等同于掌握概念。
- 不把跑通示例代码等同于能迁移到真实任务。
