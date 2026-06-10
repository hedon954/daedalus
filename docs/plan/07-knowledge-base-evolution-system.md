# 知识库自进化体系方案

> 日期：2026-06-10
> 状态：拟定

## 背景

当前 `knowledge-base/` 的目录主要是占位分类。它能保存内容，但还不能稳定回答：

- 我到底长出了什么能力？
- 哪些知识以后会在真实问题中被想起？
- 哪些知识只是资料摘要，尚未转化为能力？
- 哪些知识需要复习、验证、迁移或重组？

知识沉淀的目标不是“保存更多笔记”，而是把学习过程提炼成可回忆、可验证、可迁移、可迭代的能力系统。

## 第一性原理

一条知识只有同时满足下面条件，才值得进入知识库：

- 有问题入口：未来遇到什么场景会想起它。
- 有能力出口：掌握后能做成什么事。
- 有机制模型：不只是结论，还知道为什么成立。
- 有证据来源：能追溯到源码、书、文章、demo、测试或业务案例。
- 有迁移边界：知道什么时候适用，什么时候不能照搬。
- 有复习任务：能通过回忆、解释、对比或实践证明自己还掌握。
- 有演化空间：随着新案例进入，可以拆分、合并、降级或废弃。

核心原则：

```text
文件夹 = 存储模型
技能树 = 学习模型
索引页 = 导航模型
链接 = 关系模型
元数据 = 状态模型
练习 = 验证模型
```

## 目标结构

Markdown 是唯一可信源；前端或网页只是阅读视图。

```text
knowledge-base/
  index.toml       机器索引和状态
  concepts/        原子概念：这是什么，为什么存在
  skills/          能力条目：我能做什么
  patterns/        可迁移模式：反复出现的问题解法
  problems/        问题入口：我经常遇到的真实问题
  cases/           案例证据：一次学习或应用的完整上下文
  source-maps/     来源映射：资料到知识的可追溯链路
  trees/           技能树和学习路径
  drills/          复习、迁移、批判和实作练习
  index/           人可读导航页
  site/            可选前端投影
```

不要把所有内容塞进领域目录。领域应放在 `trees/` 或 `index/` 中组织，例如 `trees/ai-engineering.md`，再通过链接连接 `concepts/`、`skills/`、`patterns/`、`problems/`。

## 知识类型

- `concepts/`：原子概念。回答“它是什么、解决什么问题、常见误解是什么”。
- `skills/`：能力条目。回答“我掌握后能做什么、当前水平如何、证据是什么”。
- `patterns/`：可迁移模式。回答“什么场景下复用、有什么取舍、何时不要用”。
- `problems/`：问题入口。回答“这个问题为什么难、当前最佳答案是什么、还有哪些疑问”。
- `cases/`：案例证据。保存学习、demo、业务应用的真实过程和结论。
- `source-maps/`：来源映射。记录从 repo、书、论文、文章中提取了什么，以及哪些部分不应照搬。
- `trees/`：学习路径。用技能树组织能力依赖和下一步缺口。
- `drills/`：复习练习。用于主动回忆、迁移题、反例题、重构题和批判题。
- `index/`：导航面板。帮助用户快速进入当前学习、常用模式、开放问题和近期更新。

## 生命周期

知识不是一次生成就稳定，应持续演进：

```text
候选 -> 已关联 -> 已验证 -> 稳定 -> 已精炼
                         -> 已废弃
```

晋升条件：

- 有来源专题或案例。
- 有证据链接。
- 有用户校准。
- 有问题入口。
- 有能力出口。
- 有迁移边界。
- 有至少一个复习或迁移练习。

废弃条件：

- 来源过时。
- 被更好的知识合并。
- 无法在真实问题中复用。
- 没有证据支撑，且复习时无法解释清楚。

## 提取流程

专题结束或阶段闭环时，按下面流程沉淀：

```text
workspace 原始学习过程
-> source-map 来源映射
-> case 案例摘要
-> concepts / skills / patterns / problems 原子化提取
-> drills 复习与迁移练习
-> trees / index 更新导航
```

最重要的边界：

```text
workspace = 混乱的学习现场
knowledge-base = 经过校准的可复用知识
```

## 模板改造

新增模板：

```text
system/templates/knowledge/concept.md
system/templates/knowledge/skill.md
system/templates/knowledge/pattern.md
system/templates/knowledge/problem.md
system/templates/knowledge/case.md
system/templates/knowledge/source-map.md
system/templates/knowledge/tree.md
system/templates/knowledge/drill.md
system/templates/knowledge/index.md
```

所有模板至少包含：

- 回忆钩子
- 现实问题
- 第一性原理
- 机制模型
- 关键不变量
- 取舍
- 不要照搬的部分
- 迁移方式
- 证据来源
- 复习练习

不同类型再追加自己的字段。例如 `skills/` 需要能力等级、目标等级、练习任务和薄弱点；`patterns/` 需要适用条件、反例和 trade-off；`source-maps/` 需要来源信息和批判性判断。

## 自进化机制

知识自进化需要强推理、批判、归纳和重组能力。如果全部封装进 `daedalus` CLI，就等于要求 daedalus 自己实现一个高质量 agent，这会让项目过早背上过重的智能负担。

因此采用 skill-first 方案：

```text
Agent Skill 负责认知工作
daedalus CLI 负责确定性工作
Markdown 文件负责持久化
软链接负责多 agent 复用
```

认知型能力优先做成 skills：

```text
.claude/skills/daedalus-knowledge-extract/SKILL.md
.claude/skills/daedalus-knowledge-inspect/SKILL.md
.claude/skills/daedalus-knowledge-gaps/SKILL.md
.claude/skills/daedalus-knowledge-promote/SKILL.md
.claude/skills/daedalus-knowledge-reorganize/SKILL.md
```

其他 agent 的 skill 目录通过软链接复用同一份源：

```text
agents/skills/knowledge-extract -> ../../.claude/skills/daedalus-knowledge-extract
agents/skills/knowledge-inspect -> ../../.claude/skills/daedalus-knowledge-inspect
```

CLI 只保留低智能、可测试、确定性的基础能力：

```bash
daedalus knowledge validate
daedalus knowledge index
daedalus knowledge link-check
daedalus knowledge template
```

CLI 职责：

- `validate`：检查知识条目元数据、必填字段和状态是否合法。
- `index`：重建 `knowledge-base/index.toml`。
- `link-check`：检查内部链接、来源链接和孤立条目。
- `template`：生成不同类型知识条目的模板文件。

Skill 职责：

- `knowledge-extract`：从专题产物生成候选知识。
- `knowledge-inspect`：检查孤立、重复、缺证据、缺练习、缺迁移边界的条目。
- `knowledge-gaps`：从技能树和问题入口找能力缺口。
- `knowledge-promote`：判断候选知识是否可以晋升。
- `knowledge-reorganize`：提出重组建议，但不自动大规模改写。

边界原则：

- 需要解释、判断、批判、归纳、重写的，放在 skill。
- 需要校验、索引、模板、链接检查的，放在 CLI。
- skill 可以调用 CLI 获得确定性检查结果。
- CLI 不负责替用户“想明白”，只负责让产物结构稳定。

## 前端投影

可以内置只读知识库前端，但它不是知识源。

前端优先服务这些视图：

- 技能树视图：我正在长出哪些能力。
- 问题入口视图：我遇到某类问题时该看什么。
- 证据链视图：一条知识来自哪些学习材料和 demo。
- 复习视图：今天该回忆、解释、迁移或实作什么。
- 演化视图：哪些知识最近被合并、升级、废弃。

## 资料依据

本方案吸收了几个方向的结论：

- 检索练习和间隔练习：知识要能被主动回忆，而不是只被重新阅读。
- 刻意练习和反馈：能力要通过任务、表现标准和反馈闭环来提升。
- 布鲁姆分类法：知识沉淀要覆盖理解、应用、分析、评价和创造。
- Merrill 教学第一原则：学习应围绕真实问题，激活旧知识，再展示、应用和整合新知识。
- Evergreen Notes / Zettelkasten：长期知识应原子化、概念化、可链接、可演化。
- PARA：存储结构要服务行动，而不是按静态领域堆资料。
- SECI 知识创造模型：隐性经验需要外化，显性知识需要组合，并通过实践重新内化。

参考资料：

- [RetrievalPractice.org：检索练习](https://www.retrievalpractice.org/why-it-works)
- [Dunlosky 等：有效学习技术综述](https://journals.sagepub.com/doi/abs/10.1177/1529100612453266)
- [Bloom's Taxonomy：学习目标层级](https://cft.vanderbilt.edu/wp-content/uploads/sites/59/Blooms-Taxonomy.pdf)
- [Merrill's Principles of Instruction：真实问题驱动学习](https://students.tippie.uiowa.edu/tippie-resources/technology/instructional-design/models/merrill)
- [Andy Matuschak：Evergreen Notes](https://notes.andymatuschak.org/z5E5QawiXCMbtNtupvxeoEX)
- [Forte Labs：PARA 方法](https://fortelabs.com/blog/para/)
- [Forte Labs：渐进式总结](https://fortelabs.com/blog/basboverview/)
- [SECI 知识创造模型](https://ascnhighered.org/ASCN/change_theories/collection/seci.html)

## 需要实现

- 调整 `knowledge-base/` 目录结构。
- 新增 `knowledge-base/index.toml`。
- 新增知识模板。
- 新增知识沉淀相关 skills。
- 新增跨 agent skill 软链接目录。
- 新增知识校验、索引、链接检查和模板命令。
- 改造知识导出流程，由 skill 生成候选知识，由 CLI 做结构校验。
- 新增复习计划与知识条目的绑定。
- 后续实现只读前端投影。

## 验收标准

- 每个专题结束后，用户能看到自己新增了哪些能力、概念、模式和问题答案。
- 任意知识条目都能追溯到来源证据。
- 任意稳定知识都有复习或迁移练习。
- 技能树能展示能力依赖和下一步缺口。
- 用户能从真实问题入口找到相关知识。
- 系统能发现重复、孤立、过期、缺证据、缺练习的知识。
- 知识库随专题增多而演化，不只是自然堆积。
