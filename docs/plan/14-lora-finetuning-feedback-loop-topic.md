# LoRA Fine-tuning Feedback Loop Lab 启动计划

> 日期：2026-06-14
> 状态：计划中

## 背景

用户当前不是单纯想“学 Hugging Face Course”，而是在 AI 浪潮、岗位压力和长期能力信仰之间寻找新的学习主旋律。

短期现实压力是：下一份工作的岗位市场明显偏向 AI 开发，用户当前做 AI Agent，但主要停留在应用层，缺少模型机制、fine-tuning、post-training 等介于算法和工程之间的能力。

长期能力张力是：用户相信 DDIA 这类系统设计知识更接近“道”，更可能学一通百；但在 AI 浪潮下，用户也怀疑传统底层能力是否仍有职业意义。

因此，新 topic 不应被定义为“读 Hugging Face Course”，而应定义为一个能同时提升短期岗位竞争力和长期工程自信的可验证实践闭环。

## 选题结论

主线 active topic：

```text
LoRA Fine-tuning Feedback Loop Lab
```

材料入口：

- Hugging Face Course / TRL / PEFT / Transformers / Datasets 等官方材料。
- 必要时补充论文、官方文档和成熟开源示例。

学习姿态：

```text
用 DDIA 式的第一性原理，学习 AI Tech Stack 中最稳定、最可迁移的底层机制。
```

DDIA 暂不作为 active topic。它作为 ambient reading 旁路，在公司低摩擦阅读 PDF，维持系统设计语感，不占用 daedalus active WIP。

## 目标

完成一个 mini fine-tuning lab，跑通从数据到训练、评测、错误分析、数据反哺和再训练的闭环：

```text
initial dataset
  -> LoRA fine-tuning
  -> test / online-like dataset
  -> eval
  -> error analysis
  -> feedback data construction
  -> retrain
  -> compare
```

这个 topic 的核心目标不是训练出强模型，而是掌握可迁移的训练闭环能力：

- 能判断什么问题适合 fine-tuning，什么问题不适合。
- 能构造一个最小但可解释的数据集。
- 能跑通 LoRA 训练链路，并理解 adapter、base model、checkpoint、inference 的边界。
- 能设计测试集和评测指标，识别训练是否真的改善目标行为。
- 能从错误样本构造反馈数据，并比较再训练前后的变化。
- 能理解这个闭环里的工程不变量、数据风险和迁移边界。

## AI Stack 优先级

本轮优先级：

1. `fine-tuning / LoRA`
2. `post-training / RLHF / DPO`
3. `LLM 推理机制`

`eval` 已在公司有实践，不作为主要短板，但 lab 必须包含最小评测，因为没有 eval 就无法形成训练反馈闭环。

## 非目标

本 topic 暂不追求：

- 从零训练大模型。
- 大规模分布式训练。
- 复杂 RLHF pipeline。
- 追最新模型、最新榜单或最新训练技巧。
- 把 demo 做成生产级训练平台。
- 用 DDIA 另开完整 active topic。

## DDIA 旁路规则

DDIA 作为 ambient reading，不占 active topic。

规则：

- 只在公司适合读 PDF 的碎片时间阅读。
- 不要求每次产出笔记。
- 不因为 DDIA 没读完而阻塞 LoRA lab。
- 如果有精力，可以记录 1-3 条对 AI 训练闭环、数据系统或 Agent 架构有启发的原则。
- 只有当某个 DDIA 问题强烈连接到当前 lab，例如数据版本、评测一致性、反馈循环、可靠性边界，才提升为当前 topic 的一条 external lens。

DDIA 的当前作用是维持系统设计直觉，而不是承担完整学习闭环。

## 建议 project/topic 形态

这不是现有 `openai-codex-cli-deep-learning` project 的后续 topic，因为学习材料和最终产物不再是 Codex repo。

建议新建一个 learning project，例如：

```text
ai-tech-stack-deep-learning
```

初始 topic：

```text
lora-feedback-loop
```

候选标题：

```text
LoRA 微调与数据反馈闭环
```

## 预期目录和产物

进入 lifecycle 后，topic 内至少应形成：

```text
topics/lora-feedback-loop/
  .daedalus/task-card.md
  .daedalus/outcome-map.md
  .daedalus/todo.md
  guides/
    01-goal-aligner/README.md
    02-source-scout/README.md
    03-question-roadmap/README.md
    07-demo-architecture/README.md
  notes/
    03-question-roadmap/README.md
    04-runbook/README.md
    08-demo-coder/README.md
    09-transfer/README.md
  demo/
    README.md
    datasets/
    train/
    eval/
    scripts/
    reports/
```

最终关键产物：

- 一套 mini LoRA fine-tuning lab。
- 一份训练闭环 runbook。
- 一份实验对比报告：初训 vs 反哺后再训练。
- 一份业务迁移笔记：什么时候应该 fine-tune，什么时候应该 prompt/RAG/tooling，什么时候应该不做。
- closeout 后再判断哪些内容进入 `knowledge-base/`。

## 关键学习问题

第一阶段不从工具 API 开始，而从问题开始：

1. 什么行为问题值得用 LoRA fine-tuning 解决？它和 prompt engineering / RAG / tool use 的边界是什么？
2. 一个训练样本到底在教模型什么？错误样本如何变成下一轮数据，而不是污染数据集？
3. LoRA adapter 改变的是哪些参数路径？它和 base model、checkpoint、inference serving 的边界是什么？
4. 评测如何避免只验证训练集记忆？测试集、线上样本和回归样本分别保护什么？
5. 反馈闭环里的数据版本、实验版本和模型版本如何保持可追溯？
6. 这个 mini lab 哪些机制值得迁移到真实业务，哪些只是教学简化？

## Demo 验收标准

最小验收：

- 能加载一个小型开源模型或教学模型。
- 能构造初始训练数据集和独立测试集。
- 能完成一次 LoRA fine-tuning。
- 能在固定测试集上运行 eval，并输出可比较指标或结构化判定。
- 能基于错误样本构造反馈数据。
- 能进行第二轮训练，并比较第一轮和第二轮差异。
- 能记录失败案例、改善案例和未改善案例。
- 能说明当前 demo 的 non-goals 和迁移边界。

## 风险和边界

### 算力风险

本 topic 应优先选择可在本地或低成本云环境跑通的小模型。目标是学习闭环，不是追求 SOTA。

### 工具复杂度风险

Hugging Face 工具链很容易把学习带入 API 细节。每个工具学习都要回到机制问题：数据如何流动，参数如何更新，adapter 如何保存，评测如何证明行为变化。

### 追热点风险

新材料只在服务当前闭环时引入。RLHF / DPO 放在 LoRA 闭环跑通后，不提前展开。

### 注意力分散风险

active WIP 只承认 LoRA lab。DDIA 不制造待办压力。

## 启动前需要确认

1. 具体选择哪个小模型作为教学对象。
2. 数据集任务选择：分类、摘要、指令跟随、格式化输出，还是某个贴近用户业务的 agent 子任务。
3. 训练环境：本地 Mac、个人 GPU、云 GPU、Colab，或公司外的其他环境。
4. 是否使用 Hugging Face Course 作为顺序教材，还是以 lab 需求反向读取对应章节和官方文档。

## 建议启动动作

1. 先用 daedalus 创建新 project/topic，而不是把它塞进 Codex CLI project。
2. 进入 `01-goal-aligner`，把本计划中的真实诉求、非目标和验收标准压缩成 task card。
3. 进入 source/material scout，选择 Hugging Face 课程章节、PEFT/TRL 官方文档和一个小模型示例。
4. 进入 question roadmap，先回答关键学习问题，再设计 demo。
5. demo 只在 `fine-tuning / LoRA` 闭环稳定后，再考虑 DPO 或其他 post-training。
