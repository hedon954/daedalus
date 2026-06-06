---
title: Micro Checkpoint Protocol
description: 在一个小学习步骤、slice、review 或验证形成闭环后，主动同步学习状态、拆分提交并规划下一步。用于避免进度只停留在聊天里。
scope: common
---

# Micro Checkpoint Protocol

## Agent Role

你是学习闭环维护者。每个小步骤完成后，不能只报告“测试通过”或“看起来没问题”；必须把当前学习系统推进到一个可恢复、可审计、可继续的稳定状态。

## Trigger

当出现以下任一情况时，必须执行 micro checkpoint：

- 用户完成一个小实现、修复、源码验证、review 或实验。
- Agent review 通过或补齐测试。
- 一个 slice 的子目标完成，即使整个 stage 还没完成。
- 用户接受某个设计结论或下一步方向。
- 产生了会影响后续学习路径的代码、notes、guides、todo 或 outcome-map 变更。

例外：如果用户只是说明当前已推进到新的实现/阅读边界，但尚未形成可验证闭环，优先执行轻量 `Current Cursor Sync`。它只更新 `todo.md` 中的当前光标，不要求创建 notes、不要求阶段状态变化，也不单独要求提交。

## Checkpoint Classification

先判断本轮闭环属于哪些类型：

- `implementation checkpoint`：代码、测试、demo 行为发生变化。
- `learning progress checkpoint`：阶段、slice、gap、验收状态发生变化。
- `knowledge checkpoint`：用户理解、Agent 校准、源码证据或可迁移模式需要进入 notes。
- `next-plan checkpoint`：产生新的 guide、行动卡、下一步地图或 stop rules。
- `critical lens checkpoint`：本轮形成了对素材约束、局限、忠实模仿边界或迁移取舍的新判断。

不同类型可以在同一轮产生，但提交边界要清楚。

## Required Actions

Review-only 不等于 learning-state-only。只要 review 或验证改变了完成度、风险清单、阻塞点、下一步行动或验收状态，即使用户没有要求改源码，也必须同步学习产物。源码是否可改由用户授权决定；学习地图是否过期由事实决定。

1. 先做证据 grounding：
   - 如果本轮涉及实现、测试、运行或 review，读取当前源码、测试、最近 diff、TODO 和相关验收标准。
   - 学习地图、guides、notes 只能辅助定位；不能单独证明实现状态或阶段完成。
   - 如果实现证据和学习地图冲突，以代码/测试/运行输出为准，并把地图同步列入本 checkpoint。

2. 更新学习状态：
   - active topic `.daedalus/outcome-map.md`
   - active topic `.daedalus/todo.md`
   - 相关 `notes/<stage>/README.md`
   - 相关 `guides/<stage>/README.md`

3. 记录学习证据：
   - 用户回答、观察、实践结果写入 `notes/`。
   - Agent-only 的下一步地图、行动卡、预读路径写入 `guides/`。
   - 新增 `guides/` / `notes/` 专题文件默认使用 `01-`、`02-`、`03-` 这类顺序前缀；`README.md`、assets、generated diagrams 和稳定索引文件除外。
   - 不要把下一步计划伪装成已经完成的学习证据。
   - 如果本轮完成源码专题、demo slice、关键设计或迁移判断，记录 critical lens：source constraint、faithful imitation、simplified/improved/discarded、transfer risk。

4. 运行最小验证：
   - 代码变更运行最小相关测试和格式检查。
   - 状态/模板变更运行 `daedalus validate` 或相关审计脚本。
   - 如果无法验证，说明阻塞原因和未验证风险。

5. 测试断言保持稳健：
   - 错误路径优先断言错误类别、枚举变体、是否存在错误，或公开 contract 中稳定的字段。
   - 不要默认断言完整错误文案、debug 字符串或内部格式；只有当错误文案本身是用户可见 API / CLI contract 时才锁定精确文本。
   - 如果为了诊断需要检查文案，优先使用最小稳定关键词，并在测试名里说明这是 contract。

6. 提交 checkpoint，除非用户明确要求不要提交：
   - 提交前先做 learning-map sync check：代码、测试、review 或验证是否改变了当前光标、完成度、风险、证据或下一步。
   - 如果改变了，先同步 `.daedalus/todo.md`、`.daedalus/outcome-map.md`、必要的 `.daedalus/long-context.md` 和相关 notes/guides，再 staging。
   - 使用 `type(scope): 中文描述`。
   - 当前闭环产物和下一步规划产物应分开提交。
   - 不要把“已完成实现”和“后续 guide”混在一个 commit 里，除非二者不可分割。
   - 不要把未验证假设写进提交信息。

7. 最终回复必须给出：
   - 当前 stage / slice / gap。
   - 本轮完成了什么。
   - 更新了哪些学习产物。
   - 验证结果。
   - commit hash，若已提交。
   - 下一步行动或下一轮问题。
   - 若本轮发生 repo-learning commit，必须使用 `Post-Commit Orientation` 形态重新定向学习现场；不要只回复 commit hash。

## Post-Commit Orientation

每次 repo-learning commit 后，最终回复必须包含这个最小结构：

```markdown
## Post-Commit Orientation
- Commit:
- Current:
- Completed:
- Updated artifacts:
- Validation:
- Next:
```

这个结构可以很短，但不能省略 `Current` 和 `Next`。提交后的核心任务是让学习者知道“这个 checkpoint 之后我们在哪里，下一步为什么值得做”。

## Commit Boundary Rule

使用下面的边界判断是否拆 commit：

```text
当前 checkpoint = 证明刚完成的事情已经稳定
下一步 guide = 帮助下一轮开始得更清楚
```

如果两者都出现，应拆成两个提交：

```text
feat(demo): 完成本轮实现与测试边界
docs(learning): 规划下一轮行动地图
```

这样回看历史时，用户能分清“已经完成的事实”和“下一步的计划”。

## Output Shape

```markdown
## Micro Checkpoint
- Current:
- Completed:
- Updated artifacts:
- Validation:
- Commit:
- Next:

## Critical Checkpoint
- Source constraint:
- Faithful imitation:
- Simplified / improved / discarded:
- Transfer risk:
```

保持简洁。不要把总结写成聊天记录；只保留能驱动下一步行动和长期恢复的信息。
