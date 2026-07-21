# Topic 学习状态

> 从 [`.daedalus/state.toml`](state.toml) 生成。不要手动编辑。

## 当前状态

- Topic：`lora-feedback-loop` - LoRA 微调与数据反馈闭环
- 生命周期：`active`
- 当前阶段：`04-lesson-lab`
- 状态：`active`
- 下一步：继续 `04-lesson-lab`：只推进 Chapter 5/2 `What if my dataset isn't on the Hub?`，先区分通用文件 loader、local/remote source、`data_files` split mapping 与 JSON `field`，并检查加载后的 DatasetDict 结构。

## 枚举约束

- `topic.lifecycle` 只能是：`planned`、`active`、`blocked`、`awaiting-reflection`、`completed`、`abandoned`、`skipped`。
- `stage.status` 只能是：`pending`、`active`、`blocked`、`paused`、`done`。
- `transition.action` 只能是：`init`、`enter`、`complete`、`block`、`resume`、`rollback`、`topic-await-reflection`、`topic-complete`、`topic-abandon`。
- `transition.approval_source` 只能是：`user-confirmed`、`artifact-equivalent`、`stage-not-applicable`。
- Agent 不要发明新的枚举值；如需新增，先修改 Rust 领域模型、模板和测试。

## 阶段进度

- `01-need-aligner`: 对齐课程学习诉求 (done)
- `02-syllabus-mapper`: 映射课程 syllabus (done)
- `03-concept-roadmap`: 建立概念路线图 (done)
- `04-lesson-lab`: 把课程 lesson 变成可观察实验 (active)
- `05-mechanism-deep-dive`: 拆解被 API 隐藏的机制 (pending)
- `06-practice-transfer`: 迁移课程概念到真实任务 (pending)
- `07-capstone-lab`: 完成课程驱动 mini project (pending)
- `08-review-loop`: 复习与掌握度验证 (pending)
- `09-closeout-archive`: 专题回顾与知识归档 (pending)

## 缺失产物

- [`guides/05-mechanism-deep-dive/README.md`](../guides/05-mechanism-deep-dive/README.md) 属于 `05-mechanism-deep-dive`
- [`demo/design.md`](../demo/design.md) 属于 `07-capstone-lab`
- [`review/mastery-map.md`](../review/mastery-map.md) 属于 `08-review-loop`
- [`review/question-bank.md`](../review/question-bank.md) 属于 `08-review-loop`

## 阻塞项

- 无

## 最近状态流转

> 共 8 条状态流转；下面显示最近 8 条，完整历史见 [`.daedalus/state.toml`](state.toml) 的 `[[transitions]]`。

- `2026-06-14 14:42:45` 由 `daedalus-cli` 对 `01-need-aligner` 执行 `init`：初始化 course learning topic。
- `2026-06-14 20:44:11` 由 `daedalus-cli` 对 `01-need-aligner` 执行 `complete`：用户确认 fine-tuning suitability、第一轮手机品类和 suggestion next action 质量标准，01 目标对齐完成。
- `2026-06-14 20:44:11` 由 `daedalus-cli` 对 `02-syllabus-mapper` 执行 `enter`：进入 Hugging Face LLM Course source/material scout，选择支持 baseline + LoRA 对照实验的最小材料集合。
- `2026-06-14 21:32:51` 由 `daedalus-cli` 对 `02-syllabus-mapper` 执行 `complete`：完成材料入口、schema、工程形态、默认模型、训练环境和数据规模收敛。
- `2026-06-14 21:32:51` 由 `daedalus-cli` 对 `06-practice-transfer` 执行 `enter`：进入问题路线图阶段，将默认方案转成数据生成标准、审核问题和 demo 设计前置问题。
- `2026-06-23 00:53:42` 由 `daedalus-cli` 对 `03-concept-roadmap` 执行 `resume`：将 topic 阶段映射为 course-learning 阶段，并迁移 guide/notes 到 course-learning 目录。
- `2026-06-23 01:03:06` 由 `daedalus-cli` 对 `03-concept-roadmap` 执行 `complete`：完成 HF Course 1/2 concept roadmap，并同步 shared concept/progress/syllabus maps。
- `2026-06-23 01:03:06` 由 `daedalus-cli` 对 `04-lesson-lab` 执行 `enter`：进入 course-learning lesson lab，执行 Trainer 拆解 guide 并沉淀用户观察证据。

## 下一步 CLI 建议

- `daedalus state render --topic-dir /Users/hedon/mycode/ai/daedalus/workspaces/projects/hugging-face-llm-course/topics/lora-feedback-loop`
- 完成阶段前先运行 `daedalus validate`。
