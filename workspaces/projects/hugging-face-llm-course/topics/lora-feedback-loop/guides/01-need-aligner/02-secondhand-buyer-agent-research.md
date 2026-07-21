# 二手买家 Agent 业务调研与 Fine-tuning Suitability

## Research Question

本 topic 不能预设“闲鱼二手买家 Agent 的 suggestion next action”一定适合 fine-tuning。进入材料选择前，需要先回答：

- 二手购买有哪些真实痛点？
- 传统平台机制解决不了什么？
- AI Agent 可以切入哪些环节？
- fine-tuning 相比 prompt engineering / context engineering 有什么意义？
- 这个任务最终直觉上是好方向，还是硬套训练技术？

## Buyer Pain Points

二手交易的底层矛盾是信息不对称、信任不足和纠纷成本高。买家常见痛点包括：

- 商品真假、成色、功能状态难判断。
- 卖家描述可能模糊、避重就轻或误导。
- 价格是否合理需要比较同款、历史价、成色、配件和维修情况。
- 买家不知道该问什么，容易漏问关键风险问题。
- 二手 C2C 交易的退货和纠纷规则不如传统电商稳定。
- 证据保存、沟通表达和图片凭证会影响纠纷处理结果。

调研线索：

- 英国二手市场报道中，买家常遇到错误商品、空包、假货等骗局；Which? 调查中有相当比例受访买家在二手平台遭遇骗局。
- 国内二手交易风险提示强调，闲鱼等 C2C 交易通常不适用传统电商的七天无理由退货，假货、掉包、证据保存和责任边界都更复杂。
- 闲置交易平台纠纷研究显示，图片凭证、陈词方式、情绪表达和证据数量会影响争议判定。

## Traditional Solutions And Gaps

传统方案主要是平台级机制或静态工具：

| 传统方案 | 能解决什么 | 解决不了什么 |
| --- | --- | --- |
| 信誉系统 | 看评分、历史交易、评价 | 无法判断当前商品是否有暗病，也无法指导买家下一步问什么 |
| 验货 / 鉴定 | 降低真假和成色风险 | 成本高、覆盖品类有限，且通常发生在较晚阶段 |
| 担保交易 | 降低付款风险 | 不解决购买前的信息缺口 |
| 平台规则 / 小法庭 / 客服 | 处理事后纠纷 | 纠纷成本高，且结果依赖证据和表达 |
| 搜索筛选 / 比价 | 帮助找商品和价格参考 | 不理解当前对话阶段和用户下一步动作 |
| 聊天模板 | 给出通用问法 | 不能动态结合商品、风险、对话和买家意图 |

因此，传统方案的问题不是完全无效，而是它们大多不直接帮助买家做动态决策。

## Agent Opportunity

`suggestion next action` 的切入点不是替买家判断一切，而是在每轮回复后给出下一步行动建议：

- 继续追问关键成色细节。
- 要求卖家补充实拍图、视频、序列号或功能测试。
- 查询同款近期价格和历史成交价。
- 询问是否支持当面验货或平台担保。
- 提醒不要绕开平台交易。
- 对高风险商品建议暂缓或换目标。

这类能力适合做成买家决策辅助，而不是商品真假判定器。

## Suitability Judgment

### Fine-tuning 可以解决什么

Fine-tuning / LoRA 可以学习稳定的行为偏好：

- 不同购买阶段该建议什么动作。
- 哪些风险信号应该触发追问或提醒。
- next actions 的粒度、格式、语气和数量。
- 如何避免空泛建议，输出可执行动作。
- 小模型在特定任务上是否能接近大模型 prompt baseline。

如果有足够高质量样本，fine-tuning 可以把大量重复出现的策略偏好压进 adapter，让输出更稳定、更短、更符合产品语气。

### Fine-tuning 不适合解决什么

Fine-tuning 不应该承担实时事实和外部知识：

- 当前商品真实价格是否划算。
- 卖家是否可信。
- 商品是否真货。
- 平台规则是否刚更新。
- 当前同款行情和库存情况。

这些应由 context engineering、retrieval、tools、平台数据和风控系统提供。

## Prompt / Context / Fine-tuning Boundary

合理方案不是三选一，而是分层：

```text
tools / retrieval
  -> 提供实时商品、价格、规则、风险知识

context engineering
  -> 组织商品卡片、卖家信息、对话历史、用户预算、风险标签

prompt engineering
  -> 定义 action taxonomy、输出格式、少量示例和安全边界

fine-tuning / LoRA
  -> 学习稳定 next-action policy、格式、语气和偏好分布
```

生产上应先做 prompt/context baseline。Fine-tuning 只有在 baseline 出现系统性问题时才有意义，例如：

- 输出格式不稳定。
- 动作太泛，缺少场景敏感性。
- 风险提醒漏掉。
- 小模型不听复杂 prompt。
- prompt 太长、成本或延迟不可接受。

## Experiment Plan

为了避免硬套 fine-tuning，本 topic 应做公平比较：

1. 定义固定测试集和评价标准。
2. 做 prompt/context baseline。
3. 做 LoRA fine-tuned model。
4. 比较两者差异：
   - 格式稳定性。
   - 动作可执行性。
   - 购买阶段匹配。
   - 风险意识。
   - 信息增益。
   - 小模型成本和延迟潜力。

如果 LoRA 没有明显改善，也应该如实记录。这仍然是有效学习结论。

## Recommended Scope

不要把目标定成“判断闲鱼商品真假”。范围太大，也需要平台数据和工具支持。

更合适的目标：

```text
买家决策辅助 Agent 的 next-action policy lab
```

第一阶段：

- 30-100 条手写或半合成样本。
- 选择 1-2 个品类，例如手机和相机。
- 输入：用户意图、商品信息、风险线索、对话片段、Agent 当前回复。
- 输出：2-4 个 next actions。
- 比较 prompt baseline 和 LoRA。

第二阶段：

- 扩展品类和风险标签。
- 加入工具结果，例如同款价格、验机清单、平台规则。
- 更接近真实岗位作品，但不拆成第二个 active topic。

## Current Conclusion

作为生产方案：先 prompt/context/tooling，不要一上来 fine-tune。

作为学习 topic：这个方向适合 fine-tuning，因为它有清晰输入输出、可构造数据集、可做 eval、可做错误分析、可反哺训练，能逼迫学习者理解 fine-tuning 的完整工程链路。

因此，本 topic 应保留，但必须把 prompt/context baseline 作为第一对照组。Fine-tuning 的目标不是证明自己一定更好，而是通过实验判断它改善了什么、没有改善什么，以及边界在哪里。

## Sources

- [The Guardian: secondhand marketplace scams and buyer reports](https://www.theguardian.com/money/article/2024/may/09/rife-on-secondhand-marketplaces-depop-preloved-and-shpock)
- [北京市通州区风险提示：二手交易退货、假货和纠纷风险](https://www.bjtzh.gov.cn/xytzh/c109455/202009/1313887.shtml)
- [清华 CJIS PDF：闲置交易平台纠纷与众包判定因素](https://cjis.sem.tsinghua.edu.cn/vol29-03.pdf)
- [Zigpoll: P2P marketplace pain points and trust mechanisms](https://www.zigpoll.com/content/what-are-the-biggest-pain-points-your-users-encounter-when-buying-or-selling-goods-directly-with-each-other-through-your-platform)
- [OpenAI model optimization guide](https://developers.openai.com/api/docs/guides/model-optimization)
- [Hugging Face PEFT documentation](https://huggingface.co/docs/peft/en/index)
