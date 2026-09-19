# SFT 博文闭卷问题库

> 不把选择题正确当成掌握。回答必须包含推导、最小代码/伪代码、真实证据或反例；Agent 不代写答案。

## 博文 1：SFT 本质

1. 一条 `system + context + output` 样本经过 causal LM 训练时，哪些 token 成为输入，哪些 token 成为 labels，loss 在哪里计算？
2. 如果只对 assistant tokens 计算 loss，system/context 为什么仍然能影响梯度？
3. 同一个 context 有三个合理 suggestion 时，单一示范会怎样改变概率分布？什么时候这种监督会产生冲突？
4. 为什么 suggestion SFT 可以称为 behavioral distillation？它与真正蒸馏 logits 有什么不同？

## 博文 2：训练样本与 Preflight

5. 原始 JSON、chat template 后字符串、`input_ids`、`labels` 分别是什么事实层？
6. `cutoff_len` 过小时，system、context、output 谁被截断取决于什么？如何不用猜测直接验证？
7. padding token、EOS、attention mask 和 loss mask 各解决什么问题？混淆后会出现什么信号？
8. 写出训练前必须执行的最小 preflight，并说明每一步能排除哪类 failure。

## 博文 3：Full SFT 与 LoRA

9. 从 `ΔW = BA` 推导 rank 如何影响参数量和表达空间；`alpha` 为什么不是学习率？
10. 为什么 LoRA 可能比 Full 泛化更好？请同时给出一个 Full 更合适的反例。
11. 如何设计 Qwen3 1.7B / 27k 的公平 Full-vs-LoRA 对照？哪些变量必须相同？
12. 如果 LoRA 分数更高，哪些证据才能支持“低秩约束保护了 Base 能力”，而不是其他配置错误？

## 博文 4：数据 Policy

13. 为什么 27k 样本可能只有很低的有效监督多样性？你会用哪些维度证明或反驳？
14. action marginal、action combination 和 action eligibility 各回答什么问题？
15. 为什么把低频 action 强行补到均匀分布可能让线上行为变差？
16. Plan3 相比 Plan0 降低了什么 entropy，又引入了什么 systematic bias？

## 博文 5：评测

17. training loss 下降但业务分数下降，至少有哪些竞争性解释？如何逐一验证？
18. 为什么必须按 conversation/trajectory 切分，而不是随机按样本切分？
19. eligible cases、model-invoked cases、generation pass rate 和 end-to-end success rate 应如何分层？
20. Judge 分数上升时，如何排除 prompt 漂移、样本污染、路由 skip 和 checkpoint cherry-pick？

## 博文 6：反馈闭环

21. 一个 failure case 什么时候应该改 prompt、改 teacher policy、补数据、改 eval，什么时候才应该重新训练？
22. feedback data 如何避免只是重复拟合旧测试集？
23. 第二轮训练必须保留哪些版本和证据，才能把改善归因到新增数据？
24. 哪些失败没有改善时，应该停止补数据并怀疑 base capacity、任务定义或评测体系？

## 博文 7–8：课程综合与方法边界

25. 把 Hugging Face Course 0–12 章压缩成一条模型生命周期，每章的输入、输出和依赖是什么？
26. SFT 与 GRPO 分别使用什么监督信号、在什么分布上采样、优化什么目标？
27. suggestion list 有 Judge 就一定适合 GRPO 吗？reward hacking、不可验证偏好和线上安全如何改变答案？
28. 哪些业务规则应该进入模型权重，哪些应该留在 prompt/runtime？请给出会随时间变化的反例。
