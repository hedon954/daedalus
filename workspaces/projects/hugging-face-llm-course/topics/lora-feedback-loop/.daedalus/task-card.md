# 专题学习任务卡

- Project：`hugging-face-llm-course`
- Topic：`lora-feedback-loop`
- Topic 标题：LoRA 微调与数据反馈闭环
- 创建时间：`2026-06-14 14:42:45`
- 学习材料类型：课程 + 官方文档 + 开源代码示例
- 当前目标：以 Hugging Face / PEFT / TRL 等材料为入口，完成一个 LoRA fine-tuning feedback loop lab，掌握从数据构造、LoRA 训练、评测、错误分析、数据反哺到再训练对比的最小闭环。
- 现实问题：用户当前做 AI Agent 开发，但主要停留在应用层；短期需要补齐 AI 技术栈以应对岗位风险，长期希望用系统底层能力驾驭 AI，而不是疲惫追热点。
- 预期输出物：mini LoRA fine-tuning lab、训练闭环 runbook、初训 vs 反哺后再训练的实验对比报告、fine-tuning 适用边界和业务迁移笔记。
- 验收标准：能加载小型开源模型或教学模型；能构造初始训练集和独立测试集；能完成一次 LoRA 训练；能运行固定 eval；能基于错误样本构造反馈数据；能完成第二轮训练并比较差异；能记录改善、失败和未改善案例；能说明 non-goals 和迁移边界。
- 约束条件：active WIP 只承认本 topic；DDIA 只作为 ambient reading，不占 active topic；优先低成本可跑通的小模型；先做 LoRA/fine-tuning，后训练和推理机制暂不提前展开。
- 暂不学习：从零训练大模型、大规模分布式训练、完整 RLHF pipeline、追最新模型榜单、生产级训练平台、DDIA 独立 active topic。
- 用户确认：已确认主线为 Hugging Face / AI Stack 学习并完成 LoRA feedback loop lab；DDIA 作为公司碎片时间的低摩擦旁路阅读。

## Topic 学习目标

- 希望获得的能力：从 AI Agent 应用层进入模型训练闭环层，理解 fine-tuning 的数据、训练、评测、反馈和版本化边界，并能判断什么时候该 fine-tune、什么时候该用 prompt/RAG/tooling。
- 继承的 shared context：暂无；本 topic 将沉淀 Hugging Face LLM Course project 的第一批 shared context。
- 需要补充的源码入口：Hugging Face LLM Course、Transformers、Datasets、PEFT、TRL 官方文档和最小 LoRA 示例；具体材料在 `02-source-scout` 阶段确认。
- 运行/调试要求：在用户可控的非公司环境中跑通训练；优先选择本地或低成本云环境能承受的小模型和小数据集；所有训练命令、环境约束和失败案例写入 runbook。
- Mini demo 方向：`initial dataset -> LoRA fine-tuning -> test / online-like dataset -> eval -> error analysis -> feedback data construction -> retrain -> compare`。

## 角色边界

- daedalus 应该做：拆解学习路径、生成行动指南、协助排障、验收阶段产物。
- 用户必须亲自做：确认目标、运行关键命令、观察结果、回答核心问题、形成自己的笔记。
- daedalus 可以协助但不能代替：源码拉取、环境启动、问题回答、架构总结、demo 设计。
