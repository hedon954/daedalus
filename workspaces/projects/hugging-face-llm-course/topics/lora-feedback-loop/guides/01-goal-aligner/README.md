# 01 Goal Aligner：LoRA Feedback Loop Lab

## Learning Navigation

- Final artifact：mini LoRA fine-tuning feedback loop lab + runbook + 实验对比报告 + 业务迁移笔记。
- Current stage：01-goal-aligner。
- Current gap：suggestion next action 输入输出 schema、质量标准、小模型、训练环境和材料读取顺序尚未确认。
- Evidence needed：用户 review 并确认 task card；下一阶段选择材料入口。
- After this：进入 `02-source-scout`，围绕 lab 选择 Hugging Face LLM Course / PEFT / TRL / Transformers / Datasets 的最小材料集合。

## Confirmed Direction

本 topic 的主线不是“读完 Hugging Face LLM Course”，而是以它为主学习材料，围绕闲鱼二手买家 Agent 的 suggestion next action 生成完成一个可验证的训练反馈闭环：

```text
initial suggestion dataset
  -> LoRA fine-tuning
  -> test / online-like dataset
  -> eval
  -> error analysis
  -> feedback data construction
  -> retrain
  -> compare
```

DDIA 暂不作为 active topic。它只作为 ambient reading，用来维持系统设计直觉，不占用本 topic 的 WIP。

## Business Task

- Agent：闲鱼二手买家 Agent。
- 用户目标：帮助买家搜索、判断、询问、比价、规避风险并推进二手商品购买决策。
- 训练任务：给定用户意图、商品上下文和 Agent 当前回复，生成 2-4 个下一步建议动作。
- 学习策略：先用小数据快速跑通完整闭环，再把任务定义和评测逐步迁移到更贴近真实岗位作品的程度；分阶段但不拆成多个 active topic。
- 理论边界：每次实践都要回到 LoRA、数据、评测和反馈链路的机制理解，避免只会跑脚本。

## Fine-tuning Suitability Gate

在进入 `02-source-scout` 前，必须先判断 suggestion next action 是否真的适合 fine-tuning：

- 如果目标是让模型记住商品事实、实时价格、卖家动态或平台规则，fine-tuning 不是主解法；这些应该来自 context engineering / retrieval / tools。
- 如果目标是让模型稳定学会“什么购物阶段该建议什么动作、如何识别风险、如何输出统一格式和语气、如何在大量相似场景下保持策略一致”，fine-tuning 可以作为候选解法。
- prompt engineering / context engineering 必须先做成 baseline；fine-tuning 只有在 baseline 出现系统性问题时才有生产意义。
- 本 topic 的学习价值在于跑通并理解这个判断链路，而不是预设 fine-tuning 一定优于 prompt。

详细调研见 [`02-secondhand-buyer-agent-research.md`](02-secondhand-buyer-agent-research.md)。

## Next Review Questions

请先 review `.daedalus/task-card.md`，然后确认或修改下面 3 个决策：

1. suggestion next action 的输入应该包含哪些字段：用户意图、商品卡片、卖家信息、对话历史、Agent 当前回复、风险线索？
2. 一个“好”的 suggestion next action 应该满足什么标准：可执行、贴合购买阶段、降低交易风险、信息增益高、不过度打扰、符合平台语气？
3. 第一阶段快速闭环的数据规模和模型规模应该多小，才能既快又能观察到训练前后差异？

## Stop Rules

- 不在用户确认 task card 前进入 demo 编码。
- 不把 DDIA 升级为第二条 active topic。
- 不提前展开 RLHF / DPO；先把 LoRA feedback loop 跑稳。
