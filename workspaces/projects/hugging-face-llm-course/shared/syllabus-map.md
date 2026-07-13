# Syllabus Map

把课程 syllabus 转换为 daedalus 学习路线图。不要盲目照搬课程目录；要标明必学、可跳过、延后和迁移关系。

## Course

- URL：https://huggingface.co/learn/llm-course/en
- 学习策略：Chapter 1/2 基础已完成；当前进入 Chapter 3，把数据处理、Trainer 与手写训练循环连接到后续 LoRA/SFT feedback loop。
- 主线章节：1. Transformer Models；2. Using Transformers；3. Fine-tuning a pretrained model。
- 旁路章节：部署优化、认证考试、非当前任务必要的章节先延后。
- 暂不学习：完整 RLHF / DPO / 大规模分布式训练，等第一轮 SFT/LoRA 闭环跑通后再进入。

## Chapter Map

| Chapter / Section | 概念目标 | 依赖 | 必做练习 | 状态 | 证据 |
| --- | --- | --- | --- | --- | --- |
| Chapter 1 Transformer Models | 建立 pipeline、Transformer、Causal LM 和 LLM inference 直觉 | 无 | Text generation / Causal LM quick run；Chapter 1/5 文本任务观察；Chapter 1/6 架构归纳；Chapter 1/8 inference 主线 | done | 已有 task labs、架构归纳与 inference guide；用户确认已学完并进入 Chapter 3 |
| Chapter 2 Using Transformers | 理解 tokenizer、model、batch 和 forward 的基本组合 | Chapter 1 | tokenizer / model lesson lab | done | 已有 pipeline、tokenizer、model forward 与 task-head 观察；用户于 2026-07-13 确认完成 Chapter 2 |
| Chapter 3 Fine-tuning a pretrained model | 掌握数据预处理、Trainer API、完整训练循环与 Accelerate 的职责边界 | Chapter 1/2 | 依次观察 processing、Trainer、full loop 与 learning curves | active | 全章 guide 已生成；当前进入 Chapter 3/2 数据处理观察 |

## Stop Rules

- 不因为课程有章节就默认全量学习。
- 不把看完文字等同于掌握概念。
- 不把跑通示例代码等同于能迁移到真实任务。
