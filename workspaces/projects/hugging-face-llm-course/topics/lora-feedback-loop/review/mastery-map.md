# SFT 掌握度地图

> 这是 review 仪表盘，不是 Agent 代写的学习结论。状态依据已有运行证据和用户确认事实保守标注；只有用户完成闭卷解释、最小复现或博文后才能升级。

## 状态含义

- `能复述`：脱离课程，用自己的语言讲清机制和边界。
- `能运行`：亲自跑通并能解释关键输入输出。
- `能改写`：能从空白重建最小实现或改变变量做消融。
- `能迁移`：能在真实业务中定义任务、数据、训练、评测并解释失败。

## 当前地图

| 能力 | 当前证据 | 当前判断 | 博文验收 | 仍需补齐 |
| --- | --- | --- | --- | --- |
| Transformer 任务与架构选择 | Causal LM、分类、token classification、QA、summarization labs；架构复述 | 能运行，部分能改写 | 博文 7 | 将任务形态、attention 可见范围、head/loss/postprocess 串成统一选择框架 |
| Tokenizer 与真实训练样本 | tokenizer/decode、BPE/WordPiece、LM block 观察；真实 SFT 排障线索 | 能运行；迁移证据待脱敏 | 博文 2 | chat template、length quantiles、loss mask、EOS、packing 的真实证据 |
| Trainer / full loop | 已拆解 forward、loss、backward、optimizer、scheduler；有真实 Full/LoRA 训练 | 能运行、能迁移 | 博文 1、3 | 从空白写最小伪代码；用曲线解释 checkpoint 选择 |
| SFT 机制 | 用户已完成真实 Qwen3 suggestion SFT，并持续追问机制 | 能迁移；闭卷解释待验收 | 博文 1 | next-token objective、conditional distribution、teacher forcing 与多答案边界 |
| Full SFT 与 LoRA | 用户确认 Qwen3 1.7B / 27k 两种方式均已训练 | 能运行、能迁移；对照结论未归档 | 博文 3 | 公平实验矩阵、结果、最佳 checkpoint、竞争性因果假设 |
| 数据集与 Policy 设计 | 四版候选 schema；历史讨论覆盖 Plan3/Plan0 与 action 分布 | 能迁移的迹象；正式证据不足 | 博文 4 | 真实 27k 数据画像、去重、session split、eligibility 和 counterfactual cases |
| 评测与反馈闭环 | 真实评测过程存在，但 1.7B 分数与反馈再训练尚未归档 | 待验收 | 博文 5、6 | eval protocol、failure taxonomy、第二轮反馈数据与前后对照 |
| Hub、Model Card 与 Demo | 课程已读；历史 DistilGPT2 上传 Hub | 能运行的局部证据 | 博文 6、7 | suggestion model 的脱敏发布/部署说明或明确不发布理由 |
| GRPO 与 SFT 边界 | Chapter 12 已读完（用户确认） | 阅读完成，尚未证明掌握 | 博文 8 | group sampling、advantage、reward/KL、适用性与 reward hacking |

## 当前 Review Checkpoint

- 目标：完成博文 1 的提纲和核心机制图。
- 最小复现：从一条 `system + user_context + assistant_output` 样本写出 tokenization、labels、shifted cross-entropy 的伪代码。
- 通过条件：能够解释为什么 SFT 可能泛化，也能解释什么数据分布会让它只学到 output prior。
- 回补入口：若无法解释 causal shift 或 loss mask，回到 `guides/04-lesson-lab/02-causal-lm-code-reading.md` 和真实样本 preflight。
