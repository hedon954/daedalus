# SFT 博文系列与掌握度验收计划

## 为什么用博文做 Review

“全部读完”只说明完成了材料覆盖。真正掌握 SFT，至少要能：

1. 脱离教程解释底层机制。
2. 从空白重建最小训练与评测链路。
3. 解释一个真实失败，而不是只报告分数。
4. 改变数据、配置或评测后预测会出现什么信号。
5. 说明结论在哪些条件下失效，不能把单次项目经验写成普遍规律。

每篇文章都必须是可以独立阅读的技术文章，而不是课程摘要。README 只负责导航；正文由用户亲自撰写，Agent 负责提问、challenge、查证和编辑建议。

## 一篇合格技术博文的最低标准

每篇正文至少包含：

- 一个可反驳的中心论点，而不是参数或 API 清单。
- 一张用户自己设计的机制图、数据流图或实验图。
- 一段从输入到输出的最小伪代码、公式或可运行代码。
- 一项真实证据：运行输出、曲线、配置差异、错误样本或消融结果。
- 一个失败解释，并列出至少一个竞争性假设。
- 一个“不能从本文推出什么”的边界段落。
- 可公开的官方资料、论文或源码链接。
- 脱敏检查：不包含公司数据、内部地址、凭据、未公开策略或可逆推出业务机密的信息。

如果文章只能靠引用课程原文成立，删掉引用后就无法解释机制，则不算通过。

## 核心必写系列

### 1. SFT 到底学了什么：从 next-token loss 到行为蒸馏

- 核心论点：SFT 不是记住 JSON，而是通过 teacher-forced next-token cross-entropy 重新塑造 `P(output | system, context)`；对 suggestion model，可进一步理解为把 `conversation state -> next action -> suggestion` 的 teacher policy 蒸馏进参数。
- 课程依赖：Chapter 1、2、3、7、11。
- 必须讲清：chat sequence、causal shift、labels、loss mask、梯度如何改变条件概率；system/context 不稳定为何仍可能泛化。
- 必须挑战：同一 context 有多个合理 output 时，SFT 为什么仍可学习；什么时候 label entropy 会高到无法形成稳定 policy。
- 通过标准：不用 `SFTTrainer` API 名称，也能从一个 suggestion 样本推导训练目标和最小伪代码。
- 暴露薄弱点：如果只能说“让模型学习领域知识”，回到 causal LM、teacher forcing 和 cross-entropy。

### 2. 模型真正吃到的不是 JSON：Chat Template、Tokenizer 与 Dataset Preflight

- 核心论点：SFT 的真实样本是经过 template、tokenization、truncation、label construction 后的 `(input_ids, labels)`，原始 JSON 正确并不能证明训练输入正确。
- 课程依赖：Chapter 2、5、6、8、11。
- 必须讲清：模板边界、special tokens、长度分布、cutoff、truncation side、EOS、padding、loss mask、packing。
- 必须给出：p50/p90/p95/p99/max 统计、至少一个 decode 后训练样本、被截断前后的对照、preflight checklist。
- 通过标准：能判断一次异常训练究竟发生在 raw data、template、tokenizer、collator、labels 还是 trainer 阶段。
- 暴露薄弱点：如果只建议“把 cutoff_len 调大”，但说不清截掉了什么、哪些 token 参与 loss，则不通过。

### 3. Full SFT vs LoRA：优化自由度、成本与泛化边界

- 核心论点：选择 Full 或 LoRA，不只是显存问题，而是任务离 Base 能力多远、数据能否约束参数自由度、以及允许破坏多少已有 representation 的联合决策。
- 课程依赖：Chapter 3、11。
- 真实案例：Qwen3 1.7B、约 27k suggestion 数据、Full SFT 与 LoRA；精确结果补齐后再发表。
- 必须讲清：`W' = W + ΔW`、`ΔW = BA`、rank/alpha/target modules、trainable parameters、不同 learning rate/epoch 纪律、adapter/merge 部署。
- 必须给出：公平实验矩阵，明确相同项、唯一变量、最佳 checkpoint 选择和业务 eval。
- 通过标准：能够提出会推翻“LoRA 更好”或“Full 更强”结论的反例和消融实验。
- 暴露薄弱点：如果文章只是成本对比表或把一次分数差直接归因于 LoRA 正则化，则不通过。

### 4. 27k 不等于 27k 个独立知识点：Suggestion Policy 数据工程

- 核心论点：训练集的价值取决于可学习的 policy、coverage 和 decision boundary，而不是样本条数；语义决策应稳定，表面表达可以多样。
- 课程依赖：Chapter 5、10、11。
- 必须讲清：Plan3 `selector -> action -> writer` 与 Plan0 direct generation 的 bias/variance 取舍；Gold/Silver/Behavioral 数据边界。
- 必须给出：action marginal、action combination/co-occurrence、eligibility/trigger rate、品类/阶段/长度切片、session-level split 和 counterfactual pair。
- 通过标准：能解释为何强行均匀 action distribution 会造成 prior shift，以及如何补低频严格触发 action 而不制造 label noise。
- 暴露薄弱点：如果只统计 action 百分比，却没有检查合法触发场景和组合模板化，则不通过。

### 5. 如何评测一个 Suggestion Model：从 Loss 到业务决策质量

- 核心论点：training loss 只度量训练目标，不能替代 held-out generalization、decision quality 和端到端业务质量。
- 课程依赖：Chapter 3、7、8、11。
- 必须讲清：train/validation/test、conversation-level leakage、eligible cases、model-invoked cases、generation quality、end-to-end success 的分层。
- 必须给出：固定 eval 集、rubric、Judge 与人工校准、failure taxonomy、checkpoint 曲线和误差条或重复实验边界。
- 通过标准：能从一次“分数升高”中排除路由 skip、评测集污染、Judge 漂移和 checkpoint cherry-pick。
- 暴露薄弱点：如果只给总分而没有分母、调用条件和 failure slices，则不通过。

### 6. 从训练到反馈闭环：一次 Qwen3 Suggestion Model 的完整复盘

- 核心论点：真正的 SFT 能力不是启动一次训练，而是形成 `baseline -> train -> eval -> failure analysis -> feedback data -> retrain -> compare` 的可审计闭环。
- 课程依赖：前五篇全部内容，以及 Chapter 4、9 的发布与展示能力。
- 必须给出：任务契约、数据版本、训练配置、Full/LoRA 对照、关键错误、反馈数据构造、第二轮结果、未改善案例与迁移边界。
- 通过标准：第三方读者能区分事实、推断和未知项，并能根据脱敏信息判断实验结论是否成立。
- 暴露薄弱点：如果没有反馈再训练，标题必须诚实改为“训练与评测复盘”，不能声称形成 feedback loop。

## 综合挑战文章

### 7. 我如何把 Hugging Face LLM Course 串成一条模型生命周期

- 主线：`raw data -> dataset -> tokenizer -> model -> training -> evaluation -> Hub/demo -> curation -> SFT/LoRA -> reasoning optimization`。
- 目的：检验是否能把 0–12 章组织成因果依赖，而不是逐章摘要。
- 通过标准：每一章都能说明它解决哪个工程问题、向下依赖什么、向上解锁什么，以及与真实 suggestion model 的关系。

### 8. SFT 之后为什么还需要 GRPO：监督示范与奖励优化的边界

- 核心论点：SFT 提高示范输出的似然，GRPO 等方法用 reward 对模型自己采样的多条输出做相对优化；两者的数据、反馈信号和失败模式不同。
- 课程依赖：Chapter 11、12。
- 必须讲清：demonstration target、on-policy generation、group-relative advantage、reward hacking、KL 约束和 verifiable task 边界。
- 通过标准：能解释 suggestion list 是否适合 GRPO、reward 如何设计、为什么一个可被 Judge 打分的任务仍不必然适合 RL。

## 推荐写作顺序

```text
1 SFT 本质
  -> 2 真实训练样本
  -> 3 Full vs LoRA
  -> 4 数据 Policy
  -> 5 评测
  -> 6 完整实践复盘
  -> 7 课程总览
  -> 8 SFT / GRPO 边界
```

前五篇分别验证机制、数据、优化、数据工程和评测；第六篇才允许整合成项目结论。第七、八篇是综合迁移，不阻塞先完成 SFT 主线。

## 官方课程锚点

- [LLM Course 总览](https://huggingface.co/learn/llm-course/en/chapter1/1)
- [Chapter 5：Datasets](https://huggingface.co/learn/llm-course/en/chapter5/1)
- [Chapter 6：Tokenizers](https://huggingface.co/learn/llm-course/en/chapter6/1)
- [Chapter 7：Classical NLP tasks](https://huggingface.co/learn/llm-course/en/chapter7/1)
- [Chapter 10：Curate high-quality datasets](https://huggingface.co/learn/llm-course/en/chapter10/1)
- [Chapter 11：Fine-tune Large Language Models](https://huggingface.co/learn/llm-course/en/chapter11/6)
- [Chapter 12：Build Reasoning Models / GRPO](https://huggingface.co/learn/llm-course/en/chapter12/4)
