# 03 Socratic Coach：问题路线图

本阶段目标：把 `02-repo-scout` 的默认方案转成可执行的数据生成、用户审核、baseline 设计和 demo 前置问题。

> 本文件是 Agent 生成的问题路线图，不是用户学习笔记。用户自己的观察和理解应另行写入 notes。

## 当前默认方案

- 业务任务：闲鱼二手买家 Agent 的 suggestion next action。
- 第一轮品类：手机。
- 模型输出：只生成 `suggestion + reason`。
- 标注/eval 元信息：保留 `stage_fit / risk_tags / priority`。
- 第一轮模型：`Qwen/Qwen2.5-0.5B-Instruct`，必要时升级 `Qwen/Qwen2.5-1.5B-Instruct`。
- 第一批数据：train 80 / eval 20 / holdout 20。
- 工程形态：script-first / config-first / artifact-first。

## 本阶段要回答的问题

### 1. 数据是否真的覆盖购买任务？

每条样本都必须回答：

- 用户最终想买什么？
- 当前处于哪个购买阶段？
- 用户已经知道什么？
- 用户还不知道但应该知道什么？
- 当前最可能卡在哪里？
- 下一步行动是否能把用户推向“买到自己要的商品”？

不满足这些问题的样本，即使语言流畅，也不能进入训练集。

### 2. suggestion 是否真的有行动价值？

合格 suggestion 必须满足：

- 具体：用户知道下一句该问什么、下一步该查什么或该比较什么。
- 相关：不脱离当前候选商品、预算、阶段和约束。
- 前进一步：不是重复已有信息，而是解锁下一步判断。
- 多想一步：能指出用户可能忽略的风险、证据或决策点。
- 安全：不鼓励欺诈、骚扰、绕过平台规则或侵犯隐私。

### 3. reason 是否服务训练和审核？

`reason` 不是产品文案的装饰，而是训练闭环证据：

- 它应说明为什么这个 suggestion 适合当前阶段。
- 它应点出依据的风险、知识或用户卡点。
- 它不能只是复述 suggestion。

### 4. eval 是否能区分好坏？

第一轮 eval 至少要能区分：

- 格式正确但内容空泛。
- 和阶段相关但没有推动下一步。
- 覆盖风险但太泛，用户无法执行。
- 能行动但忽略安全或交易风险。
- 真正推动任务前进的高质量建议。

如果 eval 分不出这些差异，训练对比报告就不可信。

## 第一批数据生成标准

Agent 生成候选样本时必须按阶段覆盖：

| stage | 目标样本数 | 审核重点 |
| --- | ---: | --- |
| need_clarification | 10 | 需求、预算、用途是否明确 |
| search | 10 | 搜索关键词和筛选条件是否合理 |
| shortlist | 10 | 是否能排除明显不合适商品 |
| compare | 15 | 是否能比较价格、成色、风险和卖家可信度 |
| item_understanding | 15 | 是否能追问关键商品信息 |
| seller_chat | 15 | 是否能给出自然、有效的沟通动作 |
| risk_check | 20 | 是否覆盖手机二手交易高风险点 |
| order_decision | 10 | 是否能帮助用户做下单前确认 |
| after_sale | 8 | 是否能帮助到货检查和留证 |
| dispute | 7 | 是否能帮助整理证据和走平台规则 |

第一版不要追求每条都完美。目标是让样本覆盖足够多的真实购买状态，并让错误分析能发现缺口。

## 难度与边界 Case 覆盖

需要。只覆盖典型 case 会让数据集变得“太干净”，模型和 eval 都可能虚高。真实二手购买场景里，最需要 Agent 帮忙的往往是模糊、冲突和边界情况。

第一版数据集应同时覆盖：

| case 类型 | 含义 | 例子 | 目标占比 |
| --- | --- | --- | ---: |
| typical | 典型清晰场景 | 明确预算、明确机型、缺少验机证据 | 50% |
| ambiguous | 信息模糊但未必高风险 | 卖家描述含糊、用户目标摇摆、照片不完整 | 25% |
| boundary | 是否继续推进不明显 | 价格很香但证据不足、轻微维修但可接受、卖家态度一般 | 15% |
| negative / unsafe | 应该阻止或降级的场景 | 脱离平台交易、拒绝验机、疑似锁机、诱导隐私或违规 | 10% |

模糊和边界样本不是为了让模型“大胆决策”，而是训练它学会：

- 先补证据，而不是直接推荐买或不买。
- 明确哪些信息缺失会改变结论。
- 在风险和收益冲突时给出下一步验证动作。
- 遇到安全/合规风险时阻止推进购买。

对应到 label 侧，每条样本可以增加一个难度标签：

```json
{
  "label_meta": {
    "case_type": "ambiguous",
    "decision_mode": "ask_for_evidence"
  }
}
```

`decision_mode` 第一版建议只用少量枚举：

| decision_mode | 含义 |
| --- | --- |
| proceed | 可以继续推进 |
| ask_for_evidence | 先补证据 |
| compare_more | 需要更多候选对比 |
| negotiate | 可以进入议价或沟通 |
| pause_or_avoid | 应暂停或避开 |
| dispute_or_escalate | 售后/维权阶段应整理证据并走平台流程 |

## 用户审核方式

用户审核不需要逐字润色每条，而是按下面 4 类标记：

| 标记 | 含义 | 后续处理 |
| --- | --- | --- |
| keep | 可直接进入数据集 | 保留 |
| revise | 方向对，但需要修改 | 用户或 Agent 修改后再入库 |
| weak | 太泛、太浅或不够行动化 | 进入反例/错误分析 |
| reject | 事实错误、安全风险或脱离任务 | 丢弃 |

每 20 条样本做一次小结：

- 哪个 stage 样本最弱？
- 哪类 suggestion 最容易空泛？
- 哪些手机风险知识需要补充？
- 是否需要调整 schema 或 rubric？

## Baseline 设计问题

baseline 不能太弱，否则 LoRA 改善没有意义；也不能太强，否则第一轮小模型很难看出差异。

默认 baseline：

- 同一个 Qwen2.5 小模型。
- 使用 prompt + context + 少量手机风险知识。
- 不使用训练数据中的 label。
- 输出同样的 `suggestion + reason`。

baseline 必须保存：

- 输入样本。
- prompt 版本。
- 原始输出。
- 解析后输出。
- eval 分数。
- 错误样本。

## 进入 Demo 设计前的完成标准

- [ ] 生成第一批候选样本草案。
- [ ] 用户至少审核 20 条样本，校准 keep/revise/weak/reject 标准。
- [ ] 固定第一版 prompt baseline。
- [ ] 固定第一版 eval rubric 和 judge prompt。
- [ ] 明确哪些样本进入 train/eval/holdout。
- [ ] 明确第一版 demo 脚本入口和 artifact run id 约定。

## Agent 下一步

第一批 20 条候选样本已生成：

- [`01-candidate-samples-v0.1.md`](01-candidate-samples-v0.1.md)

下一步由用户按 `keep / revise / weak / reject` 审核样本。不要一次性扩展到 120 条；先用 20 条验证 schema、rubric 和审核口径。
