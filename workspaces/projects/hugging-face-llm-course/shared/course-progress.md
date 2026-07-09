# Course Progress

记录课程进度、当前 checkpoint 和复习状态。这里是恢复学习现场的入口，不替代 topic todo。

## Current Checkpoint

- Active topic：`lora-feedback-loop`
- 当前课程章节：Hugging Face LLM Course Chapter 1/8 `Deep dive into Text Generation Inference with LLMs`
- 当前 lesson / mechanism：LLM inference 如何从 prompt 到逐 token 生成，尤其是 prefill / decode、sampling controls、KV cache 和推理性能指标
- 当前 open question：LLM 推理时哪些成本来自读 prompt，哪些成本来自逐 token 生成，哪些参数控制生成行为
- 下一步：阅读 Chapter 1/8，先建立 inference 主线：attention/context -> prompting -> prefill/decode -> sampling -> performance/KV cache

## Progress Board

| Chapter / Section | Status | Evidence | Review |
| --- | --- | --- | --- |
| Chapter 1/5 How Transformers solve tasks | done-with-scope-cut | 5/8 labs complete：Causal LM / sequence classification / token classification / question answering / summarization；Translation / ASR / Image classification 按用户决定跳过或延后 | 足够进入架构归纳 |
| Chapter 1/6 Transformer Architectures | done-enough-to-advance | 架构家族已归纳；attention mechanisms 已完成 Agent-guided walkthrough，LSH/local/axial 可回看 guide | 若后续长上下文机制薄弱再回看 |
| Chapter 1/7 Ungraded quiz | passed-through | 用户已进入 Chapter 1/8 | 不单独归档 quiz 答案 |
| Chapter 1/8 Deep dive into Text Generation Inference with LLMs | active | 官方页标题已确认；当前准备读 inference 主线 | 当前读 LLM inference |
| Chapter 2 Using Transformers | planned | tokenizer / model / Trainer observations | Chapter 1 后续小节后进入 |

## Review Queue

- Chapter 1/8 LLM inference：理解 attention/context、prompting、prefill/decode、sampling controls、TTFT/TPOT/throughput/VRAM 和 KV cache。
- 后续提醒：Chapter 2 前后手写一次 `pipeline("text-classification")` 等价流程；进入 Trainer 复盘时补 batch keys / shape / device 和 `model(**batch)` 的 loss/logits 观察。
