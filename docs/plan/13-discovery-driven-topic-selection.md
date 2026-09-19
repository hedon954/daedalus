# 澄清、挖掘、选题能力改进方案

> 日期：2026-06-14
> 状态：已落地

## 计划放置规则

daedalus 的正式实现计划放在 `docs/plan/` 下，作为项目长期设计资料和未来 Agent 的权威上下文。

`.codex/plans/` 只保留 Codex 执行追踪卡，用于 todo 状态管理；其中不重复详细方案，只引用 `docs/plan/` 中的正式计划。

## 背景判断

daedalus 现在已经有 `clarify-goal`、`gatekeeper`、`coach-questioning` 和 repo learning 的 01-03 阶段。它们能把一个相对清楚的学习意图整理成任务卡，也能判断一个候选是否值得进入 active learning。

但这套机制有一个更上游的缺口：它把“现实问题、当前基础、预期产出、学习目标”当成用户可以直接提供的输入。真实学习场景里，用户经常只带着模糊冲动、焦虑、被材料吸引的直觉、对未来职业的担心、对自己能力短板的隐约感受，以及一些互相冲突的候选方向。

因此，选题不应该只是 objective matching。选题应该先是 discovery：通过问题帮助用户发现自己到底在追求什么，然后才进入 task shaping 和 gatekeeping。

## 核心产品判断

真实诉求不是选题的输入，而是选题过程的产物。

daedalus 需要新增一层 `Topic Discovery Layer`：

```text
raw impulse
  -> discovery interview
  -> need hypotheses
  -> learner confirmation / correction
  -> topic candidates
  -> task shaping
  -> gatekeeper decision
  -> project/topic lifecycle
```

现有问题是：

- `clarify-goal` 太快把模糊意图压缩成任务卡。
- `gatekeeper` 太快进入 accept / defer / reject。
- `01-goal-aligner` 更擅长对齐 repo learning 目标，不够擅长探索“用户为什么想学这个”。
- `coach-questioning` 已经强调提问，但问题主要服务阶段学习，不专门服务选题前的内在诉求发现。

## 目标形态

当用户说“我不知道接下来学 DDIA 还是 Hugging Face Course”时，daedalus 不应直接给推荐。它应该先进入 discovery：

1. 复述用户的候选材料和已知背景，但明确这些只是线索，不是结论。
2. 提出 1-3 个高价值问题，探索用户真实场景、焦虑、欲望、能力缺口和产出偏好。
3. 将用户回答整理成若干 `need hypotheses`，并请用户确认、修正或排序。
4. 在用户的真实诉求更清楚之前，不创建 active topic，不直接推荐唯一答案。
5. 当诉求足够清楚后，再生成任务卡草案、候选 topic 切法、non-goals 和验收方式。

## 新增 Prompt：`topic-discovery.md`

新增文件：

```text
system/prompts/common/topic-discovery.md
```

职责：

- 在 `clarify-goal` 和 `gatekeeper` 之前运行。
- 处理用户只表达“想学某材料/某方向/两个方向之间摇摆”的场景。
- 允许多轮探索，不急于收敛。
- 输出可被 `clarify-goal` 消费的 discovery summary。

建议结构：

```markdown
---
title: Discover Learning Need
description: 在创建 project/topic 前，通过多轮澄清帮助用户发现真实诉求、能力缺口、现实场景和选题边界。
scope: common
---

# Discover Learning Need

## Agent Role

你是 daedalus 的选题探索教练。你的任务不是马上推荐学习材料，而是通过少量高质量问题，帮助学习者发现自己为什么被某些题目吸引、真正想改变什么、当前能力缺口在哪里，以及什么产出能证明这次学习有价值。

## Trigger

- 用户在多个学习方向之间摇摆。
- 用户只说想学某材料，但说不清现实问题。
- 用户表达职业焦虑、技术焦虑、能力短板、材料选择困难或方向混乱。
- 用户问“接下来学什么”“这个 topic 值不值得开”。

## Workflow

1. 不急着推荐；先承认模糊性本身是选题材料。
2. 复述已知线索，区分事实、Agent 猜测和待确认动机。
3. 每轮最多问 3 个问题；优先问能改变选题结论的问题。
4. 用户回答后，整理为 2-4 个 need hypotheses。
5. 要求用户确认、修正、排序或否定这些 hypotheses。
6. 只有当一个 hypothesis 能导向可验证产物时，才进入 clarify-goal。

## Output

```markdown
## Discovery Summary
- 用户原始表达：
- 已确认诉求：
- 待确认诉求假设：
- 情绪/现实压力：
- 能力缺口：
- 候选选题：
- 反目标 / 不想变成：
- 下一轮问题：
- 是否允许进入任务卡：
```

## Constraints

- 不要把用户最先说出的材料当成真实目标。
- 不要把学习者的焦虑简化成“长期能力主线”。
- 不要一次性发长问卷。
- 不要急着给 accept / defer / reject。
- 不要替用户决定目标；Agent 只能提出 hypothesis。
```

## Discovery Artifact

新增一种 pre-topic artifact。它不属于 active topic，因为 active topic 尚未成立。

建议路径：

```text
workspaces/discovery/<slug>.md
```

如果暂时不想新增目录，也可以先写入：

```text
workspaces/backlog/<slug>.md
```

但 backlog 的语义偏“候选方向”，不够表达多轮选题探索。因此建议新增 `workspaces/discovery/`，专门保存 topic 诞生前的探索记录。

artifact 内容：

```markdown
# <Discovery Title>

## Raw Signals

- 用户原始表达：
- 候选材料：
- 当前上下文：

## Need Hypotheses

| Hypothesis | Evidence | Confidence | User Status |
| --- | --- | --- | --- |
|  |  | low / medium / high | pending / confirmed / rejected |

## Questions

### Round 1

- Agent question:
- User answer:
- Updated hypothesis:

## Topic Candidates

| Candidate | Serves Need | Possible Artifact | Risk |
| --- | --- | --- | --- |

## Decision Readiness

- 是否可以进入 clarify-goal：
- 仍需澄清：
- 暂不学习：
```

## 问题库设计

问题库不应该像表单，而应该按选题风险组织。

### 1. 现实场景问题

- 最近哪类工作让你明显感觉自己不够有底？
- 如果这个能力补上了，你希望在哪个具体场景里用上？
- 你是在为当前项目、下一份岗位、长期技术判断，还是个人作品做准备？

### 2. 吸引力来源问题

- DDIA / HF 分别为什么吸引你？是好奇、焦虑、工作压力、经典权威、作品欲，还是补短板？
- 哪个题让你觉得“学完我会变成另一种工程师”？
- 哪个题只是你觉得“好像应该学”？

### 3. 能力缺口问题

- 你缺的是会用、会实现、会调优、会判断边界，还是会向别人解释？
- 你现在做 AI Agent 时，最不确定的是模型机制、后训练、评估、数据、系统可靠性，还是产品架构？
- 你怕自己只停留在 API glue code，还是怕底层系统能力在 AI 时代贬值？

### 4. 产出偏好问题

- 学完后你想留下什么：demo、技术文章、业务方案、面试作品、架构判断框架，还是可复用代码？
- 你更想要一个能跑的 mini lab，还是一套能迁移到工作决策的系统模型？
- 如果只能用一个产物证明这次学习有价值，它应该是什么？

### 5. 反目标问题

- 这次学习最怕变成什么：追热点、读完没用、太理论、太浅、demo 玩具化、无法迁移、长期拖延？
- 哪些内容即使很热门，这次也应该明确不学？
- 哪种学习方式会让你很快失去动力？

## 与现有模块的关系

### `topic-discovery` vs `clarify-goal`

- `topic-discovery` 负责发现真实诉求。
- `clarify-goal` 负责把已发现的诉求压缩成任务卡。

### `topic-discovery` vs `gatekeeper`

- `topic-discovery` 不做 accept / defer / reject。
- `gatekeeper` 只在 discovery summary 足够清楚后运行。

### `topic-discovery` vs `coach-questioning`

- `topic-discovery` 的问题服务选题前探索。
- `coach-questioning` 的问题服务已选 topic 内的学习推进。

### `topic-discovery` vs backlog

- backlog 保存未来候选。
- discovery 保存“为什么这个候选值得或不值得成为 topic”的探索过程。

## 验收标准

这个计划完成后，以下行为必须成立：

- 用户在多个学习方向之间摇摆时，Agent 不会直接给唯一推荐。
- Agent 每轮最多问 3 个能改变选题结论的问题。
- Agent 会明确区分用户已确认诉求和 Agent 假设。
- 选题前探索可以恢复到文件，而不是只存在聊天里。
- `clarify-goal` 和 `gatekeeper` 能消费 discovery summary。
- 只有当诉求 hypothesis 能导向可验证产物时，才创建 project/topic。
- 用 “DDIA vs Hugging Face Course / AI Tech Stack” dogfood 时，最终 topic 不是由材料热度决定，而是由用户确认的真实诉求决定。

## Dogfood Case：AI 浪潮下的工程师选题

用户当前原始表达包括两组张力：

1. AI 浪潮让软件工程师被迫追新，用户开始怀疑底层原理、架构设计、系统内功在 AI 时代是否仍然有用、有必要、有职业价值。
2. 用户正在做 AI Agent 开发，但主要停留在应用层，希望补齐 LLM 原理、微调、后训练等介于纯算法和纯工程之间的 AI Tech Stack 短板。

这不是简单的 DDIA vs HF 二选一，而是两个可能的真实诉求：

- 职业身份与长期内功确认：AI 时代软件工程师还应该如何建立不可替代的系统能力。
- AI Agent 技术栈补全：从应用层 glue code 进入模型机制、数据、训练、评估和部署的可控工程能力。

dogfood 时不应直接判断哪一个更重要，而要通过 discovery 确认：用户当前更需要稳定自己的长期工程身份，还是更需要补齐当前 AI Agent 工作中的直接短板，或者需要设计一个组合 topic，把两者连起来。

### Dogfood Round 1：用户确认后的真实诉求

用户回答第一轮 discovery 问题后，暴露出的核心诉求不是简单材料选择，而是 AI 时代的软件工程师身份重建：

- 短期现实压力：裁员可能随时到来，下一份工作的岗位市场明显向 AI 开发倾斜；尽快补齐 AI 技术栈，尤其是 Agent 应用层背后的模型、训练、后训练、评估等能力，是短期竞争力的直接来源。
- 长期能力张力：用户相信 DDIA 这类经过时间验证的系统设计知识更接近“道”，更可能学一通百；但又怀疑在 AI 浪潮下，这类传统底层能力是否仍有职业意义。
- 心理目标：这轮选题要帮助用户减少焦虑和怀疑，找回学习、工作、生活的主旋律，拥抱 AI 但不畏惧 AI。
- 能力目标：用户不想疲惫追热点，而是想成为能快速学习、同时掌握深度底层能力的人；这种人更可能把 AI 用好，也更能解决复杂问题。

由此形成的 need hypotheses：

| Hypothesis | Evidence | Confidence | User Status |
| --- | --- | --- | --- |
| 用户需要的不只是 AI 技术栈补课，而是“AI 时代工程师长期能力框架”。 | 用户同时担心短期就业和长期能力贬值，并明确说希望解决心理问题、找回主旋律。 | high | confirmed |
| 单独学习 DDIA 可能过于远离短期岗位压力，单独学习 HF 又可能落入追热点疲惫。 | 用户认为 HF/AI stack 短期最有竞争力，但 DDIA 更接近“道”；同时明确反感追热点疲惫。 | high | confirmed |
| 最合适的 topic 可能需要把 AI 技术栈作为主线，同时用系统设计第一性原理过滤哪些 AI 能力值得长期沉淀。 | 用户目标是快速学习 + 深度底层能力，而不是只会应用层或只读经典。 | medium | pending |

### Dogfood Round 2：学习边界和并行约束

用户进一步确认 AI Stack 内部优先级和现实学习场景：

- AI Stack 优先级：`fine-tuning / LoRA` -> `post-training / RLHF / DPO` -> `LLM 推理机制`。`eval` 已在公司实践过，不是主要短板。
- 期望 demo：mini fine-tuning lab，但不是只跑一次训练；要覆盖从初始化数据集、训练、线上/测试数据集、评测、反哺训练到循环迭代的完整闭环。
- DDIA 的现实位置：公司摸鱼时间不能做训练、看代码或看视频，但可以读 PDF。因此 DDIA 适合作为公司场景下的低摩擦阅读线。
- 当前疑惑：双线并行可能导致注意力分散；单线学习又会让两个长期任务都难以推进。

新的选题约束：

| Constraint | Implication |
| --- | --- |
| active WIP 仍应严格控制 | 不应把 AI Stack 和 DDIA 都建成 active topic。 |
| 学习场景不同 | AI Stack 适合高能量、可动手环境；DDIA 适合低摩擦阅读环境。 |
| AI Stack 有明确短期岗位价值 | 应作为主 active topic。 |
| DDIA 有长期“道”的价值但不适合重型实践 | 可作为 ambient reading / backlog / non-active track，不占 active topic。 |
| 用户担心注意力分散 | 需要定义双线边界、节奏和 stop rules，而不是简单禁止并行。 |

## 实施顺序

1. 新增 `system/prompts/common/topic-discovery.md`。
2. 调整 `clarify-goal`，要求在用户诉求明显混沌时先进入 discovery，而不是直接生成任务卡。
3. 调整 `gatekeeper`，要求只消费已澄清的 discovery summary，不负责挖掘真实诉求。
4. 调整 `01-goal-aligner`，让 repo/source 选题前置 discovery gate。
5. 增加 `workspaces/discovery/` 或等价 pre-topic artifact，并更新模板/文档。
6. 用 `LoRA Fine-tuning Feedback Loop Lab` 启动过程做 dogfood。

## 落地结果

- 新增 `system/prompts/common/topic-discovery.md`，把选题前的诉求探索独立成 prompt。
- 新增 `system/templates/discovery/item.md` 和 `workspaces/discovery/README.md`，让 discovery 记录可以恢复到文件。
- `clarify-goal`、`gatekeeper`、`01-goal-aligner` 已接入 discovery 边界：真实诉求不清楚时先探索，不直接推荐材料或创建 lifecycle。
- CLI 稳定 workspace layout 已纳入 `workspaces/discovery`，并由测试固定。
- 全局契约和 repo-learning skill 已说明 discovery / backlog / active topic 的语义分层。
