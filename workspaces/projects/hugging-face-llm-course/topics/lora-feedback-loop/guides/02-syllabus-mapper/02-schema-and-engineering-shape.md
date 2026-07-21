# Schema 与工程形态草案

本文件沉淀当前 source scout 阶段对业务输入、模型输出、评测 rubric 和工程目录的初版判断。它不是最终实现，但后续 lab 代码不能偏离这里定义的 production-shaped 约束。

## 设计原则

第一阶段可以小，但不能散。

- Script-first：核心动作必须通过脚本执行，而不是依赖 notebook cell 顺序。
- Config-first：模型、数据、训练参数、评测参数、输出路径都由配置驱动。
- Artifact-first：每一步输入输出都落盘，能用 run id 串起数据、模型、评测和报告。
- Migration-ready：本地/Colab 跑通后，迁移到阿里云百炼时尽量复用同一套数据 schema、rubric、评测集和报告结构。

这个设计参考了几个一线实践信号：

- Hugging Face Transformers 官方推荐从 example training scripts 入手，并用小样本参数先验证训练链路。
- Hugging Face Alignment Handbook 将 recipes、scripts、src、tests 分离，用 recipe configs 驱动训练与评测。
- MLflow Projects 强调用项目入口和参数封装可复现执行。
- ZenML 强调 pipeline、step、configuration、artifact、model 的职责分离，以及 artifact lineage。

## 买家上下文 Schema

一次买家上下文应该表达“用户正在完成什么购买任务，以及下一步建议为什么应该出现”。

建议第一版使用 JSONL，每行一个样本：

```json
{
  "sample_id": "phone_000001",
  "category": "phone",
  "task": {
    "goal": "买一台二手 iPhone 14 Pro 作为主力机",
    "budget_cny": [3500, 4300],
    "must_haves": ["电池健康不低于 85%", "无拆修", "支持当面验机"],
    "nice_to_haves": ["原盒配件", "保修未过期"],
    "hard_constraints": ["不接受监管机", "不接受隐藏 ID 锁风险"]
  },
  "stage": "compare",
  "progress": {
    "known": ["用户已找到 3 个候选商品", "用户知道大致预算"],
    "unknown": ["是否有维修记录", "电池是否更换过", "是否支持线下验机"],
    "blockers": ["候选商品价格接近，但风险信息不足"]
  },
  "candidates": [
    {
      "item_id": "xianyu_001",
      "title": "iPhone 14 Pro 256G 深空黑",
      "price_cny": 3980,
      "seller_claims": ["国行", "无拆修", "电池 88%"],
      "visible_risks": ["没有验机报告", "图片未展示边框细节"]
    }
  ],
  "conversation": [
    {
      "role": "user",
      "content": "这几个价格差不多，我不知道该选哪个。"
    },
    {
      "role": "assistant",
      "content": "先不要只按价格排序，可以优先排除维修、ID 锁和电池异常风险。"
    }
  ],
  "knowledge_context": {
    "injected": true,
    "scope": "stage_specific",
    "snippets": [
      {
        "id": "phone_risk_id_lock",
        "text": "二手 iPhone 需要确认是否退出 Apple ID、关闭查找功能，并避免监管机或隐藏锁风险。"
      }
    ]
  },
  "labels": {
    "target_actions": [
      {
        "suggestion": "让卖家录屏展示 Apple ID 已退出、查找功能已关闭，并说明是否为监管机。",
        "reason": "当前处于风险确认前的对比阶段，ID 锁和监管机是用户可能不知道但必须先排除的关键风险。"
      }
    ]
  },
  "metadata": {
    "source": "agent_generated_user_reviewed",
    "split": "train",
    "version": "v0.1"
  }
}
```

## 阶段枚举

第一版不做复杂动态知识检索，但必须把阶段显式化：

| stage | 含义 | 下一步建议重点 |
| --- | --- | --- |
| need_clarification | 需求澄清 | 预算、用途、硬性约束 |
| search | 搜索 | 关键词、筛选条件、价格区间 |
| shortlist | 初筛 | 排除明显不匹配和高风险商品 |
| compare | 对比 | 同款价格、成色、维修、电池、卖家可信度 |
| item_understanding | 了解单个商品 | 追问细节、要求补图、确认参数 |
| seller_chat | 与卖家沟通 | 问法、证据、议价、验货安排 |
| risk_check | 风险确认 | 假货、翻新、监管、锁、维修、售后风险 |
| order_decision | 下单决策 | 交易方式、验货、支付和收货检查 |
| after_sale | 售后 | 到货检查、问题留证、协商 |
| dispute | 维权 | 证据整理、平台规则、沟通路径 |

## 知识注入策略

第一版采用“显式字段 + 可选注入块”，不做复杂检索系统。

- 数据集中保留 `knowledge_context`，让模型和评测都知道本样本依据了哪些领域知识。
- 第一轮可以一次性注入少量手机品类风险知识。
- 第二轮再尝试按 `stage` 选择性注入。
- 不在第一阶段实现完整 RAG；否则会把 fine-tuning lab 变成检索系统项目。

## 输出 Schema

第一版训练输出不要过重。小模型容易在复杂 JSON 上失稳，且产品里的 suggestion next action 不是给用户看的菜单标签，而是用户点击后直接发送给买家 Agent 的下一步指令。

最终 target schema 收敛为“短 suggestion + 短 reason”的数组：

```json
{
  "suggestions": [
    {
      "s": "帮我让卖家补一张屏幕纯白图",
      "r": "缺少屏幕成色证据"
    },
    {
      "s": "帮我问问卖家是否支持线下验机",
      "r": "降低功能和成色争议"
    }
  ]
}
```

### 输出约束

- `suggestions` 是数组，最多 3 条。
- 每条包含 `s` 和 `r` 两个短字段。
- `s` 是用户口吻的短指令，点击后会直接发送给买家 Agent。
- `r` 是短原因，用于解释为什么这个 action 是当前好下一步。
- 多条之间要互补，不能只是同义改写。
- 如果当前只有一个最重要动作，可以只输出 1 条。
- `r` 不能把 `s` 重新写成长解释，也不能引入新事实。
- 不在模型输出中生成标签、评分或内部审核字段。

`s` 的判断标准：

- 好：`帮我把这三台按电池、保修和验机证据做个对比表`
- 好：`帮我生成一句礼貌话术，问卖家是否退出 Apple ID`
- 差：`生成三台对比表`
- 差：`确认已退出 Apple ID`

差的原因是它们更像内部动作标签，不像用户会直接发给 Agent 的自然指令。

复杂信息仍然放在标注和评测侧：

```json
{
  "label_meta": {
    "stage_fit": "item_understanding",
    "risk_tags": ["condition_uncertainty"],
    "priority": "high"
  }
}
```

权衡：

- `{s, r}` 比纯字符串更难一点，但仍然比长建议稳定。
- `s` 保持“用户发给 Agent 的自然指令”形态，`r` 保持训练和审核所需的解释信号。
- 字段名使用短名可以降低输出长度，也减少小模型格式负担。
- eval 仍然可以用 `stage_fit / risk_tags / priority` 判断输出是否贴合阶段、覆盖关键风险、优先级是否合理。

推荐边界：第一轮训练就使用 `{s, r}`，但严格控制 `s` 和 `r` 的长度；如果小模型格式不稳，再退回 action-only 作为降级方案。

## Eval Rubric

“推动购买任务往前”需要拆成可评测维度。

| 维度 | 问题 | 评分 |
| --- | --- | --- |
| relevance | 是否贴合当前购买任务、商品和阶段？ | 0-2 |
| stage_progress | 是否帮助用户进入下一步，而不是原地闲聊？ | 0-2 |
| hidden_need | 是否发现用户不知道但应该知道的关键点？ | 0-2 |
| domain_correctness | 手机/二手交易知识是否正确？ | 0-2 |
| risk_awareness | 是否覆盖必要风险，而不是只促成下单？ | 0-2 |
| actionability | 用户能不能直接照着做？ | 0-2 |
| safety | 是否避免违法、欺诈、隐私侵犯或危险建议？ | pass/fail |
| format | 是否满足输出 schema？ | pass/fail |

第一阶段可用 rule-based + LLM judge 混合评测：

- rule-based 检查 JSON schema、数量、空值、重复、禁用词。
- LLM judge 评分语义维度，但评分 prompt 和样本必须版本化。
- 用户人工审核一小批样本，用来校准 judge 是否跑偏。

## 工程目录草案

建议 lab 目录采用下面结构：

```text
demo/
  README.md
  pyproject.toml
  configs/
    baseline.yaml
    train_lora.yaml
    eval.yaml
    feedback_round_1.yaml
  src/
    buyer_agent_lab/
      __init__.py
      schemas.py
      data.py
      prompts.py
      baseline.py
      train.py
      evaluate.py
      feedback.py
      reporting.py
  scripts/
    prepare_dataset.py
    run_baseline.py
    train_lora.py
    run_eval.py
    build_feedback_set.py
    generate_report.py
  data/
    raw/
    interim/
    processed/
    eval/
  artifacts/
    runs/
      2026-06-14T000000Z_baseline/
      2026-06-14T010000Z_lora_r1/
      2026-06-14T020000Z_lora_r2/
  reports/
    baseline_vs_lora_r1.md
    lora_r1_vs_lora_r2.md
  tests/
    test_schemas.py
    test_eval_rules.py
```

目录职责：

- `configs/`：所有可调参数，便于本地、Colab、百炼迁移时替换执行环境。
- `src/`：可复用业务逻辑，不把核心逻辑塞进 scripts。
- `scripts/`：命令行入口，只做参数解析和调用。
- `data/`：数据生命周期分层；真实敏感数据后续不能直接入库。
- `artifacts/runs/`：每次运行的模型输出、预测、指标、错误样本和配置快照。
- `reports/`：人能读懂的对比结论。
- `tests/`：先覆盖 schema 和 eval rule 这两个最容易影响结果可信度的部分。

## Run Artifact 约定

每次运行至少生成：

```text
artifacts/runs/<run_id>/
  config.resolved.yaml
  dataset_manifest.json
  predictions.jsonl
  metrics.json
  errors.jsonl
  run_summary.md
```

训练运行额外生成：

```text
adapter/
trainer_state.json
train_metrics.json
```

这样后续迁移到百炼时，即使训练执行层换掉，也能继续保留同一套输入输出协议。

## 当前取舍

- 先不做完整 RAG，只把知识注入设计成 schema 字段。
- 先不做 DPO/RLHF，只做 baseline 与 LoRA/SFT 对照。
- 先不追最强中文模型，优先保证链路可解释、可复现、可迁移。
- 先不引入完整 MLOps 平台，但保留 artifact lineage 和配置化入口。
