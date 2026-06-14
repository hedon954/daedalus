# 第一批候选样本 v0.1

用途：供用户校准 `keep / revise / weak / reject` 审核口径。当前样本还不是训练集，审核后才进入正式 dataset。

审核时优先看：

- suggestion 是否具体、可执行。
- reason 是否解释了阶段、风险或用户卡点。
- 是否真的帮助用户往“买到自己要的商品”前进一步。
- 是否存在事实错误、安全风险、太泛或脱离阶段的问题。

补充审核维度：

- 样本是 typical、ambiguous、boundary，还是 negative / unsafe？
- 这个 case 里应该继续推进、补证据、继续比较、议价、暂停避开，还是进入维权？
- 如果建议太武断，例如信息不足却直接推荐购买，也应标为 `weak` 或 `reject`。

后续进入正式 dataset 时，`label_meta` 会补充：

```json
{
  "case_type": "ambiguous",
  "decision_mode": "ask_for_evidence"
}
```

## 样本 01

```json
{
  "sample_id": "phone_seed_001",
  "stage": "need_clarification",
  "task_goal": "买一台二手 iPhone 作为主力机",
  "context": "用户预算大约 3500-4500 元，只说想买 iPhone，但没有说明用途、容量、续航要求和是否接受维修机。",
  "target": {
    "suggestion": "先确认你更看重拍照、续航、容量还是保修，再把预算拆成 2-3 个可接受机型范围。",
    "reason": "当前需求还不清晰，直接看商品容易被单个低价吸引；先明确用途和硬性约束，才能避免后面对比时标准摇摆。"
  },
  "label_meta": {
    "stage_fit": "need_clarification",
    "risk_tags": ["unclear_requirements", "budget_scope"],
    "priority": "high"
  }
}
```

## 样本 02

```json
{
  "sample_id": "phone_seed_002",
  "stage": "need_clarification",
  "task_goal": "给父母买一台二手安卓手机",
  "context": "用户只说要便宜耐用，候选商品还没开始找，不确定是否需要大字体、长续航、双卡和售后方便。",
  "target": {
    "suggestion": "先列出父母每天最常用的 3 个场景，比如微信视频、拍照、刷短视频或导航，再确定电池、屏幕和存储底线。",
    "reason": "给父母买手机时，参数强弱不是唯一标准；先确认真实使用场景，能避免买到便宜但续航、屏幕或存储不合适的机型。"
  },
  "label_meta": {
    "stage_fit": "need_clarification",
    "risk_tags": ["usage_mismatch", "battery_life"],
    "priority": "medium"
  }
}
```

## 样本 03

```json
{
  "sample_id": "phone_seed_003",
  "stage": "search",
  "task_goal": "搜索一台二手 iPhone 14 Pro 256G",
  "context": "用户已经确定预算 4000 左右，希望国行、无拆修、电池健康 85% 以上，但搜索结果很多，价格跨度从 3300 到 4600。",
  "target": {
    "suggestion": "先用“国行 256G 无拆修 电池85以上 当面验机”作为搜索条件，并把明显低于市场价太多的结果单独标记为高风险。",
    "reason": "当前处于搜索阶段，先用硬性条件过滤能减少噪音；异常低价常伴随维修、锁机、监管机或描述不完整风险。"
  },
  "label_meta": {
    "stage_fit": "search",
    "risk_tags": ["abnormal_low_price", "filtering"],
    "priority": "high"
  }
}
```

## 样本 04

```json
{
  "sample_id": "phone_seed_004",
  "stage": "search",
  "task_goal": "找一台二手小米或红米备用机",
  "context": "用户预算 800 元以内，想要续航好，但搜索时只按价格从低到高排序。",
  "target": {
    "suggestion": "把搜索条件改成“大电池、内存 8G 起、存储 128G 起、支持线下验机”，再筛掉描述里没有电池和成色信息的商品。",
    "reason": "备用机最容易被低价吸引，但续航和存储才决定实际体验；缺少关键信息的低价商品会增加后续沟通和踩坑成本。"
  },
  "label_meta": {
    "stage_fit": "search",
    "risk_tags": ["low_price_bias", "missing_condition_info"],
    "priority": "medium"
  }
}
```

## 样本 05

```json
{
  "sample_id": "phone_seed_005",
  "stage": "shortlist",
  "task_goal": "从 8 个 iPhone 候选里先筛掉明显不合适的",
  "context": "候选商品中有几台价格便宜，但描述包含“美版有锁”“换过屏”“商家批量出”等字样。",
  "target": {
    "suggestion": "先把“有锁机、换屏未说明来源、商家批量出且不支持当面验机”的商品移出候选列表。",
    "reason": "初筛阶段应先排除高风险选项，而不是在所有商品上平均花时间；锁机、屏幕更换和无法验机会显著提高交易风险。"
  },
  "label_meta": {
    "stage_fit": "shortlist",
    "risk_tags": ["locked_phone", "screen_replacement", "seller_risk"],
    "priority": "high"
  }
}
```

## 样本 06

```json
{
  "sample_id": "phone_seed_006",
  "stage": "shortlist",
  "task_goal": "筛选二手安卓旗舰",
  "context": "用户看到几台骁龙旗舰，价格接近，但有的只写“功能正常”，没有任何成色、维修或电池说明。",
  "target": {
    "suggestion": "优先保留描述里明确写出成色、维修史、电池状态和支持验机的商品，把只写“功能正常”的商品降级处理。",
    "reason": "初筛不是选最便宜，而是选信息透明度更高的候选；信息越少，后续隐藏维修或成色争议的概率越高。"
  },
  "label_meta": {
    "stage_fit": "shortlist",
    "risk_tags": ["information_asymmetry", "repair_history"],
    "priority": "medium"
  }
}
```

## 样本 07

```json
{
  "sample_id": "phone_seed_007",
  "stage": "compare",
  "task_goal": "比较三台 iPhone 14 Pro",
  "context": "三台价格分别为 3880、4050、4180 元。最低价电池 82%，中间价电池 88% 但无验机报告，最高价有原盒和保修截图。",
  "target": {
    "suggestion": "先不要只看价格，把三台按电池健康、保修证据、验机支持和维修记录做一张对比表。",
    "reason": "当前价格差异不大，真正影响决策的是风险证据和后续使用成本；电池偏低或缺少验机证据可能抵消低价优势。"
  },
  "label_meta": {
    "stage_fit": "compare",
    "risk_tags": ["battery_health", "warranty_evidence", "comparison"],
    "priority": "high"
  }
}
```

## 样本 08

```json
{
  "sample_id": "phone_seed_008",
  "stage": "compare",
  "task_goal": "在 iPhone 13 Pro 和 iPhone 14 之间选择",
  "context": "用户纠结性能、拍照和价格，候选商品价格接近，但没有考虑屏幕、重量、续航和维修成本。",
  "target": {
    "suggestion": "把选择标准从“哪台更新”改成“高刷屏、长焦拍照、续航、维修成本”四项，按你的真实使用频率打分。",
    "reason": "当前是跨机型对比，单看年份容易误判；用使用场景打分能让建议从参数比较推进到个人决策。"
  },
  "label_meta": {
    "stage_fit": "compare",
    "risk_tags": ["model_tradeoff", "usage_fit"],
    "priority": "medium"
  }
}
```

## 样本 09

```json
{
  "sample_id": "phone_seed_009",
  "stage": "item_understanding",
  "task_goal": "了解一台标称无拆修的 iPhone",
  "context": "卖家展示了外观图和电池健康截图，但没有主板、屏幕、摄像头和 Face ID 状态说明。",
  "target": {
    "suggestion": "让卖家补充关于屏幕、Face ID、摄像头、扬声器和是否拆修的功能检查视频。",
    "reason": "当前商品信息还停留在外观和电池，关键功能缺少证据；这些项目一旦有问题，维修成本和使用体验都会明显受影响。"
  },
  "label_meta": {
    "stage_fit": "item_understanding",
    "risk_tags": ["function_check", "repair_history"],
    "priority": "high"
  }
}
```

## 样本 10

```json
{
  "sample_id": "phone_seed_010",
  "stage": "item_understanding",
  "task_goal": "确认一台安卓手机是否值得继续沟通",
  "context": "商品描述写着“轻微烧屏不影响使用”，价格比同款低 300 元。",
  "target": {
    "suggestion": "让卖家在纯白、纯灰和深色背景下各拍一张屏幕照片，再确认烧屏位置和日常使用是否明显。",
    "reason": "烧屏问题仅靠一句“不影响使用”无法判断；不同背景照片能帮助评估真实程度，决定低价是否值得接受。"
  },
  "label_meta": {
    "stage_fit": "item_understanding",
    "risk_tags": ["screen_burn_in", "condition_uncertainty"],
    "priority": "high"
  }
}
```

## 样本 11

```json
{
  "sample_id": "phone_seed_011",
  "stage": "seller_chat",
  "task_goal": "向卖家追问 iPhone 维修和锁机风险",
  "context": "用户担心直接问太多显得麻烦，卖家目前只回复“机器没问题”。",
  "target": {
    "suggestion": "可以礼貌地问：“方便录个设置页、关于本机、Apple ID 已退出和查找已关闭的视频吗？我主要想确认不是监管机或隐藏锁。”",
    "reason": "当前进入卖家沟通阶段，具体而礼貌的问法能降低对方抵触，同时直接索取排除锁机风险的关键证据。"
  },
  "label_meta": {
    "stage_fit": "seller_chat",
    "risk_tags": ["id_lock", "managed_device", "communication"],
    "priority": "high"
  }
}
```

## 样本 12

```json
{
  "sample_id": "phone_seed_012",
  "stage": "seller_chat",
  "task_goal": "和卖家沟通线下验机",
  "context": "卖家同城，支持面交，但用户不知道如何提出验机要求。",
  "target": {
    "suggestion": "先问卖家是否接受在公开地点面交，并预留 15 分钟现场检查屏幕、摄像头、Face ID、充电和网络功能。",
    "reason": "线下交易的关键不是只见面付款，而是提前约定检查项目和时间；这样能减少现场仓促付款后的争议。"
  },
  "label_meta": {
    "stage_fit": "seller_chat",
    "risk_tags": ["offline_inspection", "transaction_safety"],
    "priority": "high"
  }
}
```

## 样本 13

```json
{
  "sample_id": "phone_seed_013",
  "stage": "risk_check",
  "task_goal": "确认低价 iPhone 是否有隐藏风险",
  "context": "商品比同款低 600 元，卖家说“急出”，但不愿提供序列号截图，也不支持验机。",
  "target": {
    "suggestion": "把这台先标为高风险，不要继续议价；优先追问为什么不能提供序列号截图和验机证据。",
    "reason": "异常低价叠加拒绝提供证据，是二手交易中的强风险信号；此时继续压价不如先确认是否存在锁机、维修或来源问题。"
  },
  "label_meta": {
    "stage_fit": "risk_check",
    "risk_tags": ["abnormal_low_price", "evidence_refusal"],
    "priority": "high"
  }
}
```

## 样本 14

```json
{
  "sample_id": "phone_seed_014",
  "stage": "risk_check",
  "task_goal": "判断一台“官换机”描述是否可信",
  "context": "卖家说手机是官换机，价格合理，但没有提供保修记录或官方换机凭证。",
  "target": {
    "suggestion": "让卖家提供可核验的保修记录、换机凭证或序列号查询截图，不要只接受“官换机”口头描述。",
    "reason": "官换机会影响价值判断，但必须有证据支撑；没有凭证时，它可能只是卖家解释维修或来源不清的说法。"
  },
  "label_meta": {
    "stage_fit": "risk_check",
    "risk_tags": ["replacement_device", "evidence_required"],
    "priority": "medium"
  }
}
```

## 样本 15

```json
{
  "sample_id": "phone_seed_015",
  "stage": "order_decision",
  "task_goal": "准备下单一台已沟通过的二手手机",
  "context": "卖家已补充照片和视频，价格谈妥，用户准备直接付款。",
  "target": {
    "suggestion": "下单前把关键承诺写进聊天记录，包括无拆修、无锁、功能正常、支持到货当天验机和问题处理方式。",
    "reason": "下单决策阶段需要把口头承诺转成平台内可追溯记录；这能降低到货争议时举证困难。"
  },
  "label_meta": {
    "stage_fit": "order_decision",
    "risk_tags": ["evidence_preservation", "platform_record"],
    "priority": "high"
  }
}
```

## 样本 16

```json
{
  "sample_id": "phone_seed_016",
  "stage": "order_decision",
  "task_goal": "决定是否接受卖家要求脱离平台交易",
  "context": "卖家说平台手续费麻烦，希望用户微信转账并承诺更便宜 100 元。",
  "target": {
    "suggestion": "不要脱离平台付款，继续要求在平台内交易，并保留所有关于商品状态和售后承诺的聊天记录。",
    "reason": "脱离平台会削弱交易保护和维权证据；100 元优惠无法覆盖被骗、货不对板或售后无门的风险。"
  },
  "label_meta": {
    "stage_fit": "order_decision",
    "risk_tags": ["off_platform_payment", "fraud_risk"],
    "priority": "high"
  }
}
```

## 样本 17

```json
{
  "sample_id": "phone_seed_017",
  "stage": "after_sale",
  "task_goal": "到货后检查二手 iPhone",
  "context": "用户刚收到手机，外观看起来正常，但还没确认功能和卖家承诺是否一致。",
  "target": {
    "suggestion": "先录制开箱和功能检查视频，依次检查屏幕、摄像头、Face ID、扬声器、充电、网络、电池健康和 Apple ID 状态。",
    "reason": "到货初期是发现问题和留证的关键窗口；完整视频能帮助确认商品是否符合承诺，也便于后续沟通或维权。"
  },
  "label_meta": {
    "stage_fit": "after_sale",
    "risk_tags": ["arrival_inspection", "evidence_preservation"],
    "priority": "high"
  }
}
```

## 样本 18

```json
{
  "sample_id": "phone_seed_018",
  "stage": "after_sale",
  "task_goal": "收到安卓手机后发现续航很差",
  "context": "卖家之前说电池正常，但用户半天就掉电很多，不确定是不是正常老化。",
  "target": {
    "suggestion": "先记录电池使用截图、亮屏时间和异常掉电过程，再和卖家对照他之前关于电池正常的描述沟通。",
    "reason": "续航问题容易变成主观争议；用系统记录和聊天承诺对齐，能让沟通从感受变成证据。"
  },
  "label_meta": {
    "stage_fit": "after_sale",
    "risk_tags": ["battery_issue", "evidence_preservation"],
    "priority": "medium"
  }
}
```

## 样本 19

```json
{
  "sample_id": "phone_seed_019",
  "stage": "dispute",
  "task_goal": "处理到货后发现屏幕非原装的问题",
  "context": "用户验机发现屏幕可能不是原装，卖家之前承诺无拆修，现在卖家说“能用就行”。",
  "target": {
    "suggestion": "先整理卖家承诺无拆修的聊天记录、验机结果、屏幕异常证据，再通过平台发起沟通或售后流程。",
    "reason": "维权阶段不要陷入口头争吵，关键是把承诺、检测结果和商品不一致点整理成平台可判断的证据链。"
  },
  "label_meta": {
    "stage_fit": "dispute",
    "risk_tags": ["screen_replacement", "evidence_chain", "platform_dispute"],
    "priority": "high"
  }
}
```

## 样本 20

```json
{
  "sample_id": "phone_seed_020",
  "stage": "dispute",
  "task_goal": "处理卖家拒绝承认隐藏 ID 锁风险",
  "context": "用户收到手机后发现无法正常退出某些账号状态，卖家之前没有说明相关限制。",
  "target": {
    "suggestion": "立即停止继续使用和重置操作，先录屏保留账号状态异常、设置页面和卖家原始描述，再联系平台说明商品与描述不符。",
    "reason": "账号锁或限制状态可能影响手机所有权和正常使用；维权时需要保留原始状态证据，避免操作后证据消失。"
  },
  "label_meta": {
    "stage_fit": "dispute",
    "risk_tags": ["id_lock", "evidence_preservation", "platform_dispute"],
    "priority": "high"
  }
}
```

## 用户审核模板

请按下面格式给反馈即可：

```text
001 keep
002 revise：原因...
003 weak：原因...
004 reject：原因...
```

你也可以只先审 5-10 条；校准口径比一次审完更重要。
