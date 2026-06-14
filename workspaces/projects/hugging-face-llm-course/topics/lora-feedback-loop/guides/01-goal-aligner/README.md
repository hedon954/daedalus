# 01 Goal Aligner：LoRA Feedback Loop Lab

## Learning Navigation

- Final artifact：mini LoRA fine-tuning feedback loop lab + runbook + 实验对比报告 + 业务迁移笔记。
- Current stage：01-goal-aligner。
- Current gap：小模型、数据集任务、训练环境和材料读取顺序尚未确认。
- Evidence needed：用户 review 并确认 task card；下一阶段选择材料入口。
- After this：进入 `02-source-scout`，围绕 lab 选择 Hugging Face LLM Course / PEFT / TRL / Transformers / Datasets 的最小材料集合。

## Confirmed Direction

本 topic 的主线不是“读完 Hugging Face LLM Course”，而是以它为主学习材料，完成一个可验证的训练反馈闭环：

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

DDIA 暂不作为 active topic。它只作为 ambient reading，用来维持系统设计直觉，不占用本 topic 的 WIP。

## Next Review Questions

请先 review `.daedalus/task-card.md`，然后确认或修改下面 3 个决策：

1. 本 topic 的小模型优先用本地可跑模型、云 GPU 小模型，还是先用教学级 toy model 证明闭环？
2. 数据集任务更贴近哪类问题：指令跟随、格式化输出、分类、摘要，还是某个 AI Agent 子任务？
3. 材料读取方式是顺序走 Hugging Face LLM Course，还是围绕 lab 反向读取 Course + PEFT/TRL 官方文档？

## Stop Rules

- 不在用户确认 task card 前进入 demo 编码。
- 不把 DDIA 升级为第二条 active topic。
- 不提前展开 RLHF / DPO；先把 LoRA feedback loop 跑稳。
