# Daedalus 验证日志

记录本学习任务对 daedalus 教学引导能力的反向验证结果。

## 使用规则

- 不要把普通学习笔记写到这里；学习笔记写入 [`notes/`](../notes)。
- 这里只记录 daedalus 本身的引导效果、缺口和改进建议。
- 每完成一个阶段，至少复盘一次“Agent 是否代替用户做太多”。

## 2026-06-14 14:42:45

- 阶段：`01-goal-aligner`
- 有效引导：
- 用户亲自完成的实践：
- Agent 代替用户过多的地方：
- Prompt/template/CLI/docs 改进建议：
- 是否足以进入下一阶段：

## 2026-06-14 初始化补充

- 阶段：`01-goal-aligner`
- 有效引导：根据 topic discovery 对话初始化 `hugging-face-llm-course` / `lora-feedback-loop`，并将已确认的目标、边界和验收标准写入 task card、outcome map、todo 和 goal guide。
- 用户亲自完成的实践：用户确认主线为 Hugging Face / AI Stack 学习并完成 LoRA feedback loop lab；DDIA 作为 ambient reading。
- Agent 代替用户过多的地方：尚未进入 notes 或 demo；本次只整理用户已确认的目标和待 review 的初始化产物。
- Prompt/template/CLI/docs 改进建议：当前 CLI 仍以 `repo-learning` 命名初始化课程/官方文档型项目，后续可考虑 general learning project 类型。
- 是否足以进入下一阶段：等待用户 review task card 后再进入材料选择。

## 2026-06-14 业务任务收敛

- 阶段：`01-goal-aligner`
- 有效引导：用户将 demo 任务收敛为“闲鱼二手买家 Agent 的 suggestion next action 生成”，并确认学习策略为实践 + 理论闭环。
- 用户亲自完成的实践：用户确认最终目标贴近真实岗位，但允许先快速形成闭环，再迁移到真实业务需求程度。
- Agent 代替用户过多的地方：未写用户 notes，当前只同步 task-card / outcome-map / todo / guide。
- Prompt/template/CLI/docs 改进建议：后续 source scout 需要支持课程 + 官方文档 + 示例代码的混合材料，不应强行只按 repo 选择。
- 是否足以进入下一阶段：仍需确认 suggestion next action schema、质量标准、模型/训练环境和材料读取策略。

## 2026-06-14 01-goal-aligner 用户确认

- 阶段：`01-goal-aligner`
- 有效引导：用户确认 fine-tuning 是最后选择，生产上应先做 prompt/context/tooling baseline；第一轮品类选择手机；核心质量标准是推动购买任务往前、多帮用户想一步，并满足安全性、语义完整性和相关性。
- 用户亲自完成的实践：完成 fine-tuning suitability gate 的关键判断。
- Agent 代替用户过多的地方：无；Agent 只同步学习地图。
- Prompt/template/CLI/docs 改进建议：`02-repo-scout` 对课程型项目应更名为 source/material scout。
- 是否足以进入下一阶段：是，可以进入材料选择。

## 2026-06-14 02-source-scout 背景选择

- 阶段：`02-repo-scout`
- 有效引导：用户确认第一阶段先证明完整 pipeline，可接受 Colab/云 GPU，数据由 Agent 生成候选样本后用户审核。
- 用户亲自完成的实践：完成训练环境和数据生成方式的方向选择。
- Agent 代替用户过多的地方：无；Agent 只补充基础知识 primer 和成本判断。
- Prompt/template/CLI/docs 改进建议：后续 guide 应继续降低小白选型门槛，先解释选项含义再要求用户选择。
- 是否足以进入下一阶段：还需确认具体材料清单和第一轮 schema/rubric。
