# Daedalus 验证日志

记录本学习任务对 daedalus 教学引导能力的反向验证结果。

## 使用规则

- 不要把普通学习笔记写到这里；学习笔记写入 [`notes/`](../notes)。
- 这里只记录 daedalus 本身的引导效果、缺口和改进建议。
- 每完成一个阶段，至少复盘一次“Agent 是否代替用户做太多”。

## 2026-06-14 14:42:45

- 阶段：`01-need-aligner`
- 有效引导：
- 用户亲自完成的实践：
- Agent 代替用户过多的地方：
- Prompt/template/CLI/docs 改进建议：
- 是否足以进入下一阶段：

## 2026-06-14 初始化补充

- 阶段：`01-need-aligner`
- 有效引导：根据 topic discovery 对话初始化 `hugging-face-llm-course` / `lora-feedback-loop`，并将已确认的目标、边界和验收标准写入 task card、outcome map、todo 和 goal guide。
- 用户亲自完成的实践：用户确认主线为 Hugging Face / AI Stack 学习并完成 LoRA feedback loop lab；DDIA 作为 ambient reading。
- Agent 代替用户过多的地方：尚未进入 notes 或 demo；本次只整理用户已确认的目标和待 review 的初始化产物。
- Prompt/template/CLI/docs 改进建议：课程/官方文档型项目应使用 course-learning 项目类型，避免混用仓库学习阶段语义。
- 是否足以进入下一阶段：等待用户 review task card 后再进入材料选择。

## 2026-06-14 业务任务收敛

- 阶段：`01-need-aligner`
- 有效引导：用户将 demo 任务收敛为“闲鱼二手买家 Agent 的 suggestion next action 生成”，并确认学习策略为实践 + 理论闭环。
- 用户亲自完成的实践：用户确认最终目标贴近真实岗位，但允许先快速形成闭环，再迁移到真实业务需求程度。
- Agent 代替用户过多的地方：未写用户 notes，当前只同步 task-card / outcome-map / todo / guide。
- Prompt/template/CLI/docs 改进建议：后续 syllabus mapper 需要支持课程 + 官方文档 + 示例代码的混合材料，不应强行只按仓库选择。
- 是否足以进入下一阶段：仍需确认 suggestion next action schema、质量标准、模型/训练环境和材料读取策略。

## 2026-06-14 01-need-aligner 用户确认

- 阶段：`01-need-aligner`
- 有效引导：用户确认 fine-tuning 是最后选择，生产上应先做 prompt/context/tooling baseline；第一轮品类选择手机；核心质量标准是推动购买任务往前、多帮用户想一步，并满足安全性、语义完整性和相关性。
- 用户亲自完成的实践：完成 fine-tuning suitability gate 的关键判断。
- Agent 代替用户过多的地方：无；Agent 只同步学习地图。
- Prompt/template/CLI/docs 改进建议：课程型项目应使用 `02-syllabus-mapper` 来承接 source/material 选择。
- 是否足以进入下一阶段：是，可以进入材料选择。

## 2026-06-14 02-syllabus-mapper 背景选择

- 阶段：`02-syllabus-mapper`
- 有效引导：用户确认第一阶段先证明完整 pipeline，可接受 Colab/云 GPU，数据由 Agent 生成候选样本后用户审核。
- 用户亲自完成的实践：完成训练环境和数据生成方式的方向选择。
- Agent 代替用户过多的地方：无；Agent 只补充基础知识 primer 和成本判断。
- Prompt/template/CLI/docs 改进建议：后续 guide 应继续降低小白选型门槛，先解释选项含义再要求用户选择。
- 是否足以进入下一阶段：还需确认具体材料清单和第一轮 schema/rubric。

## 2026-06-15 suggestion 产品形态修正

- 阶段：`06-practice-transfer`
- 有效引导：用户 review 第一批候选样本后指出 v0.1 的 suggestion 太长，无法直接点击发送给 Agent；Agent 将默认模型输出 schema 修正为最多 3 条可点击短句数组，并生成 v0.2 样本。
- 用户亲自完成的实践：完成关键产品形态判断：suggestion next action 不是解释型建议，而是用户可点击的下一步指令。
- Agent 代替用户过多的地方：v0.1 样本生成时过度偏解释型 coach 文案，没有先贴近产品交互形态。
- Prompt/template/CLI/docs 改进建议：后续样本生成 prompt 应显式区分“模型输出字段”和“标注/eval 字段”，要求 suggestion 是短句、可点击、最多 3 条且互补；标注侧必须保留逐条 reason，模型输出 reason 则作为小规模 ablation。
- 是否足以进入下一阶段：等待用户审核 v0.2 的 5-10 条样本后再扩展到 120 条。

## 2026-06-15 target schema 收敛

- 阶段：`06-practice-transfer`
- 有效引导：用户进一步确认 target 应直接采用 `{ "suggestions": [{ "s": "建议", "r": "原因" }] }`，而不是 action-only 或 action/reason 长字段。
- 用户亲自完成的实践：完成产品输出 schema 的关键收敛：`s` 保持可点击短建议，`r` 保持短原因，最多 3 条且互补。
- Agent 代替用户过多的地方：Agent 曾将 reason 拆成标注侧或 ablation，导致 schema 分叉；用户纠正后已收敛到单一 target。
- Prompt/template/CLI/docs 改进建议：后续样本生成 prompt 应直接以 `{s,r}` 为目标 schema，避免再次发散到 action-only / action-with-reason。
- 是否足以进入下一阶段：等待用户审核 v0.3 的 5-10 条样本后再扩展到 120 条。

## 2026-06-15 suggestion 说话对象修正

- 阶段：`06-practice-transfer`
- 有效引导：用户指出 v0.3 的 `s` 仍像菜单标签，而不是用户点击后直接发送给买家 Agent 的自然指令；Agent 将 `s` 的定义修正为“用户口吻发给买家 Agent 的下一步请求”，并生成 v0.4 样本。
- 用户亲自完成的实践：完成交互语义判断：suggestion next action 不是给用户阅读的建议，而是降低用户发号施令摩擦的可点击用户消息。
- Agent 代替用户过多的地方：Agent 之前只关注“短”和“可点击”，忽略点击后的消息接收者和说话口吻。
- Prompt/template/CLI/docs 改进建议：后续样本生成必须先写清 `s` 的 recipient 是买家 Agent，speaker 是用户；禁止生成内部动作标签式文案。
- 是否足以进入下一阶段：等待用户审核 v0.4 的 5-10 条样本后再扩展到 120 条。
