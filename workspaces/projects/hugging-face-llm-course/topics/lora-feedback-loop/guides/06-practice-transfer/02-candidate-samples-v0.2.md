# 第一批候选样本 v0.2

用途：根据用户反馈重生成 suggestion next action 样本。v0.2 的核心修正是：suggestion 必须是短句、可点击、可直接作为用户给 Agent 的下一步指令；每条样本最多 3 个 suggestion，且彼此互补。

v0.1 的问题：

- suggestion 太长，更像解释型建议。
- suggestion 无法直接点击发送给 Agent。
- 单条 suggestion 承载太多意图，没有形成互补行动组。

v0.2 的目标输出：

```json
{
  "target": {
    "suggestions": [
      "补充我的使用场景",
      "按预算推荐候选机型",
      "列出必须避开的风险"
    ]
  }
}
```

标注和评测侧仍保留 reason。更准确地说：

- `target.suggestions` 是 action-only 模型输出候选。
- `label_meta.action_reasons` 是逐条 reason，用于审核、评测和 action-with-reason ablation。
- `label_meta.stage_fit / risk_tags / priority / case_type / decision_mode` 是评测元信息。

如果后续实验显示小模型可以稳定输出短 reason，可以把训练目标切到 action-with-reason：

```json
{
  "suggestions": [
    {
      "action": "补充我的主力机场景",
      "reason": "当前用途不清晰"
    }
  ]
}
```

## 样本 01

```json
{
  "sample_id": "phone_seed_v02_001",
  "stage": "need_clarification",
  "task_goal": "买一台二手 iPhone 作为主力机",
  "context": "用户预算 3500-4500 元，只说想买 iPhone，没有说明用途、容量、续航要求和是否接受维修机。",
  "target": {
    "suggestions": [
      "补充我的主力机场景",
      "帮我确定容量底线",
      "列出不能接受的风险"
    ]
  },
  "label_meta": {
    "case_type": "typical",
    "decision_mode": "ask_for_evidence",
    "stage_fit": "need_clarification",
    "risk_tags": ["unclear_requirements", "budget_scope"],
    "priority": "high",
    "action_reasons": [
      "当前主力机用途不清晰，无法判断拍照、性能和续航优先级。",
      "容量会影响长期使用体验和二手价格，需要先确定底线。",
      "维修机、锁机和电池问题会直接改变是否值得继续搜索。"
    ],
    "review_reason": "当前需求不清晰，下一步应先补用途、容量和风险边界，而不是直接搜索商品。"
  }
}
```

## 样本 02

```json
{
  "sample_id": "phone_seed_v02_002",
  "stage": "need_clarification",
  "task_goal": "给父母买一台二手安卓手机",
  "context": "用户只说要便宜耐用，不确定是否需要大字体、长续航、双卡和售后方便。",
  "target": {
    "suggestions": [
      "确认父母常用功能",
      "优先筛长续航机型",
      "列出老人机避坑点"
    ]
  },
  "label_meta": {
    "case_type": "typical",
    "decision_mode": "ask_for_evidence",
    "stage_fit": "need_clarification",
    "risk_tags": ["usage_mismatch", "battery_life"],
    "priority": "medium",
    "action_reasons": [
      "父母的真实使用场景决定屏幕、续航、声音和系统流畅度优先级。",
      "长续航通常比性能参数更影响老人日常体验。",
      "老人机需要额外关注字体、存储、售后和误触等问题。"
    ],
    "review_reason": "给父母买手机需要先确认真实使用场景，再决定电池、屏幕、双卡和存储底线。"
  }
}
```

## 样本 03

```json
{
  "sample_id": "phone_seed_v02_003",
  "stage": "search",
  "task_goal": "搜索一台二手 iPhone 14 Pro 256G",
  "context": "用户预算 4000 左右，希望国行、无拆修、电池健康 85% 以上，但搜索结果很多，价格跨度从 3300 到 4600。",
  "target": {
    "suggestions": [
      "按硬性条件重新筛选",
      "标记异常低价商品",
      "生成搜索关键词"
    ]
  },
  "label_meta": {
    "case_type": "typical",
    "decision_mode": "compare_more",
    "stage_fit": "search",
    "risk_tags": ["abnormal_low_price", "filtering"],
    "priority": "high",
    "review_reason": "搜索阶段应先减少噪音，并把异常低价商品单独标记为后续风险确认对象。"
  }
}
```

## 样本 04

```json
{
  "sample_id": "phone_seed_v02_004",
  "stage": "search",
  "task_goal": "找一台二手小米或红米备用机",
  "context": "用户预算 800 元以内，想要续航好，但搜索时只按价格从低到高排序。",
  "target": {
    "suggestions": [
      "筛选大电池机型",
      "排除低存储版本",
      "过滤信息太少的商品"
    ]
  },
  "label_meta": {
    "case_type": "ambiguous",
    "decision_mode": "compare_more",
    "stage_fit": "search",
    "risk_tags": ["low_price_bias", "missing_condition_info"],
    "priority": "medium",
    "review_reason": "低价不是备用机的唯一标准，续航、存储和描述完整度会直接影响真实体验。"
  }
}
```

## 样本 05

```json
{
  "sample_id": "phone_seed_v02_005",
  "stage": "shortlist",
  "task_goal": "从 8 个 iPhone 候选里先筛掉明显不合适的",
  "context": "候选商品中有几台价格便宜，但描述包含“美版有锁”“换过屏”“商家批量出”等字样。",
  "target": {
    "suggestions": [
      "先排除有锁机",
      "筛掉不支持验机的",
      "保留信息完整商品"
    ]
  },
  "label_meta": {
    "case_type": "typical",
    "decision_mode": "pause_or_avoid",
    "stage_fit": "shortlist",
    "risk_tags": ["locked_phone", "screen_replacement", "seller_risk"],
    "priority": "high",
    "review_reason": "初筛阶段应先排除锁机、换屏不明和无法验机等高风险候选。"
  }
}
```

## 样本 06

```json
{
  "sample_id": "phone_seed_v02_006",
  "stage": "shortlist",
  "task_goal": "筛选二手安卓旗舰",
  "context": "用户看到几台骁龙旗舰，价格接近，但有的只写“功能正常”，没有任何成色、维修或电池说明。",
  "target": {
    "suggestions": [
      "按信息完整度排序",
      "追问维修和电池情况",
      "移除描述含糊商品"
    ]
  },
  "label_meta": {
    "case_type": "ambiguous",
    "decision_mode": "ask_for_evidence",
    "stage_fit": "shortlist",
    "risk_tags": ["information_asymmetry", "repair_history"],
    "priority": "medium",
    "review_reason": "信息不完整不等于一定不能买，但需要先补证据再进入对比。"
  }
}
```

## 样本 07

```json
{
  "sample_id": "phone_seed_v02_007",
  "stage": "compare",
  "task_goal": "比较三台 iPhone 14 Pro",
  "context": "三台价格分别为 3880、4050、4180 元。最低价电池 82%，中间价电池 88% 但无验机报告，最高价有原盒和保修截图。",
  "target": {
    "suggestions": [
      "生成三台对比表",
      "优先比较电池成本",
      "检查保修和验机证据"
    ]
  },
  "label_meta": {
    "case_type": "boundary",
    "decision_mode": "compare_more",
    "stage_fit": "compare",
    "risk_tags": ["battery_health", "warranty_evidence", "comparison"],
    "priority": "high",
    "review_reason": "价格差异不大时，应比较风险证据和后续使用成本，而不是只看标价。"
  }
}
```

## 样本 08

```json
{
  "sample_id": "phone_seed_v02_008",
  "stage": "compare",
  "task_goal": "在 iPhone 13 Pro 和 iPhone 14 之间选择",
  "context": "用户纠结性能、拍照和价格，候选商品价格接近，但没有考虑屏幕、重量、续航和维修成本。",
  "target": {
    "suggestions": [
      "按使用场景打分",
      "比较维修成本",
      "列出取舍理由"
    ]
  },
  "label_meta": {
    "case_type": "ambiguous",
    "decision_mode": "compare_more",
    "stage_fit": "compare",
    "risk_tags": ["model_tradeoff", "usage_fit"],
    "priority": "medium",
    "review_reason": "跨机型对比需要把参数讨论转成个人使用场景下的取舍。"
  }
}
```

## 样本 09

```json
{
  "sample_id": "phone_seed_v02_009",
  "stage": "item_understanding",
  "task_goal": "了解一台标称无拆修的 iPhone",
  "context": "卖家展示了外观图和电池健康截图，但没有主板、屏幕、摄像头和 Face ID 状态说明。",
  "target": {
    "suggestions": [
      "要求补功能检查视频",
      "追问是否拆修换件",
      "核对 Face ID 状态"
    ]
  },
  "label_meta": {
    "case_type": "typical",
    "decision_mode": "ask_for_evidence",
    "stage_fit": "item_understanding",
    "risk_tags": ["function_check", "repair_history"],
    "priority": "high",
    "review_reason": "外观和电池截图不足以证明整机状态，需要补关键功能和拆修证据。"
  }
}
```

## 样本 10

```json
{
  "sample_id": "phone_seed_v02_010",
  "stage": "item_understanding",
  "task_goal": "确认一台安卓手机是否值得继续沟通",
  "context": "商品描述写着“轻微烧屏不影响使用”，价格比同款低 300 元。",
  "target": {
    "suggestions": [
      "让卖家拍纯色屏幕",
      "确认烧屏具体位置",
      "评估低价是否值得"
    ]
  },
  "label_meta": {
    "case_type": "boundary",
    "decision_mode": "ask_for_evidence",
    "stage_fit": "item_understanding",
    "risk_tags": ["screen_burn_in", "condition_uncertainty"],
    "priority": "high",
    "review_reason": "轻微烧屏可能可接受，也可能影响体验，应先补证据而不是直接排除或下单。"
  }
}
```

## 样本 11

```json
{
  "sample_id": "phone_seed_v02_011",
  "stage": "seller_chat",
  "task_goal": "向卖家追问 iPhone 维修和锁机风险",
  "context": "用户担心直接问太多显得麻烦，卖家目前只回复“机器没问题”。",
  "target": {
    "suggestions": [
      "生成礼貌追问话术",
      "要求录屏设置页面",
      "确认已退出 Apple ID"
    ]
  },
  "label_meta": {
    "case_type": "ambiguous",
    "decision_mode": "ask_for_evidence",
    "stage_fit": "seller_chat",
    "risk_tags": ["id_lock", "managed_device", "communication"],
    "priority": "high",
    "review_reason": "卖家笼统承诺不够，需要用具体、礼貌的话术索取锁机和监管风险证据。"
  }
}
```

## 样本 12

```json
{
  "sample_id": "phone_seed_v02_012",
  "stage": "seller_chat",
  "task_goal": "和卖家沟通线下验机",
  "context": "卖家同城，支持面交，但用户不知道如何提出验机要求。",
  "target": {
    "suggestions": [
      "约定公开地点面交",
      "生成验机清单",
      "确认预留检查时间"
    ]
  },
  "label_meta": {
    "case_type": "typical",
    "decision_mode": "proceed",
    "stage_fit": "seller_chat",
    "risk_tags": ["offline_inspection", "transaction_safety"],
    "priority": "high",
    "review_reason": "线下交易可以推进，但应提前约定地点、验机项目和检查时间。"
  }
}
```

## 样本 13

```json
{
  "sample_id": "phone_seed_v02_013",
  "stage": "risk_check",
  "task_goal": "确认低价 iPhone 是否有隐藏风险",
  "context": "商品比同款低 600 元，卖家说“急出”，但不愿提供序列号截图，也不支持验机。",
  "target": {
    "suggestions": [
      "标记为高风险",
      "追问拒绝验机原因",
      "暂停继续议价"
    ]
  },
  "label_meta": {
    "case_type": "negative_unsafe",
    "decision_mode": "pause_or_avoid",
    "stage_fit": "risk_check",
    "risk_tags": ["abnormal_low_price", "evidence_refusal"],
    "priority": "high",
    "review_reason": "异常低价叠加拒绝提供证据是强风险信号，应先暂停推进。"
  }
}
```

## 样本 14

```json
{
  "sample_id": "phone_seed_v02_014",
  "stage": "risk_check",
  "task_goal": "判断一台“官换机”描述是否可信",
  "context": "卖家说手机是官换机，价格合理，但没有提供保修记录或官方换机凭证。",
  "target": {
    "suggestions": [
      "要求换机凭证",
      "核对保修记录",
      "不要只信口头描述"
    ]
  },
  "label_meta": {
    "case_type": "boundary",
    "decision_mode": "ask_for_evidence",
    "stage_fit": "risk_check",
    "risk_tags": ["replacement_device", "evidence_required"],
    "priority": "medium",
    "review_reason": "官换机并非一定不可买，但必须有可核验证据。"
  }
}
```

## 样本 15

```json
{
  "sample_id": "phone_seed_v02_015",
  "stage": "order_decision",
  "task_goal": "准备下单一台已沟通过的二手手机",
  "context": "卖家已补充照片和视频，价格谈妥，用户准备直接付款。",
  "target": {
    "suggestions": [
      "整理卖家关键承诺",
      "确认到货验机规则",
      "保存平台聊天记录"
    ]
  },
  "label_meta": {
    "case_type": "typical",
    "decision_mode": "proceed",
    "stage_fit": "order_decision",
    "risk_tags": ["evidence_preservation", "platform_record"],
    "priority": "high",
    "review_reason": "下单前应把承诺转成平台内可追溯记录，降低后续争议成本。"
  }
}
```

## 样本 16

```json
{
  "sample_id": "phone_seed_v02_016",
  "stage": "order_decision",
  "task_goal": "决定是否接受卖家要求脱离平台交易",
  "context": "卖家说平台手续费麻烦，希望用户微信转账并承诺更便宜 100 元。",
  "target": {
    "suggestions": [
      "拒绝脱离平台付款",
      "要求平台内继续交易",
      "保留卖家转账要求"
    ]
  },
  "label_meta": {
    "case_type": "negative_unsafe",
    "decision_mode": "pause_or_avoid",
    "stage_fit": "order_decision",
    "risk_tags": ["off_platform_payment", "fraud_risk"],
    "priority": "high",
    "review_reason": "脱离平台付款会削弱交易保护，不能为了小额优惠牺牲维权能力。"
  }
}
```

## 样本 17

```json
{
  "sample_id": "phone_seed_v02_017",
  "stage": "after_sale",
  "task_goal": "到货后检查二手 iPhone",
  "context": "用户刚收到手机，外观看起来正常，但还没确认功能和卖家承诺是否一致。",
  "target": {
    "suggestions": [
      "录制开箱验机视频",
      "逐项检查核心功能",
      "核对卖家原承诺"
    ]
  },
  "label_meta": {
    "case_type": "typical",
    "decision_mode": "ask_for_evidence",
    "stage_fit": "after_sale",
    "risk_tags": ["arrival_inspection", "evidence_preservation"],
    "priority": "high",
    "review_reason": "到货初期应先留证和核对承诺，避免错过争议处理窗口。"
  }
}
```

## 样本 18

```json
{
  "sample_id": "phone_seed_v02_018",
  "stage": "after_sale",
  "task_goal": "收到安卓手机后发现续航很差",
  "context": "卖家之前说电池正常，但用户半天就掉电很多，不确定是不是正常老化。",
  "target": {
    "suggestions": [
      "记录掉电过程",
      "截图电池使用情况",
      "对照卖家电池承诺"
    ]
  },
  "label_meta": {
    "case_type": "ambiguous",
    "decision_mode": "ask_for_evidence",
    "stage_fit": "after_sale",
    "risk_tags": ["battery_issue", "evidence_preservation"],
    "priority": "medium",
    "review_reason": "续航争议需要从主观感受转成系统记录和聊天承诺对照。"
  }
}
```

## 样本 19

```json
{
  "sample_id": "phone_seed_v02_019",
  "stage": "dispute",
  "task_goal": "处理到货后发现屏幕非原装的问题",
  "context": "用户验机发现屏幕可能不是原装，卖家之前承诺无拆修，现在卖家说“能用就行”。",
  "target": {
    "suggestions": [
      "整理无拆修承诺",
      "保存验机结果",
      "发起平台售后"
    ]
  },
  "label_meta": {
    "case_type": "negative_unsafe",
    "decision_mode": "dispute_or_escalate",
    "stage_fit": "dispute",
    "risk_tags": ["screen_replacement", "evidence_chain", "platform_dispute"],
    "priority": "high",
    "review_reason": "维权阶段要整理承诺、检测结果和不一致点，形成平台可判断的证据链。"
  }
}
```

## 样本 20

```json
{
  "sample_id": "phone_seed_v02_020",
  "stage": "dispute",
  "task_goal": "处理卖家拒绝承认隐藏 ID 锁风险",
  "context": "用户收到手机后发现无法正常退出某些账号状态，卖家之前没有说明相关限制。",
  "target": {
    "suggestions": [
      "停止重置和继续使用",
      "录屏保留账号异常",
      "联系平台说明不符"
    ]
  },
  "label_meta": {
    "case_type": "negative_unsafe",
    "decision_mode": "dispute_or_escalate",
    "stage_fit": "dispute",
    "risk_tags": ["id_lock", "evidence_preservation", "platform_dispute"],
    "priority": "high",
    "review_reason": "账号锁或限制状态会影响正常使用和所有权判断，应先保留原始证据再走平台流程。"
  }
}
```

## 用户审核模板

请按下面格式给反馈即可：

```text
001 keep
002 revise：第二条还是太泛，建议改成...
003 weak：三条之间重复
004 reject：不适合直接点击发送
```

v0.2 审核时请重点看：

- 每条是否足够短。
- 是否能直接点击发送给 Agent。
- 3 条之间是否互补。
- 是否缺少更关键的下一步动作。
