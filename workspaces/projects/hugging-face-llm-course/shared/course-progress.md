# Course Progress

记录课程进度、当前 checkpoint 和复习状态。这里是恢复学习现场的入口，不替代 topic todo。

## Current Checkpoint

- Active topic：`lora-feedback-loop`
- 当前课程章节：Hugging Face LLM Course Chapter 5/2 `What if my dataset isn't on the Hub?`
- 当前 lesson / mechanism：用 `load_dataset(format, data_files=..., field=...)` 从本地或远程文件构造 DatasetDict，并显式映射 train/test split
- 当前 open question：Hub dataset repository ID 与 CSV/text/JSON/pandas 等通用 loader 名称分别在什么场景使用，`data_files` 如何决定 split 与数据来源
- 下一步：阅读并观察 Chapter 5/2，先区分 local path、remote URL、压缩文件与 `data_files` split mapping，再检查加载后的 rows、columns 和嵌套结构

## Progress Board

| Chapter / Section | Status | Evidence | Review |
| --- | --- | --- | --- |
| Chapter 1/5 How Transformers solve tasks | done-with-scope-cut | 5/8 labs complete：Causal LM / sequence classification / token classification / question answering / summarization；Translation / ASR / Image classification 按用户决定跳过或延后 | 足够进入架构归纳 |
| Chapter 1/6 Transformer Architectures | done-enough-to-advance | 架构家族已归纳；attention mechanisms 已完成 Agent-guided walkthrough，LSH/local/axial 可回看 guide | 若后续长上下文机制薄弱再回看 |
| Chapter 1/7 Ungraded quiz | passed-through | 用户已进入 Chapter 1/8 | 不单独归档 quiz 答案 |
| Chapter 1/8 Deep dive into Text Generation Inference with LLMs | done-by-user-confirmation | 已有 inference 主线 guide；用户确认 Chapter 1 已完成并已越过 Chapter 2 | 机制弱点按需回看 |
| Chapter 2 Using Transformers | done-by-user-confirmation | 已有 tokenizer、model、pipeline 与 forward 观察；用户于 2026-07-13 确认完成整章 | 深挖项不再阻塞前进 |
| Chapter 3/1 Introduction | done | 官方页与本章路线已确认；全章 guide 已生成 | 进入 3/2 |
| Chapter 3/2 Processing the data | read-complete | 用户确认已读完；API 漂移已记录，数据处理实验仍待单独验收 | 实验观察按需补齐 |
| Chapter 3/3 Fine-tuning with the Trainer API | read-complete | 用户确认已读完；Trainer 运行产物与指标尚未作为掌握证据验收 | 与 full loop 对照时复查 |
| Chapter 3/4 A full training loop | done-by-user-confirmation | 用户已完成 shuffle、`**batch`、forward/backward/optimizer/scheduler/zero-grad 顺序等机制学习；未虚构完整训练运行结果 | 运行证据按需补齐 |
| Chapter 3/5-3/7 Learning Curves / Check / Quiz | read-complete | 用户确认进入 Chapter 5/2 前的内容均已读完 | 曲线诊断实验仍待验收 |
| Chapter 4 Sharing models and tokenizers | read-complete | 用户确认整章已读完 | Hub 上传与 model card 实践未验收 |
| Chapter 5/1 Introduction | read-complete | 用户确认已读完；Datasets 深入学习路线已建立 | 进入 5/2 |
| Chapter 5/2 What if my dataset isn't on the Hub? | active | 官方页标题与 local/remote loading 主线已核对 | 当前 lesson |

## Review Queue

- Chapter 1/8 inference 与 Chapter 2 的深挖项保留为按需复习，不再作为 Chapter 3 的前置阻塞。
- Chapter 3 的训练/曲线运行证据、Chapter 4 的 Hub 分享实践尚未验收，但不阻塞阅读游标。
- Chapter 5/2：观察 `load_dataset("json", data_files=..., field=...)` 中 loader、source、split 与 nested field 的职责；延续完整 Hub ID 与通用文件 loader 的区分。
