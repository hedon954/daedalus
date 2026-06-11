# Human-Owned Notes 与知识库协议改造方案

> 日期：2026-06-11
> 状态：已实现

## 背景

当前 daedalus 已经把知识生成能力从 CLI 移到 skill，但这个设计仍然有一个根本问题：AI 仍可能成为知识提取者，只是换了入口。

daedalus 的核心不应该是替用户总结，而是帮助用户完成总结这个动作。学习材料经过用户学习后，用户必须先写出自己的 notes，哪怕粗糙、不完整、有误解也可以。AI 的价值是追问、挑战、校准、关联和组织，而不是代写用户的理解。

目标链路应改为：

```text
learning material
  -> human studies
  -> human writes rough notes during learning
  -> AI asks review questions
  -> human revises notes
  -> topic closeout
  -> human writes a deliberate learning retrospective
  -> AI challenges / links / checks consistency
  -> AI formats only after user understanding exists
  -> knowledge-base
```

## 核心原则

```text
意义萃取归用户
结构辅助归 AI
知识库保存 reviewed human retrospective understanding
```

最底层的学习判断：

```text
用户主动回顾不是形式，而是能力提升的必要流程。
```

只有用户自己思考、自己总结、自己经历表达不清、证据不足、迁移失败、反例挑战这些“思考摩擦”，理解才可能真正长出来。daedalus 不能为了效率跳过这段摩擦，因为跳过它就等于跳过成长本身。

但主动回顾非常难：用户容易卡住、空泛、逃避、过早满足于“我好像懂了”。所以 daedalus 的任务不是替用户完成回顾，而是用 AI 能力帮助用户穿过这段困难：

- 降低开始成本：给空模板、问题入口、最小起笔提示。
- 提高反馈密度：及时指出含糊、跳步、证据缺口和矛盾。
- 保持推进感：把大总结拆成小问题、小验证、小修订。
- 增加迁移压力：要求用户举反例、换场景、对比旧知识。
- 提供外部参照：关联源码证据、历史 notes、业内最佳实践和 trade-off。
- 保护用户表达：保留用户自己的粗糙语言，不把它替换成 AI 的流畅总结。

其中外部参照不应该只依赖模型记忆。daedalus 应更主动搜索公网资料和业内最佳实践，用它们帮助用户校准理解：

- 当用户提出设计判断时，主动查找主流项目、官方文档、论文、工程博客或行业实践进行对比。
- 当用户总结 trade-off 时，搜索不同方案的真实约束和失败案例。
- 当用户准备迁移知识时，查找相似场景下的成熟做法，帮助判断哪些部分可迁移、哪些部分不能照搬。
- 当用户的 notes 只有单一来源时，引入外部参照，避免把一个项目的局部选择误当成通用真理。
- 外部资料只能作为 challenge / comparison / reference，不能替代用户自己的总结。

AI 可以是：

- `mirror`：映照用户理解是否清楚。
- `coach`：提出下一步思考问题。
- `challenger`：挑战含糊、跳步、矛盾和无证据判断。
- `organizer`：整理结构、链接、标签和索引。
- `connector`：关联旧知识、源码证据和业内最佳实践。

AI 不应该是：

- automatic summarizer
- main extractor
- notes ghost writer
- knowledge-base author of first resort

## 新协议：Human-Owned Notes

### 1. Guides 由 AI 写

`guides/` 是 AI 的空间，负责：

- 阅读路径。
- 下一步行动地图。
- 思考题。
- 证据检查点。
- 第一性原理提示。
- 外部最佳实践对比入口。

AI 可以主动创建和更新 guides，因为 guides 是“学习引导”，不是“用户理解”。

### 2. Notes 默认由用户写

`notes/` 是用户理解的空间，默认不能由 AI 新建正文总结。

当用户要求：

```text
帮我写 notes
帮我总结成 notes
把刚刚内容整理成 notes
```

AI 应默认拒绝代写正文，并改为：

1. 询问这篇 note 想记录的主题。
2. 给出 3-7 个关键问题。
3. 要求用户先写粗糙版本。
4. 等用户写完后再 review。

允许的 AI 行为：

- 创建空 note 模板。
- 写 `AI Questions`。
- 写 `AI Challenge`。
- 写 `Open Questions`。
- 写 `Suggested Links`。
- 写 `Formatting Proposal`。

不允许的 AI 行为：

- 直接生成新的用户理解正文。
- 把 AI 总结伪装成用户 notes。
- 在没有用户原始表达时生成 knowledge-base 条目。

### 3. Notes Review 流程

用户写完 notes 后，AI 进入 review/coaching，而不是改写：

```text
user note
  -> clarity review
  -> evidence review
  -> contradiction review
  -> transfer review
  -> relation review
  -> user revises
```

AI 应重点检查：

- 是否说明现实问题。
- 是否有用户自己的判断。
- 是否能从第一性原理推导。
- 是否有源码、实验或案例证据。
- 是否存在跳步、模糊词或循环解释。
- 是否能举反例。
- 是否能迁移到新场景。
- 是否和旧知识冲突或互补。
- 是否需要搜索公网资料或业内最佳实践来校准判断。

### 4. Knowledge-base 归档门槛

`knowledge-base/` 只能保存 reviewed human retrospective understanding。

它不是从 `notes/` 自动提取出来的摘要。`notes/` 是学习过程中的原始证据和阶段性理解；进入知识库前，用户必须在 topic 完成后主动做一次完整回顾总结。这个 closeout retrospective 是学习闭环的一部分，而不是可选整理动作。

进入 knowledge-base 前必须满足：

- 有用户原始 note。
- 有 AI challenge 或 review 记录。
- 有用户修订后的理解。
- 有 topic closeout retrospective：用户用自己的语言回答“这次学习到底改变了什么理解、长出了什么能力、哪些地方还不能迁移”。
- 有证据链接。
- 有适用边界和不要照搬的部分。
- 有复习或迁移练习。

AI 可以做：

- 格式化。
- 链接补全。
- 一致性检查。
- 分类建议。
- 与旧知识关联。
- 主动搜索公网资料，与业内最佳实践对比。

AI 不可以做：

- 在没有用户 notes 的情况下生成知识条目。
- 自动从学习材料提取知识库正文。
- 自动判断用户“已经懂了”。

## 需要删除的旧设计

以下内容体现了“AI 知识提取者”心智，应整体删除或改名，不保留功能入口：

- `.claude/skills/daedalus-knowledge-extract/`
- `.claude/skills/daedalus-knowledge-promote/`
- `.claude/skills/daedalus-knowledge-inspect/`
- `.claude/skills/daedalus-knowledge-gaps/`
- `.claude/skills/daedalus-knowledge-reorganize/`
- `agents/skills/knowledge-*` 软链接。
- prompt 中“AI 萃取知识体系 / 自动晋升 / promotion pipeline”的表述。
- README 中 “skill-driven knowledge extraction / archival pipeline” 的旧流程图。
- `system/templates/knowledge-system/` 中鼓励 AI 从 evidence 直接萃取的模板。

可以保留的能力：

- `daedalus knowledge template`
- `daedalus knowledge index`
- `daedalus knowledge list`
- `daedalus knowledge link-check`
- `daedalus knowledge validate`

这些 CLI 能力只做结构，不做理解。

## 新增或改写的协议文件

建议新增：

```text
system/prompts/common/human-owned-notes.md
```

职责：

- 说明 AI 何时必须拒绝代写 notes。
- 定义 AI 如何提出 notes writing questions。
- 定义 AI 如何 review 用户 notes。
- 定义 AI 如何 organize 已完成 notes。
- 定义何时允许进入 knowledge-base。

建议改写：

```text
system/prompts/common/archive-knowledge.md
.claude/skills/repo-learning-coach/SKILL.md
system/prompts/common/agent-operating-contract.md
README.md
AGENTS.md
```

## 新 workflow

### 用户想写新 notes

```text
User: 帮我记录一篇关于 X 的 notes
AI:
  - 不直接写正文
  - 提出 X 的关键问题
  - 给出空模板
  - 要求用户先写 rough note
```

### 用户已经写了 notes

```text
User: review 我的 notes
AI:
  - 检查清晰度
  - 挑战证据
  - 找矛盾
  - 追问反例
  - 建议关联旧知识
  - 不直接替用户改写正文
```

### 用户完成修订后

```text
User: organize 这篇 notes
AI:
  - 可整理标题和层级
  - 可补链接
  - 可提 taxonomy 建议
  - 可生成 knowledge-base 模板
  - 但不新增未经用户确认的知识结论
```

### Topic 完成后进入知识库前

```text
User: 这个 topic 可以归档知识了
AI:
  - 不直接从 notes 提取 knowledge-base 正文
  - 引导用户完成 closeout retrospective
  - 要求用户回答能力变化、核心判断、证据、迁移边界、遗留问题
  - 主动搜索外部资料和业内最佳实践，提出对比问题
  - review 用户回顾总结
  - 通过 challenge 后，再协助格式化和链接到 knowledge-base
```

## 验收标准

- 当用户要求 AI 新写 notes 时，AI 会拒绝代写，并改为提问。
- 当用户提供 rough notes 后，AI 会 challenge 和 coach，而不是直接重写。
- 当用户要求归档知识时，系统会检查是否存在用户 notes 和 review 记录。
- 当 topic 完成准备归档时，系统会要求用户先写 closeout retrospective，不能只从 notes 自动提取知识。
- 当前所有 `daedalus-knowledge-*` AI 提取型 skill 被删除。
- README 和 prompts 中不再出现 AI 作为知识主提取者的流程。
- CLI 仍只保留 knowledge 结构能力。

## 风险

- 用户可能觉得流程变慢；需要允许“空模板 + 关键问题”降低开始成本。
- AI 可能在 review 时不自觉总结；需要要求新增结论必须标为 `AI suggestion`，不能混入用户正文。
- 旧文档里关于 extraction / promotion 的历史记录很多；当前改造只清理活跃协议，历史 plan/changelog 可保留为演进记录，但要避免被 prompt 加载。

## 迁移策略

1. 新增 `human-owned-notes.md` 协议。
2. 删除 AI 知识提取型 skills 和软链接。
3. 改写 repo-learning coach 的 notes / knowledge 规则。
4. 改写 archive knowledge：从“提取知识”改为“归档用户 closeout retrospective 后的已验证理解”。
5. 清理 README / AGENTS / operating contract 的措辞。
6. 保留 CLI knowledge template/index/list/link-check/validate。
7. 运行 grep，确保活跃协议不再引导 AI 代写 notes 或自动萃取知识。
