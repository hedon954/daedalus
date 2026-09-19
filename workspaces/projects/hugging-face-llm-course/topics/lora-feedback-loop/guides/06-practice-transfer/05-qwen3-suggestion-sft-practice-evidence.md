# Qwen3 Suggestion Model：真实 SFT 实践证据边界

这份文件只归档用户已经明确确认、且可以脱敏保留的真实工作实践。它不是实验报告；未确认的配置、分数和因果解释必须继续保留为待核验项。

## 已确认事实

- 真实任务：Buyer Agent 的 suggestion list middleware model。
- Base model：Qwen3 1.7B。
- 数据规模：约 27k。
- 训练方式：Full SFT 与 LoRA 均已实际执行。
- 课程状态：用户于 2026-08-27 确认 Hugging Face LLM Course 已全部阅读完毕。

这组事实证明本 topic 的实践已经超过“第一次跑通 LoRA”的入门目标；后续应转向真实实验复盘、可比性审查、评测与反馈闭环，而不是机械重复课程 notebook。

## 需要与历史对话分开的信息

历史对话还出现过另一组 `0.6B`、约 `28k-32k` 数据以及若干评测分数和 `cutoff_len` 排障线索。在用户明确说明两组实验的关系前：

- 不把 `0.6B` 结果归到本次 `1.7B / 27k` 实验。
- 不把历史助手对 Full SFT、LoRA、截断或参数漂移的解释写成已验证因果。
- 博文可以把这些内容列为待重建的实验假设，但不能以“结论已证明”的口吻发表。

## 形成可发布实验报告前仍需补齐

- Base checkpoint 的准确身份与 revision。
- 27k 数据的来源、生成策略、去重方式与 train/validation/test 切分单位。
- Full SFT 与 LoRA 的共同变量、唯一差异和训练配置。
- Chat template、tokenizer、长度分布、截断策略、EOS 与 loss mask 的实际检查结果。
- 两种方法的最佳 checkpoint、评测集、评分口径、误差区间和代表性 failure cases。
- 是否完成 `failure analysis -> feedback data -> retrain -> compare` 第二轮闭环。
- 部署形态：完整 checkpoint、Base + adapter，或 merge 后 checkpoint。

## Transfer Practice

- 来源概念：Hugging Face LLM Course Chapter 3、5、10、11，以及已有 Trainer、tokenizer、task lab 观察。
- 真实任务：用小型 Qwen3 模型替代多阶段 suggestion 生成链路，输出可执行的下一步建议。
- 忠实模仿：chat template、causal LM labels、Trainer/full loop、LoRA adapter、held-out evaluation。
- 简化/丢弃：课程 demo 的通用数据和纯 benchmark 导向；不在档案中保留公司敏感数据、平台凭据或内部地址。
- 重新验证：数据分布、session leakage、模板与截断、Full/LoRA 可比性、业务 eval 和反馈数据收益。
- 最小验证：写出一篇可复核的实验复盘，使第三方能从脱敏配置、数据契约、评测协议和失败样本判断结论是否成立。
