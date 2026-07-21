# 第一批候选样本 v0.4

用途：根据用户反馈，将 `s` 修正为“用户点击后直接发送给买家 Agent 的自然指令”。

v0.4 规则：

- target schema：`{ "suggestions": [{ "s": "建议", "r": "原因" }] }`。
- `s` 不是菜单标签，而是用户口吻的指令。
- 点击某条 suggestion 后，`s` 会作为用户消息发给买家 Agent。
- `s` 应减少交互摩擦，让用户不用自己组织下一句话。
- `r` 是短原因，解释为什么这个 action 是当前好下一步。
- 最多 3 条，彼此互补。

## 样本 01

```json
{
  "sample_id": "phone_seed_v04_001",
  "stage": "need_clarification",
  "task_goal": "买一台二手 iPhone 作为主力机",
  "context": "用户预算 3500-4500 元，只说想买 iPhone，没有说明用途、容量、续航要求和是否接受维修机。",
  "target": {
    "suggestions": [
      {"s": "帮我先问清楚这台主力机最该满足哪些使用场景", "r": "用途会决定机型优先级"},
      {"s": "帮我判断 3500-4500 预算适合看哪些 iPhone", "r": "先缩小候选范围"},
      {"s": "帮我列出买二手 iPhone 必须提前排除的风险", "r": "先明确维修锁机等红线"}
    ]
  },
  "label_meta": {
    "case_type": "typical",
    "decision_mode": "ask_for_evidence",
    "stage_fit": "need_clarification",
    "risk_tags": ["unclear_requirements", "budget_scope"],
    "priority": "high"
  }
}
```

## 样本 02

```json
{
  "sample_id": "phone_seed_v04_002",
  "stage": "search",
  "task_goal": "搜索一台二手 iPhone 14 Pro 256G",
  "context": "用户预算 4000 左右，希望国行、无拆修、电池健康 85% 以上，但搜索结果很多，价格跨度从 3300 到 4600。",
  "target": {
    "suggestions": [
      {"s": "帮我按国行、无拆修、电池 85% 以上重新筛一遍", "r": "先减少不匹配商品"},
      {"s": "帮我把明显低于市场价的商品标出来", "r": "低价可能隐藏风险"},
      {"s": "帮我生成一组更精准的闲鱼搜索关键词", "r": "提高搜索结果相关性"}
    ]
  },
  "label_meta": {
    "case_type": "typical",
    "decision_mode": "compare_more",
    "stage_fit": "search",
    "risk_tags": ["abnormal_low_price", "filtering"],
    "priority": "high"
  }
}
```

## 样本 03

```json
{
  "sample_id": "phone_seed_v04_003",
  "stage": "shortlist",
  "task_goal": "从 8 个 iPhone 候选里先筛掉明显不合适的",
  "context": "候选商品中有几台价格便宜，但描述包含“美版有锁”“换过屏”“商家批量出”等字样。",
  "target": {
    "suggestions": [
      {"s": "帮我先排除这些候选里锁机风险高的商品", "r": "锁机会影响正常使用"},
      {"s": "帮我筛掉不支持验机或描述太含糊的商品", "r": "无法验机会放大风险"},
      {"s": "帮我保留信息最完整、最值得继续问的几台", "r": "透明度高更便于判断"}
    ]
  },
  "label_meta": {
    "case_type": "typical",
    "decision_mode": "pause_or_avoid",
    "stage_fit": "shortlist",
    "risk_tags": ["locked_phone", "screen_replacement", "seller_risk"],
    "priority": "high"
  }
}
```

## 样本 04

```json
{
  "sample_id": "phone_seed_v04_004",
  "stage": "compare",
  "task_goal": "比较三台 iPhone 14 Pro",
  "context": "三台价格分别为 3880、4050、4180 元。最低价电池 82%，中间价电池 88% 但无验机报告，最高价有原盒和保修截图。",
  "target": {
    "suggestions": [
      {"s": "帮我把这三台按电池、保修和验机证据做个对比表", "r": "价格接近时要比证据"},
      {"s": "帮我算一下电池 82% 那台后续可能多花多少钱", "r": "低电池会增加后续支出"},
      {"s": "帮我判断哪台的下单风险最低", "r": "证据影响下单风险"}
    ]
  },
  "label_meta": {
    "case_type": "boundary",
    "decision_mode": "compare_more",
    "stage_fit": "compare",
    "risk_tags": ["battery_health", "warranty_evidence", "comparison"],
    "priority": "high"
  }
}
```

## 样本 05

```json
{
  "sample_id": "phone_seed_v04_005",
  "stage": "item_understanding",
  "task_goal": "确认一台安卓手机是否值得继续沟通",
  "context": "商品描述写着“轻微烧屏不影响使用”，价格比同款低 300 元。",
  "target": {
    "suggestions": [
      {"s": "帮我生成一句话，让卖家补纯色背景的屏幕照片", "r": "纯色图更容易看烧屏"},
      {"s": "帮我判断这个烧屏程度会不会影响日常使用", "r": "位置影响使用体验"},
      {"s": "帮我评估便宜 300 元值不值得接受烧屏风险", "r": "要权衡折价和体验损失"}
    ]
  },
  "label_meta": {
    "case_type": "boundary",
    "decision_mode": "ask_for_evidence",
    "stage_fit": "item_understanding",
    "risk_tags": ["screen_burn_in", "condition_uncertainty"],
    "priority": "high"
  }
}
```

## 样本 06

```json
{
  "sample_id": "phone_seed_v04_006",
  "stage": "seller_chat",
  "task_goal": "向卖家追问 iPhone 维修和锁机风险",
  "context": "用户担心直接问太多显得麻烦，卖家目前只回复“机器没问题”。",
  "target": {
    "suggestions": [
      {"s": "帮我写一句礼貌的话，问卖家有没有拆修或换件", "r": "降低卖家抵触感"},
      {"s": "帮我生成一段话，请卖家录屏展示设置页面", "r": "核对设备真实状态"},
      {"s": "帮我问清楚 Apple ID 和查找功能是否已退出", "r": "排除隐藏锁风险"}
    ]
  },
  "label_meta": {
    "case_type": "ambiguous",
    "decision_mode": "ask_for_evidence",
    "stage_fit": "seller_chat",
    "risk_tags": ["id_lock", "managed_device", "communication"],
    "priority": "high"
  }
}
```

## 样本 07

```json
{
  "sample_id": "phone_seed_v04_007",
  "stage": "risk_check",
  "task_goal": "确认低价 iPhone 是否有隐藏风险",
  "context": "商品比同款低 600 元，卖家说“急出”，但不愿提供序列号截图，也不支持验机。",
  "target": {
    "suggestions": [
      {"s": "帮我判断这台低价 iPhone 现在是不是高风险", "r": "低价叠加拒证很危险"},
      {"s": "帮我生成一句话，追问卖家为什么不能验机", "r": "先确认卖家顾虑"},
      {"s": "帮我找几台更稳妥的同款候选", "r": "风险未清不应推进"}
    ]
  },
  "label_meta": {
    "case_type": "negative_unsafe",
    "decision_mode": "pause_or_avoid",
    "stage_fit": "risk_check",
    "risk_tags": ["abnormal_low_price", "evidence_refusal"],
    "priority": "high"
  }
}
```

## 样本 08

```json
{
  "sample_id": "phone_seed_v04_008",
  "stage": "order_decision",
  "task_goal": "决定是否接受卖家要求脱离平台交易",
  "context": "卖家说平台手续费麻烦，希望用户微信转账并承诺更便宜 100 元。",
  "target": {
    "suggestions": [
      {"s": "帮我判断要不要接受卖家微信转账的要求", "r": "平台外交易缺少保障"},
      {"s": "帮我写一句话，坚持在平台内继续交易", "r": "保留平台维权路径"},
      {"s": "帮我总结这次交易里需要保留的证据", "r": "异常要求需要留证"}
    ]
  },
  "label_meta": {
    "case_type": "negative_unsafe",
    "decision_mode": "pause_or_avoid",
    "stage_fit": "order_decision",
    "risk_tags": ["off_platform_payment", "fraud_risk"],
    "priority": "high"
  }
}
```

## 样本 09

```json
{
  "sample_id": "phone_seed_v04_009",
  "stage": "after_sale",
  "task_goal": "到货后检查二手 iPhone",
  "context": "用户刚收到手机，外观看起来正常，但还没确认功能和卖家承诺是否一致。",
  "target": {
    "suggestions": [
      {"s": "帮我列一个到货后 iPhone 验机清单", "r": "确认是否符合描述"},
      {"s": "帮我设计一套开箱留证步骤", "r": "保留到货原始证据"},
      {"s": "帮我对照卖家承诺逐项检查", "r": "发现不符便于沟通"}
    ]
  },
  "label_meta": {
    "case_type": "typical",
    "decision_mode": "ask_for_evidence",
    "stage_fit": "after_sale",
    "risk_tags": ["arrival_inspection", "evidence_preservation"],
    "priority": "high"
  }
}
```

## 样本 10

```json
{
  "sample_id": "phone_seed_v04_010",
  "stage": "dispute",
  "task_goal": "处理到货后发现屏幕非原装的问题",
  "context": "用户验机发现屏幕可能不是原装，卖家之前承诺无拆修，现在卖家说“能用就行”。",
  "target": {
    "suggestions": [
      {"s": "帮我整理卖家承诺无拆修的聊天证据", "r": "先固定卖家原描述"},
      {"s": "帮我把验机结果整理成维权说明", "r": "用检测证据支撑判断"},
      {"s": "帮我起草一段发给平台客服的话", "r": "让平台介入争议处理"}
    ]
  },
  "label_meta": {
    "case_type": "negative_unsafe",
    "decision_mode": "dispute_or_escalate",
    "stage_fit": "dispute",
    "risk_tags": ["screen_replacement", "evidence_chain", "platform_dispute"],
    "priority": "high"
  }
}
```
