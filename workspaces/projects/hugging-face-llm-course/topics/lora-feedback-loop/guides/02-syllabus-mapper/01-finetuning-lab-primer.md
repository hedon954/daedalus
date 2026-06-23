# Fine-tuning Lab 基础知识 Primer

## 为什么先补这层

当前用户已经确认业务任务：手机品类二手买家 Agent 的 suggestion next action。下一步本来要选择训练环境、模型和材料，但用户明确表示自己是这方面小白，缺少背景知识。

因此本 guide 的目标不是做最终选型，而是建立最小判断地图。

## 一句话总览

这个 lab 有两条线：

```text
产品线：prompt/context baseline 能不能把 next action 做好？
训练线：LoRA fine-tuning 能不能让输出更稳定、更贴合偏好？
```

如果没有 baseline，fine-tuning 的效果无从判断。
如果没有固定 eval，训练前后只是主观感觉。
如果没有数据闭环，LoRA 只是跑了一次脚本。

## 核心概念

### 1. Base Model

Base model 是原始模型，例如一个小型开源 LLM。它已经具备通用语言能力。

本 topic 不从零训练模型，而是在 base model 上做适配。

选择 base model 时主要看：

- 能不能理解中文。
- 体积是否小到能训练。
- Hugging Face 上是否容易加载。
- 是否适合 LoRA / PEFT。
- 是否能满足第一轮教学实验，而不是追最强效果。

### 2. Prompt / Context Baseline

Baseline 是“不训练模型”的对照组。

它用：

- prompt：任务说明、输出格式、few-shot examples。
- context：商品信息、对话历史、用户意图、风险线索。

baseline 的作用是回答：不用训练，只靠大模型能力和上下文工程，能做到什么程度？

如果 baseline 已经足够好，生产上可能不需要 fine-tuning。
如果 baseline 有系统性问题，才值得用 LoRA 做对照。

### 3. Dataset

Dataset 是训练和评测的材料，不是随便写一些例子。

本 topic 至少要分：

- train set：用来训练。
- test set：训练时不能看，用来比较训练前后。
- feedback set：从错误案例里构造，用于第二轮训练。

对于 suggestion next action，每条样本大概需要：

```text
input:
  user_goal
  product_context
  seller_context
  conversation_history
  agent_reply
  risk_signals

output:
  suggested_next_actions
```

### 4. Eval / Rubric

Eval 是判断输出好坏的规则。没有 eval，就不知道 fine-tuning 是否真的变好。

当前核心 rubric 可以先包括：

- 推动购买任务往前。
- 多帮用户想一步。
- 安全性。
- 语义完整性。
- 相关性。
- 可执行性。
- 不过度打扰。

第一轮 eval 可以先人工打分或规则 + 人工混合，不必追求自动化完美。

### 5. Fine-tuning

Fine-tuning 是继续训练模型，让模型更适合某个任务。

它不是给模型塞实时知识，而是让模型学习稳定行为模式：

- 什么场景该建议什么动作。
- 输出格式如何稳定。
- 语气和策略偏好如何统一。
- 小模型如何更像目标任务专家。

### 6. LoRA

LoRA 是一种参数高效微调方式。

直觉理解：

- 不改动 base model 的大部分参数。
- 在模型里插入少量可训练的 adapter。
- 训练时主要更新 adapter。
- 保存时保存 adapter，而不是整个大模型。

这让训练成本更低，也更适合学习和实验。

### 7. PEFT

PEFT 是 Hugging Face 的参数高效微调工具集合，LoRA 是其中一种常见方法。

我们学 PEFT 不是为了背 API，而是为了理解：

- adapter 如何挂到模型上。
- 哪些参数被训练。
- checkpoint 保存了什么。
- 推理时 base model 和 adapter 如何组合。

### 8. TRL

TRL 更偏 post-training，例如 SFT、DPO、RLHF 等。

本 topic 第一阶段不展开完整 TRL pipeline。只有在 LoRA / SFT 闭环跑通后，才把 DPO 等作为后续问题。

## 训练环境怎么选

### 本地 Mac

优点：

- 启动快。
- 不依赖云环境。
- 适合写数据、跑 baseline、跑小脚本。

限制：

- 训练 LLM 很容易慢或显存不够。
- 适合 very small model / toy model / CPU pipeline proof，不适合稍微真实一点的 LoRA 训练。

### Colab

优点：

- 上手快。
- 有免费或低成本 GPU。
- Hugging Face 示例多。

限制：

- 环境不稳定。
- 文件持久化和复现实验要额外注意。
- 长时间训练可能中断。

### 云 GPU

优点：

- 更接近真实训练环境。
- 可控性更高。

限制：

- 成本更高。
- 环境配置和账单管理有额外负担。

### Toy Model / CPU Proof

优点：

- 最快证明 pipeline。
- 适合先理解数据、训练、eval、feedback 的因果链。

限制：

- 业务效果可能没有说服力。
- 不能代表真实 LLM fine-tuning 能力。

## 第一轮推荐策略

对小白最稳的路线不是直接追真实业务效果，而是：

```text
Step 1: 本地完成数据集 schema + prompt/context baseline + eval rubric
Step 2: 用 Colab 或小 GPU 跑最小 LoRA
Step 3: 比较 baseline vs LoRA
Step 4: 从错误样本构造 feedback set，再训一次
```

这样既不会纯理论，也不会纯跑脚本。

## 用户当前选择

- 第一阶段先证明完整 pipeline，而不是追求第一版业务效果。
- 可以接受 Colab / 云 GPU，但需要成本可控。
- 数据集先由 Agent 生成候选样本，再由用户审核修改。

这意味着第一轮 source scout 应优先寻找：

- 最快能跑通的数据集构造、baseline、LoRA、eval 示例。
- 低成本训练环境。
- 解释清楚 LoRA / dataset / eval 因果链的材料。

## 云成本直觉

对这个 topic 的第一阶段，云成本不应该高。目标是几十到几百条样本、小模型、小步数训练，不是大规模训练。

成本控制原则：

- 先本地完成数据 schema、baseline prompt 和 eval rubric。
- 只在真正训练 LoRA 时开 GPU。
- 优先 Colab 或 T4/L4/A10/4090 这类低成本 GPU。
- 不长时间挂着 notebook 或 pod。
- 每次训练前先用极小样本 dry run。

合理预期：

- pipeline proof：可能 0-10 美元级别。
- 小规模 LoRA 多次实验：大约 10-50 美元级别。
- 如果开始用 A100/H100 长时间训练，成本才会明显上升；这不属于第一阶段。

## 云迁移的工程改造成本

用户这里的“成本高”更准确地说是工程改造成本：从本地 / notebook / 教学 pipeline 迁移到云上训练，是否需要重写很多东西。

结论：

- 如果第一阶段写成一堆 notebook cell 和本地路径，后续迁移成本会偏高。
- 如果第一阶段从一开始就把数据、训练、评测、配置和产物保存拆清楚，迁移到 Colab / RunPod / Modal / Hugging Face Jobs 的成本不高。

工程上最容易迁移的是这种结构：

```text
demo/
  datasets/
    raw/
    processed/
  configs/
    train_lora.yaml
    eval.yaml
  scripts/
    prepare_dataset.py
    run_baseline.py
    train_lora.py
    run_eval.py
    build_feedback_set.py
  reports/
    baseline.json
    lora-round-1.json
    lora-round-2.json
```

这样本地和云上的差异主要变成：

- 安装依赖。
- 下载 / 挂载数据。
- 选择 GPU。
- 修改 config 里的模型名、batch size、输出路径。

不应该把核心逻辑绑死在：

- notebook cell 顺序。
- 绝对路径。
- 手工复制文件。
- 隐式环境变量。
- 只在某个云平台可用的 API。

第一阶段如果坚持“script-first + config-first + artifact-first”，迁移成本通常是中低的；如果 notebook-first 且无结构，迁移成本会高。

这里的原则不是“以后再工程化”，而是从第一版开始就用生产工程的标准约束自己：

- 可以使用小模型、小数据、小 GPU 时间，但不能让核心流程只存在于 notebook cell 里。
- 可以先不追求线上效果，但必须能复现实验、定位输入输出、比较报告和追溯数据版本。
- 可以晚一点迁移到百炼，但训练数据、评测集、rubric、配置和报告结构要从一开始为迁移留出边界。

换句话说，demo 的规模可以小，工程标准不能低。

## 阿里云百炼作为训练平台

阿里云百炼 / Model Studio 可以作为后续迁移目标。根据官方文档，它支持模型调优能力，包括：

- SFT：监督微调。
- CPT：继续预训练。
- DPO：偏好训练。
- 通过控制台、API 或命令行创建训练任务。
- 管理训练集和评测集。
- 管理调优后模型，以及导入本地训练的 LoRA 模型。
- 云端部署已调优模型。

但对本 topic 来说，百炼不一定适合作为第一阶段唯一入口：

- 第一阶段我们需要看清底层链路：数据如何进来、LoRA adapter 如何训练、eval 如何比较、feedback set 如何构造。
- 平台化训练会隐藏一部分训练细节，适合业务落地，但不一定最适合小白理解机制。
- 百炼更适合作为第二阶段：当本地/Colab pipeline 跑通后，把同一份数据 schema、评测集和训练目标迁移到企业平台。

因此推荐路径是：

```text
Stage 1: 本地/Colab script-first pipeline，理解完整链路
Stage 2: 百炼平台复现同一任务，学习企业训练平台的数据、任务、部署边界
```

如果第一阶段就直接上百炼，也不是不行，但要额外记录平台隐藏了哪些训练细节，避免只会点控制台。

## 模型选择的判断标准

第一轮不追最强模型，追可解释闭环。

优先级：

1. 容易加载和训练。
2. Hugging Face / PEFT 示例多。
3. 支持中文或至少能处理中文任务。
4. 体积小，能在低成本环境跑。
5. 输出足够稳定，便于 eval。

模型强不强不是第一优先。第一优先是能不能让你看清训练链路。

## 材料选择的判断标准

不要通读所有材料。每份材料必须服务一个实验环节：

| 实验环节 | 需要的材料 |
| --- | --- |
| 加载模型和生成 baseline | Transformers / LLM Course generation 相关内容 |
| 构造数据集 | Datasets 文档 |
| LoRA 训练 | PEFT LoRA 文档和示例 |
| 训练循环 | Trainer / SFTTrainer / 示例 notebook |
| 评测和对比 | LLM Course eval / 自定义 rubric |
| 后续 DPO/RLHF | TRL 文档，暂不展开 |

## 当前还不能决定的事

这些不是现在必须有答案：

- 最终用哪个具体模型。
- 是否上云 GPU。
- 最终数据集规模。
- 是否做 DPO。

现在只需要理解这些选项在解决什么问题。

## 下一步问题

用户不需要直接选技术栈。下一轮先回答更低门槛的问题：

1. 第一阶段你更想先证明 pipeline，还是希望第一版就有一点真实业务效果？
2. 你能接受使用 Colab / 云 GPU 吗，还是第一阶段必须尽量本地？
3. 你更愿意先手写小数据集，还是先让 Agent 帮你生成候选样本再由你审核？
