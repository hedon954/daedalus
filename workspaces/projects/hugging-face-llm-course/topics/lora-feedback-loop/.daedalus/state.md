# Topic 学习状态

> 从 [`.daedalus/state.toml`](state.toml) 生成。不要手动编辑。

## 当前状态

- Topic：`lora-feedback-loop` - LoRA 微调与数据反馈闭环
- 生命周期：`parked`
- 当前阶段：`08-review-loop`
- 状态：`paused`
- 下一步：已搁置：用户决定中途搁置，先释放 active 槽以便启动下一题。接回：`daedalus topic activate lora-feedback-loop`。搁置前：继续 `08-review-loop`：先写博文 1《SFT 到底学了什么：从 next-token loss 到行为蒸馏》，闭卷画出 suggestion 样本从 chat template 到 tokens、labels、loss 和参数更新的因果链。

## 枚举约束

- `topic.lifecycle` 只能是：`planned`、`active`、`blocked`、`parked`、`awaiting-reflection`、`completed`、`abandoned`、`skipped`。
- `stage.status` 只能是：`pending`、`active`、`blocked`、`paused`、`done`。
- `transition.action` 只能是：`init`、`enter`、`complete`、`block`、`resume`、`rollback`、`topic-park`、`topic-await-reflection`、`topic-complete`、`topic-abandon`。
- `transition.approval_source` 只能是：`user-confirmed`、`artifact-equivalent`、`stage-not-applicable`。
- Agent 不要发明新的枚举值；如需新增，先修改 Rust 领域模型、模板和测试。

## 阶段进度

- `01-need-aligner`: 对齐课程学习诉求 (done)
- `02-syllabus-mapper`: 映射课程 syllabus (done)
- `03-concept-roadmap`: 建立概念路线图 (done)
- `04-lesson-lab`: 把课程 lesson 变成可观察实验 (done)
- `05-mechanism-deep-dive`: 拆解被 API 隐藏的机制 (pending)
- `06-practice-transfer`: 迁移课程概念到真实任务 (done)
- `07-capstone-lab`: 完成课程驱动 mini project (pending)
- `08-review-loop`: 复习与掌握度验证 (paused)
- `09-closeout-archive`: 专题回顾与知识归档 (pending)

## 缺失产物

- [`guides/05-mechanism-deep-dive/README.md`](../guides/05-mechanism-deep-dive/README.md) 属于 `05-mechanism-deep-dive`
- [`demo/design.md`](../demo/design.md) 属于 `07-capstone-lab`

## 阻塞项

- 无

## 最近状态流转

> 共 13 条状态流转；下面显示最近 10 条，完整历史见 [`.daedalus/state.toml`](state.toml) 的 `[[transitions]]`。

- `2026-06-14 21:32:51` 由 `daedalus-cli` 对 `02-syllabus-mapper` 执行 `complete`：完成材料入口、schema、工程形态、默认模型、训练环境和数据规模收敛。
- `2026-06-14 21:32:51` 由 `daedalus-cli` 对 `06-practice-transfer` 执行 `enter`：进入问题路线图阶段，将默认方案转成数据生成标准、审核问题和 demo 设计前置问题。
- `2026-06-23 00:53:42` 由 `daedalus-cli` 对 `03-concept-roadmap` 执行 `resume`：将 topic 阶段映射为 course-learning 阶段，并迁移 guide/notes 到 course-learning 目录。
- `2026-06-23 01:03:06` 由 `daedalus-cli` 对 `03-concept-roadmap` 执行 `complete`：完成 HF Course 1/2 concept roadmap，并同步 shared concept/progress/syllabus maps。
- `2026-06-23 01:03:06` 由 `daedalus-cli` 对 `04-lesson-lab` 执行 `enter`：进入 course-learning lesson lab，执行 Trainer 拆解 guide 并沉淀用户观察证据。
- `2026-08-27 21:36:21` 由 `daedalus-cli` 对 `04-lesson-lab` 执行 `complete`：用户于 2026-08-27 确认 Hugging Face LLM Course 当前 0–12 章已全部阅读；已有 lesson-lab 产物保留为实验与机制回补入口，未验证的运行证据转入 review 缺口。
- `2026-08-27 21:36:21` 由 `daedalus-cli` 对 `06-practice-transfer` 执行 `enter`：同步用户已完成的真实工作迁移：Qwen3 1.7B、约 27k 数据、Full SFT 与 LoRA 两种训练。
- `2026-08-27 21:36:21` 由 `daedalus-cli` 对 `06-practice-transfer` 执行 `complete`：真实工作迁移事实已由用户确认，并已建立脱敏实践证据边界；精确配置、评测结果和反馈再训练证据留待 review 补齐。
- `2026-08-27 21:36:21` 由 `daedalus-cli` 对 `08-review-loop` 执行 `enter`：课程阅读全部完成，进入以 SFT 博文、掌握度地图和闭卷问题库驱动的复习与掌握度验证。
- `2026-09-19 16:50:39` 由 `daedalus-cli` 对 `topic` 执行 `topic-park`：用户决定中途搁置，先释放 active 槽以便启动下一题。

## 下一步 CLI 建议

- `daedalus state render --topic-dir /Users/hedon/mycode/ai/daedalus/workspaces/projects/hugging-face-llm-course/topics/lora-feedback-loop`
- 完成阶段前先运行 `daedalus validate`。
