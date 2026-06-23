# Course Learning 一等学习类型实现方案

> 日期：2026-06-23
> 状态：已完成

## 背景

`hugging-face-llm-course` 这类学习材料有明确的课程主页、章节顺序、练习代码、概念依赖和官方文档。daedalus 需要把它们建模成独立的 course-learning project/topic，而不是只靠通用学习状态机承载。

course learning 的核心材料是一套有顺序、有教学目标、有练习、有概念依赖的课程。它的学习路径围绕：

```text
学习诉求 -> syllabus 路线 -> 概念机制 -> 最小实验 -> 练习复现 -> 迁移任务 -> 掌握度验证
```

如果没有专门的 course-learning 设施，Agent 容易把课程进度、工具 API、示例代码运行和用户掌握度混在一起。结果是“看过一节课”或“跑通一段代码”被误判为已经理解机制，也难以把课程概念迁移到用户真实任务。

因此，daedalus 需要把 `course-learning` 做成一等学习类型。

## 核心判断

course-learning 共享 daedalus 的底层原则：

- filesystem-first，学习状态必须可从文件恢复。
- WIP 严格，一次只推进一个 active topic。
- 不把聊天当长期记忆。
- 以最终产物倒推学习路径。
- 学习材料不是权威，要保留 first principles、trade-off 和迁移边界。
- 用户必须亲自完成关键观察、练习和理解闭环。

course-learning 的独立定义：

- 主材料：课程 syllabus、章节、练习、官方文档。
- 核心问题：这节课要建立哪个概念能力？
- 证据来源：课程章节、练习输出、用户复述、小实验、迁移题。
- 主要风险：照抄教程、概念似懂非懂、练习与真实任务脱节。
- 教练动作：引导预习、实验观察、概念重构、迁移练习。
- 阶段退出：能脱离教程复现机制并迁移到目标任务。
- 最终产物：course map、lesson labs、concept map、transfer lab、复习题库。

## 目标形态

新增一等学习类型：

```text
course-learning
```

它应该支持：

```text
daedalus init course-learning <project-name> --topic <topic-slug> --title <topic-title> --course-url <url>
```

生成的 project/topic 应明确表达：

```toml
[task]
kind = "course-learning"

[project]
source_kind = "course"
course_url = "..."
```

现有 `hugging-face-llm-course` project 已手动整理为 course-learning 类型，并保留当前 topic、notes、guides、demo 和历史证据。

## Course Learning Lifecycle

course-learning 使用以下阶段：

```text
01-need-aligner
02-syllabus-mapper
03-concept-roadmap
04-lesson-lab
05-mechanism-deep-dive
06-practice-transfer
07-capstone-lab
08-review-loop
09-closeout-archive
```

### 01-need-aligner

目标：确认为什么学这门课、服务哪个现实问题、最终要留下什么能力证明。

区别于 repo `goal-aligner`：这里不问“是否值得学这个 repo”，而是问：

- 这门课在用户真实目标中承担什么角色？
- 是顺序学、跳读、反向查阅，还是以项目问题驱动？
- 哪些章节是主线，哪些是旁路？
- 学完后要能做什么，而不只是“看完课程”？

核心产物：

```text
.daedalus/task-card.md
course-purpose.md
non-goals.md
```

### 02-syllabus-mapper

目标：把课程 syllabus 变成学习路线图。

课程通常已经有章节顺序，但 daedalus 不能盲目照单全收。需要建立：

- chapter / section 列表。
- 概念依赖。
- 必做练习、可跳过材料、延后材料。
- 当前 topic 与课程章节的映射。
- 每一章的可验证输出。

核心产物：

```text
shared/syllabus-map.md
shared/course-progress.md
guides/02-syllabus-mapper/README.md
```

### 03-concept-roadmap

目标：把章节标题转换成概念问题和学习检查点。

例子：

```text
Chapter 1 Text Generation
  -> Causal LM 到底在训练什么？
  -> labels = input_ids 为什么不是复制？
  -> logits 的 shape 为什么是 [batch, seq_len, vocab]？
```

核心产物：

```text
guides/03-concept-roadmap/README.md
shared/concept-map.md
```

### 04-lesson-lab

目标：把课程代码变成可观察实验，而不是照抄 recipe。

每个 lesson lab 都必须包含：

- 本节概念目标。
- 用户要先预测什么。
- 要运行的最小代码。
- 要打印的 input / output / shape / loss / artifact。
- 观察后要回答的问题。
- 哪些 warning 是语义信号，哪些只是环境噪音。

核心产物：

```text
guides/04-lesson-lab/<lesson>.md
notes/04-lesson-lab/README.md
demo/<course-exercise>/
```

### 05-mechanism-deep-dive

目标：当课程 API 把机制藏起来时，进入必要的源码 / 文档 / 论文解释。

进入条件必须很窄：

```text
只有当课程概念不能靠文档、实验和输出解释清楚时，才读源码。
```

例子：

- `Trainer.train()` 如何把 batch 变成 loss 和 backward。
- `pipeline("text-generation")` 如何展开成 tokenizer、model.generate、decode。
- `ForCausalLMLoss` 如何 shift labels。

核心产物：

```text
guides/05-mechanism-deep-dive/<mechanism>.md
notes/05-mechanism-deep-dive/README.md
```

### 06-practice-transfer

目标：把课程概念迁移到用户真实 topic 的小任务。

对当前 Hugging Face topic 来说，迁移不是“继续 ELI5”，而是：

```text
把 Causal LM / Trainer / pipeline 的理解
迁移到闲鱼买家 Agent suggestion next action 的 SFT / LoRA 数据与训练闭环
```

核心产物：

```text
guides/06-practice-transfer/README.md
demo/transfer-lab/
```

### 07-capstone-lab

目标：形成课程驱动的最终 mini project。

course-learning 的 capstone 不必是完整生产系统，但必须证明：

- 用户能选择合适章节和工具。
- 能独立搭建 pipeline。
- 能解释关键机制。
- 能跑出可比较结果。
- 能说清迁移边界。

当前 topic 的 capstone 是：

```text
suggestion dataset
  -> baseline
  -> LoRA/SFT train
  -> eval
  -> error analysis
  -> feedback data
  -> retrain
  -> compare
```

### 08-review-loop

目标：课程学习必须有复习与掌握度验证。

除了完成 demo 或练习输出，course-learning 还需要检查：

- 概念是否能脱离材料复述。
- 能否从空白写出最小代码。
- 能否解释 shape / loss / artifact。
- 能否判断何时不该使用某工具。
- 能否完成迁移题。

核心产物：

```text
review/mastery-map.md
review/question-bank.md
review/sessions/
```

### 09-closeout-archive

目标：关闭课程 topic，归档可复用知识。

归档前必须区分：

- 课程原文结论。
- 用户通过实验验证过的结论。
- Agent 的解释。
- 可迁移到业务的模式。
- 仍然薄弱的基础。

## Prompt 设施

新增 prompt namespace：

```text
system/prompts/course/
  01-need-aligner.md
  02-syllabus-mapper.md
  03-concept-roadmap.md
  04-lesson-lab.md
  05-mechanism-deep-dive.md
  06-practice-transfer.md
  07-capstone-lab.md
  08-review-loop.md
  09-closeout-archive.md
  course-learning-cli-contract.md
```

这些 prompt 应复用 common 层：

- `topic-discovery.md`
- `coach-questioning.md`
- `first-principles.md`
- `critical-lens.md`
- `human-owned-notes.md`
- `checkpoint-lifecycle.md`
- `review-guidance.md`
- `closeout-flow.md`

course prompt 应只引用 common 层能力和 course 层阶段 prompt。

## Skill 设施

新增 skill：

```text
.agents/skills/course-learning-coach/SKILL.md
```

职责：

- 只在用户学习 course / tutorial / official learning path / online curriculum 时触发。
- 根据 course-learning lifecycle 路由到对应 prompt。
- 明确区分 lesson guide、mechanism deep dive、practice transfer。
- 每轮最多推进一个 lesson / mechanism / transfer checkpoint。

触发例子：

- “我在学 Hugging Face Course 第 2 章。”
- “我跟着官方 tutorial 做了一个训练 notebook，但不知道自己在干什么。”
- “帮我把这节课程变成学习路线。”
- “这门课下一步学什么？”

## Template 设施

新增模板：

```text
system/templates/course/
system/templates/course-topic/
```

project-level 建议包含：

```text
.daedalus/state.toml
.daedalus/project-map.md
.daedalus/topic-board.md
shared/syllabus-map.md
shared/course-progress.md
shared/concept-map.md
shared/glossary.md
shared/evidence-registry.md
shared/transfer-patterns.md
source/README.md
```

topic-level 建议包含：

```text
.daedalus/task-card.md
.daedalus/outcome-map.md
.daedalus/todo.md
.daedalus/validation-log.md
guides/
notes/
demo/
review/
reflection/
```

course topic 的 guides 默认目录是：

```text
guides/01-need-aligner/
guides/02-syllabus-mapper/
guides/03-concept-roadmap/
guides/04-lesson-lab/
guides/05-mechanism-deep-dive/
guides/06-practice-transfer/
guides/07-capstone-lab/
guides/08-review-loop/
guides/09-closeout-archive/
```

## CLI 设施

新增或扩展命令：

```text
daedalus init course-learning <project-name> \
  --topic <topic-slug> \
  --title <topic-title> \
  --course-url <url>
```

`init course-learning`：

- 使用 course templates。
- 写入 `kind = "course-learning"`。
- 写入 `source_kind = "course"`。
- 创建 syllabus / course-progress / concept-map。
- 初始化 topic 为 course-topic。

既有项目的 course-learning 整理是一次性人工维护动作，不提供长期 CLI 命令。Agent 应直接修改具体文件、保留学习证据，并在修改后运行 validate。

## Validate 设施

`daedalus validate` 应理解 course-learning project。

project-level 检查：

- `kind = "course-learning"`。
- `source_kind = "course"`。
- `shared/syllabus-map.md` 存在。
- `shared/course-progress.md` 存在。
- `shared/concept-map.md` 存在。
- active topic 指向存在的 course topic。

topic-level 检查：

- task-card / outcome-map / todo 存在。
- 当前 active lesson / concept checkpoint 在 todo 中可恢复。
- lesson lab 有对应 guide 或 notes。
- 如果标记 lesson 完成，必须有用户运行或用户解释证据。
- 如果进入 transfer/capstone，必须能指向前置 course concepts。

validation 不应该要求用户已经写完整 notes；它只检查结构和生命周期一致性。用户掌握证据仍然来自 notes、运行输出、review 和 closeout。

## 当前 Hugging Face Project 一次性整理

现状：

```text
workspaces/projects/hugging-face-llm-course
  .daedalus/state.toml: 沿用旧类型标识
  project.source_kind: 沿用旧 source 标识
  topic-board: 沿用旧 project 文案
  guides/04-debugger-guide: 实际承担 lesson-lab / mechanism-deep-dive
```

目标：

```text
kind = "course-learning"
source_kind = "course"
course_url = "https://huggingface.co/learn/llm-course/en"
```

整理时不要破坏现有学习现场。建议分两步：

1. 引入 course-learning 类型、prompt、template 和 validate。
2. 手动更新 `hugging-face-llm-course` 的 project/topic metadata，并保留历史目录说明：

```text
guides/04-debugger-guide
  -> 保留为整理前的历史学习现场
  -> 后续新 guide 使用 course-learning 阶段命名
```

后续新文件应逐步使用：

```text
guides/04-lesson-lab/
guides/05-mechanism-deep-dive/
```

## 实现边界

本计划不是要把 daedalus 一次性改成支持所有学习材料类型。

本计划只要求 course-learning 成为一等类型。book-learning、paper-learning 可以以后参考同一模式，但不要在本轮一起实现。

## 验收标准

完成后必须满足：

- 可以用 `daedalus init course-learning ...` 创建课程学习 project。
- 新 project 的 state / project-map / topic-board 使用 course-learning 语义。
- course-learning 有独立 prompt namespace 和 skill。
- Agent 在 Hugging Face Course 这类任务中会触发 course-learning-coach。
- `daedalus validate` 能校验 course-learning project/topic。
- 现有 `hugging-face-llm-course` 已被手动整理为 course-learning project。
- 新增或修改的测试覆盖 init、validate、render。
- 当前 Hugging Face topic 的下一份 guide 使用 lesson-lab 或 mechanism-deep-dive 阶段命名。

## 建议实施顺序

### Step 1：领域模型与模板

- 新增 course project/topic templates。
- 扩展 state kind / source kind。
- 保留现有 project/topic lifecycle 机制。

### Step 2：CLI init / render / validate

- 新增 `init course-learning`。
- 更新 render 文案。
- 更新 validate 结构检查。
- 增加 CLI workflow tests。

### Step 3：Prompt 与 Skill

- 新增 `system/prompts/course/`。
- 新增 `.agents/skills/course-learning-coach/`。
- 更新 Agent 入口规则：course/tutorial/official curriculum 使用 course-learning skill。

### Step 4：当前项目手动整理

- 手动更新 `hugging-face-llm-course` 的 state、project-map、topic-board 和 active topic metadata。
- 保留历史 guides / notes / demo / reflection。
- 整理后运行 validate。

### Step 5：当前 topic 整理

- 新建 course 命名目录并为后续 guide 使用。
- 不强行重命名历史已提交目录，除非用户要求清理历史结构。
- 在 outcome-map 中标注历史目录的整理说明。

## 风险

### 风险 1：过度抽象

course-learning 不应变成“通用 learning type 大重构”。本轮只处理课程型材料。

### 风险 2：破坏现有 topic

当前 Hugging Face topic 已经有有效学习证据。整理必须保留现有文件，不因命名洁癖丢失上下文。

### 风险 3：把课程学习变成刷进度

course-progress 只能记录进度，不能替代掌握证据。每个完成的 lesson 至少要有实验、复述、迁移题或 review evidence。

### 风险 4：重复 common prompt

course prompt 应引用 common prompt 的原则，不复制大段规则，避免未来再次漂移。

## 最小不变量

无论具体实现细节如何，course-learning 必须保护以下不变量：

```text
课程进度不是掌握度。
跑通教程不是理解机制。
章节顺序不是学习目标。
练习输出必须能回到概念机制。
概念机制必须能迁移到用户真实任务。
```
