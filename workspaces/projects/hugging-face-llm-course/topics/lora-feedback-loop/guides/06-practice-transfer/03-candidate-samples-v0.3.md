# 第一批候选样本 v0.3

用途：根据用户反馈，将 target schema 收敛为 `{ "suggestions": [{ "s": "建议", "r": "原因" }] }`。

v0.3 规则：

- `suggestions` 最多 3 条。
- `s` 是短建议，可被用户直接点击发送给 Agent。
- `r` 是短原因，解释为什么当前应该做这一步。
- 多条 suggestions 之间要互补。
- 不输出 stage、risk_tags、priority 等内部评测字段。

## 样本 01

```json
{
  "sample_id": "phone_seed_v03_001",
  "stage": "need_clarification",
  "task_goal": "买一台二手 iPhone 作为主力机",
  "context": "用户预算 3500-4500 元，只说想买 iPhone，没有说明用途、容量、续航要求和是否接受维修机。",
  "target": {
    "suggestions": [
      {"s": "补充我的主力机场景", "r": "用途会决定机型优先级"},
      {"s": "帮我确定容量底线", "r": "容量影响长期体验和价格"},
      {"s": "列出不能接受的风险", "r": "先明确维修锁机等红线"}
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
  "sample_id": "phone_seed_v03_002",
  "stage": "search",
  "task_goal": "搜索一台二手 iPhone 14 Pro 256G",
  "context": "用户预算 4000 左右，希望国行、无拆修、电池健康 85% 以上，但搜索结果很多，价格跨度从 3300 到 4600。",
  "target": {
    "suggestions": [
      {"s": "按硬性条件重新筛选", "r": "先减少不匹配商品"},
      {"s": "标记异常低价商品", "r": "低价可能隐藏风险"},
      {"s": "生成搜索关键词", "r": "提高搜索结果相关性"}
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
  "sample_id": "phone_seed_v03_003",
  "stage": "shortlist",
  "task_goal": "从 8 个 iPhone 候选里先筛掉明显不合适的",
  "context": "候选商品中有几台价格便宜，但描述包含“美版有锁”“换过屏”“商家批量出”等字样。",
  "target": {
    "suggestions": [
      {"s": "先排除有锁机", "r": "锁机会影响正常使用"},
      {"s": "筛掉不支持验机的", "r": "无法验机会放大风险"},
      {"s": "保留信息完整商品", "r": "透明度高更便于判断"}
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
  "sample_id": "phone_seed_v03_004",
  "stage": "compare",
  "task_goal": "比较三台 iPhone 14 Pro",
  "context": "三台价格分别为 3880、4050、4180 元。最低价电池 82%，中间价电池 88% 但无验机报告，最高价有原盒和保修截图。",
  "target": {
    "suggestions": [
      {"s": "生成三台对比表", "r": "价格接近时要比证据"},
      {"s": "优先比较电池成本", "r": "低电池会增加后续支出"},
      {"s": "检查保修和验机证据", "r": "证据影响下单风险"}
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
  "sample_id": "phone_seed_v03_005",
  "stage": "item_understanding",
  "task_goal": "确认一台安卓手机是否值得继续沟通",
  "context": "商品描述写着“轻微烧屏不影响使用”，价格比同款低 300 元。",
  "target": {
    "suggestions": [
      {"s": "让卖家拍纯色屏幕", "r": "纯色图更容易看烧屏"},
      {"s": "确认烧屏具体位置", "r": "位置影响日常使用"},
      {"s": "评估低价是否值得", "r": "要权衡折价和体验损失"}
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
  "sample_id": "phone_seed_v03_006",
  "stage": "seller_chat",
  "task_goal": "向卖家追问 iPhone 维修和锁机风险",
  "context": "用户担心直接问太多显得麻烦，卖家目前只回复“机器没问题”。",
  "target": {
    "suggestions": [
      {"s": "生成礼貌追问话术", "r": "降低卖家抵触感"},
      {"s": "要求录屏设置页面", "r": "核对设备真实状态"},
      {"s": "确认已退出 Apple ID", "r": "排除隐藏锁风险"}
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
  "sample_id": "phone_seed_v03_007",
  "stage": "risk_check",
  "task_goal": "确认低价 iPhone 是否有隐藏风险",
  "context": "商品比同款低 600 元，卖家说“急出”，但不愿提供序列号截图，也不支持验机。",
  "target": {
    "suggestions": [
      {"s": "标记为高风险", "r": "低价叠加拒证很危险"},
      {"s": "追问拒绝验机原因", "r": "先确认卖家顾虑"},
      {"s": "暂停继续议价", "r": "风险未清不应推进"}
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
  "sample_id": "phone_seed_v03_008",
  "stage": "order_decision",
  "task_goal": "决定是否接受卖家要求脱离平台交易",
  "context": "卖家说平台手续费麻烦，希望用户微信转账并承诺更便宜 100 元。",
  "target": {
    "suggestions": [
      {"s": "拒绝脱离平台付款", "r": "平台外交易缺少保障"},
      {"s": "要求平台内继续交易", "r": "保留平台维权路径"},
      {"s": "保存卖家转账要求", "r": "异常要求需要留证"}
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
  "sample_id": "phone_seed_v03_009",
  "stage": "after_sale",
  "task_goal": "到货后检查二手 iPhone",
  "context": "用户刚收到手机，外观看起来正常，但还没确认功能和卖家承诺是否一致。",
  "target": {
    "suggestions": [
      {"s": "录制开箱验机视频", "r": "保留到货原始证据"},
      {"s": "逐项检查核心功能", "r": "确认是否符合描述"},
      {"s": "核对卖家原承诺", "r": "发现不符便于沟通"}
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
  "sample_id": "phone_seed_v03_010",
  "stage": "dispute",
  "task_goal": "处理到货后发现屏幕非原装的问题",
  "context": "用户验机发现屏幕可能不是原装，卖家之前承诺无拆修，现在卖家说“能用就行”。",
  "target": {
    "suggestions": [
      {"s": "整理无拆修承诺", "r": "先固定卖家原描述"},
      {"s": "保存验机结果", "r": "用检测证据支撑判断"},
      {"s": "发起平台售后", "r": "让平台介入争议处理"}
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
