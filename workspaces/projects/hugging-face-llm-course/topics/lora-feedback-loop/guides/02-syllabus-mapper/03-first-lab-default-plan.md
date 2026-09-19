# 第一轮 Lab 默认方案

本文件给出 daedalus 主动推进后的第一轮默认方案。除非用户明确反对，后续进入 `06-practice-transfer` 和 demo 设计时按这里执行。

## 默认结论

第一轮目标不是追求线上效果，而是跑通一个 production-shaped 的 LoRA feedback loop：

```text
agent-generated reviewed dataset
  -> prompt/context baseline
  -> LoRA/SFT round 1
  -> fixed eval
  -> error analysis
  -> feedback dataset
  -> LoRA/SFT round 2
  -> comparison report
```

默认选择：

| 维度 | 默认方案 | 备选 |
| --- | --- | --- |
| 第一轮模型 | `Qwen/Qwen2.5-0.5B-Instruct` | 如果输出质量太弱，升级到 `Qwen/Qwen2.5-1.5B-Instruct` |
| 训练方式 | PEFT LoRA + TRL SFTTrainer | 如果 TRL 引入复杂度过高，退回 Transformers Trainer + PEFT |
| 训练环境 | Colab 或低成本单卡云 GPU | 本地只做数据、baseline、eval、报告，不强求训练 |
| 数据规模 | 初始 train 80 / eval 20 / holdout 20 | 如果训练不稳定，先缩到 train 40 / eval 10 / holdout 10 验证链路 |
| 输出 schema | `{"suggestions":[{"s":"用户口吻短指令","r":"短原因"}]}`，最多 3 条 | 如果小模型格式不稳，再降级为只输出 `s` |
| 企业平台 | 阿里云百炼作为第二阶段复现目标 | 第一阶段不以百炼为唯一入口 |

## 为什么默认 Qwen2.5 小模型

选择标准不是“最强”，而是“最适合第一轮闭环”：

- 中文任务友好，适合闲鱼二手买家 Agent 场景。
- 0.5B / 1.5B 体量较小，适合低成本 LoRA 实验。
- Hugging Face 上有 instruction-tuned 版本，baseline 更容易起步。
- 与 PEFT、Transformers、TRL 生态兼容，后续迁移和排障资料多。

风险：

- 0.5B 可能对复杂上下文理解不足。
- 小数据 fine-tuning 可能主要改善格式稳定性，不一定改善真实任务质量。
- 如果 baseline 已经很强，LoRA 改善可能不明显；这本身也是有效结论。

对应处理：

- 第一轮默认要求模型输出最多 3 条 `{s, r}`；`s` 必须像用户直接发给买家 Agent 的指令，`r` 必须短且解释当前为什么要这样做。
- 评测必须比较 baseline vs LoRA，而不是只看 LoRA 是否“看起来能用”。
- 如果 0.5B 无法形成基本质量，再升级到 1.5B，不直接跳到大模型。

## 材料入口

只读支撑当前 lab 的最小材料。

| 材料 | 用途 | 阅读方式 |
| --- | --- | --- |
| Hugging Face LLM Course | 建立 Transformers / Datasets / training 基础主线 | 只读和当前 lab 直接相关章节 |
| Transformers training scripts | 了解官方脚本化训练入口和小样本调试方式 | 看运行方式和参数组织，不照抄整套 |
| PEFT docs: LoRA | 理解 adapter、LoraConfig、保存/加载边界 | 重点读 LoRA quicktour 和 package reference |
| TRL SFTTrainer docs | 使用 SFTTrainer 跑 supervised fine-tuning | 重点读 dataset 格式、trainer 参数和最小脚本 |
| Qwen2.5 model cards | 确认模型加载、chat template、license/usage | 只确认第一轮模型可用性 |
| 阿里云百炼文档 | 第二阶段迁移边界 | 只记录 SFT/DPO/数据集/部署能力，不在第一轮展开 |

当前官方依据：

- Hugging Face Transformers 提供 example training scripts，并建议先用 `max_train_samples` 等参数在小样本上验证链路。
- Hugging Face PEFT 说明 LoRA 通过只训练少量额外参数降低训练成本。
- Hugging Face TRL 的 SFTTrainer 官方示例已经使用 Qwen 小模型作为 quick start。
- Qwen2.5 官方 Hugging Face model card 提供 0.5B 到 72B 的 base 和 instruction-tuned 模型。

## 第一批数据规模

默认先做 120 条样本：

```text
train: 80
eval: 20
holdout: 20
```

覆盖阶段：

| stage | 目标样本数 |
| --- | ---: |
| need_clarification | 10 |
| search | 10 |
| shortlist | 10 |
| compare | 15 |
| item_understanding | 15 |
| seller_chat | 15 |
| risk_check | 20 |
| order_decision | 10 |
| after_sale | 8 |
| dispute | 7 |

为什么不是更大：

- 用户需要审核 Agent 生成样本；样本越大，人工校准成本越高。
- 第一轮重点是 pipeline，而不是数据规模。
- 如果 120 条无法看出趋势，再用错误分析决定补哪些 stage，而不是平均扩充。

同时按难度覆盖：

| case 类型 | 目标样本数 | 用途 |
| --- | ---: | --- |
| typical | 60 | 让模型学会标准购买路径 |
| ambiguous | 30 | 训练模型在信息不足时先补证据 |
| boundary | 18 | 训练模型处理风险和收益冲突 |
| negative / unsafe | 12 | 训练模型学会暂停、避开或走维权路径 |

这两个维度要交叉覆盖。例如 `risk_check` 里应有 typical 风险确认，也应有 boundary 和 negative/unsafe；`order_decision` 里应有正常下单前确认，也应有脱离平台付款这类 negative case。

## 第一轮工程入口

后续 demo 应提供这些命令级入口：

```bash
python scripts/prepare_dataset.py --config configs/dataset.yaml
python scripts/run_baseline.py --config configs/baseline.yaml
python scripts/train_lora.py --config configs/train_lora_qwen_0_5b.yaml
python scripts/run_eval.py --config configs/eval.yaml
python scripts/build_feedback_set.py --config configs/feedback_round_1.yaml
python scripts/generate_report.py --run-a baseline --run-b lora_r1
```

每个脚本必须生成 artifact，不能只打印结果。

## 是否进入 03

`02-syllabus-mapper` 可以在满足下面条件后进入 `03-concept-roadmap`；业务样本与 baseline 设计在后续 `06-practice-transfer` 承接：

- 已有默认模型和升级路径。
- 已有默认训练环境。
- 已有第一批数据规模。
- 已有最小材料入口。
- 已有工程目录与 artifact 约定。

目前这些条件已具备。下一阶段应开始构建问题路线图和第一批数据生成/审核标准。
