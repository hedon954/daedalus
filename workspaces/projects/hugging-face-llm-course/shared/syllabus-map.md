# Syllabus Map

把课程 syllabus 转换为 daedalus 学习路线图。阅读状态与掌握状态必须分开：用户已于 2026-08-27 确认读完当前 Hugging Face LLM Course 0–12 章，但仍需通过博文、闭卷解释和真实实验复盘验收掌握度。

## Course

- URL：https://huggingface.co/learn/llm-course/en
- 官方目录核对日期：2026-08-27。
- 阅读状态：全部阅读完毕（用户确认）。
- Review 策略：以 Chapter 11 的 SFT/LoRA/评测为主线，向前追溯 Transformer、Datasets、Tokenizers、Trainer，向后连接数据策展和 GRPO；不平均复述每章。
- 真实迁移：用户已用 Qwen3 1.7B、约 27k 数据执行 Full SFT 与 LoRA；精确配置、结果和反馈再训练仍需脱敏归档。

## Chapter Map

| Chapter | 课程解决的问题 | 与 SFT 主线的关系 | 阅读状态 | 掌握度验收 |
| --- | --- | --- | --- | --- |
| 0 Setup | 环境、Colab、虚拟环境与依赖 | 保证实验入口可复现 | read-complete-by-user-confirmation | 不单独写文；在实践复盘中交代环境 |
| 1 Transformer Models | pipeline、Transformer、架构、任务与 LLM inference | 提供模型、attention、生成与 transfer learning 直觉 | read-complete-by-user-confirmation | 博文 7；已有多任务 lab |
| 2 Using Transformers | pipeline 内部、model、tokenizer、batch | 解释训练与推理时模型真正接收什么 | read-complete-by-user-confirmation | 博文 2；已有 tokenizer/forward 观察 |
| 3 Fine-tuning a pretrained model | preprocessing、Trainer、full loop、Accelerate、learning curves | 连接 batch 到 loss/backward/optimizer/eval | read-complete-by-user-confirmation | 博文 1、3、5；已有机制 guide 和 notebook |
| 4 Sharing models and tokenizers | Hub、checkpoint、model card、版本化 | 支撑实验资产、部署与复现 | read-complete-by-user-confirmation | 博文 6；实践证据仍需归档 |
| 5 Datasets | 文件/Hub 数据加载、变换、Arrow、流式处理、创建数据集 | 支撑 27k 数据的 schema、split、处理和版本 | read-complete-by-user-confirmation | 博文 2、4；不再停留在 5/2 游标 |
| 6 Tokenizers | tokenizer 训练、fast tokenizer、BPE/WordPiece/Unigram | 决定 template 后文本如何变成训练序列 | read-complete-by-user-confirmation | 博文 2；已有 BPE/WordPiece 笔记 |
| 7 Classical NLP Tasks | token classification、MLM、translation、summarization、CLM、QA | 用不同 task contract 对照 head/loss/postprocess | read-complete-by-user-confirmation | 博文 7；已有五类文本任务实验 |
| 8 How to Ask for Help | 错误定位、训练管线调试、issue 书写 | 把训练失败变成可验证的 diagnosis | read-complete-by-user-confirmation | 博文 2、5、6 的事故复盘质量 |
| 9 Building and Sharing Demos | Gradio、Interface/Blocks、Hub/Spaces | 把模型能力和限制暴露给真实使用者 | read-complete-by-user-confirmation | 博文 6/7；demo 可做但不阻塞 SFT review |
| 10 Curate High-quality Datasets | Argilla、标注、反馈、数据导出 | 把数据质量和人工反馈变成可审计流程 | read-complete-by-user-confirmation | 博文 4；真实数据治理证据待脱敏 |
| 11 Fine-tune Large Language Models | chat template、SFTTrainer、LoRA、evaluation | 本 topic 的核心课程章节 | read-complete-by-user-confirmation | 博文 1–6；Qwen3 Full/LoRA 实践 |
| 12 Build Reasoning Models | RL/GRPO、reward、group-relative optimization | 检验 SFT 与奖励优化的边界 | read-complete-by-user-confirmation | 博文 8；目前只有阅读确认 |

## Stop Rules

- 不把“全部读完”写成“全部掌握”。
- 不按章节平均分配写作篇幅；围绕真实 SFT 问题反向调用课程内容。
- 不把课程 recipe 当作业务最佳实践；数据、成本、安全、评测和部署必须重新验证。
- 不把未经核验的历史对话分数或因果解释并入 Qwen3 1.7B / 27k 实验结论。
