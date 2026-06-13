# Reflection 先挖掘后回顾改造方案

> 日期：2026-06-13
> 状态：已实施

## 背景

当前 `10-reflection` 的流程容易把 回顾 和 知识挖掘 混在一起：

```text
用户写 回顾
-> Agent 追问校准
-> Agent 再从 回顾 里挖 知识候选
-> 用户确认
-> 归档
```

这个顺序有两个问题：

- 回顾 容易变成“盲写回顾”：用户只复习主线，漏掉学习过程中暴露的通用能力和基础薄弱点。
- Agent 容易挖得太浅：只看 回顾，而不是系统扫描 guides、notes、demo、测试、源码和外部补充资料。

但 回顾 也不能无限扩散。完整 专题 已经经历 01-09 阶段，如果 回顾 再把所有旁路知识都讲透，学习战线会被拉得过长，反馈节奏会变差。

因此需要把 `10-reflection` 拆成两种动作：

```text
知识候选挖掘：贪心扫描，尽可能发现可迁移能力。
回顾反思：聚焦核心，由用户主动回顾、判断、确认理解。
```

## 核心原则

### 1. 先挖掘，再回顾

完成 01-09 后，Agent 不应立刻要求用户写 回顾。

它应该先扫描：

- 学习目标和 outcome map。
- `guides/`：Agent 引导过的阅读路径、实现切片、第一性原理解释。
- `notes/`：用户回答、纠正、争论、失败、设计决策。
- `demo/`：用户实际实现、简化、放弃或验证的机制。
- 测试和运行证据。
- 业务迁移材料。
- 用户要求补充的外部资料、底层原理和最佳实践。

然后生成一份 知识候选地图草稿。

### 2. draft 不是知识库正文

知识候选地图草稿 只是回顾地图，不是最终知识结论。

它可以由 Agent 贪心生成，因为它的职责是提醒用户：

- 这次 专题 里可能长出了哪些能力。
- 哪些点是核心主线。
- 哪些点是旁路技能。
- 哪些点暴露了基础薄弱。
- 哪些点需要用户确认、删掉或降级。

它不能直接进入 `knowledge-base/`。

### 3. 回顾 聚焦核心，但要点名重要旁路

回顾 不追求穷尽所有知识点，但至少要覆盖：

- 从最初学习目标中学到了什么。
- demo 实现中做了哪些关键决策、trade-off 和简化。
- demo 的核心架构图是什么。
- 哪些结论有证据支撑。
- 哪些设计只是学习时忠实模仿，不能直接迁移。
- 学习和实现过程中暴露了哪些基础薄弱点。

基础薄弱点不需要在 回顾 中展开成教程，但必须被点名。

### 4. 知识挖掘 贪心，知识库 归档克制

知识挖掘 要贪心：

- 尽可能挖掘核心机制、旁路技能、底层原理、工程模式、失败修正、外部对比。
- 尽可能关联已有知识：新增、补充、修正、对比。
- 如果没有合适旧条目，明确写“暂无可关联条目”，不要强行关联。

知识库 归档要克制：

- 用户最终筛选。
- 宁缺毋滥。
- 只保留高价值、可复习、可迁移、经过用户确认的条目。
- 被删掉的候选可以进入 backlog、复习计划或保持在 draft，不污染知识库。

## 新流程：循环而非线性

这不是一条单向流水线。真正有价值的是两个循环：

- `理解验证循环`：知识候选地图草稿 驱动 回顾，回顾 和 追问校准 反过来修正 知识候选地图；如果发现理解缺口，就回到 guides/notes/demo/source/external evidence 补证据。
- `候选筛选循环`：候选不会自动进入知识库；用户可以归档、延后、删除或要求重写，只有精选条目才进入知识库。

```mermaid
flowchart LR
    Start["01-09 主体学习完成"] --> Scan["Agent 贪心扫描"]
    Evidence["证据池<br/>guides / notes / demo / tests / source / external"] --> Scan
    Scan --> Map["知识候选地图草稿<br/>全部待用户确认"]

    subgraph ReviewLoop["理解验证循环"]
        Map --> Prompts["回顾提示<br/>围绕候选定制问题"]
        Prompts --> Reflection["用户回顾<br/>主动解释、判断、取舍"]
        Reflection --> Challenge["Agent 追问校准<br/>追问证据、边界、反例"]
        Challenge --> Clear{"理解足够清楚？"}
        Clear -- "否：补证据 / 复习 / 改 demo" --> Repair["定向补强<br/>回看源码、补 notes、跑测试、查外部资料"]
        Repair --> Map
        Clear -- "是：更新信心和边界" --> Revised["修订后的知识候选地图"]
    end

    subgraph CurationLoop["候选筛选循环"]
        Revised --> Selection{"用户筛选候选"}
        Selection -- "归档" --> KB["知识库<br/>精选归档"]
        Selection -- "延后" --> Deferred["复习 / backlog<br/>稍后补强"]
        Selection -- "删除" --> Deleted["删除<br/>不污染知识库"]
        Selection -- "修订" --> Map
    end

    KB --> Complete["专题完成"]

    classDef evidence fill:#f8fafc,stroke:#94a3b8,color:#0f172a
    classDef draft fill:#f5f3ff,stroke:#8b5cf6,color:#2e1065
    classDef loop fill:#fff7ed,stroke:#fb923c,color:#431407
    classDef archived fill:#ecfdf5,stroke:#10b981,color:#064e3b
    class Evidence evidence
    class Map,Revised draft
    class Prompts,Reflection,Challenge,Repair,Selection loop
    class KB,Complete archived
```

## 新状态

`10-reflection` 内部仍然可以采用 gate，但这些 gate 只是外层推进坐标，不代表内部没有循环。

```text
KnowledgeDraftReady
-> CloseoutPromptReady
-> ReflectionReviewLoop
-> CandidateMapRevised
-> UserSelectionConfirmed
-> KnowledgeArchived
-> TopicCompleted
```

`ReflectionReviewLoop` 可以重复多次：如果追问校准发现理解缺口，就回到证据池补证据、补 notes、补测试或补外部资料，再修订知识候选地图。

这些 gate 应同时体现在 prompt 协议、文件产物、CLI 校验和 TUI 展示中。prompt 负责引导，artifact 负责恢复现场，CLI 负责结构约束和一致性检查，TUI 负责让学习者看见当前循环位置。

## 新产物

建议新增：

```text
01-knowledge-candidate-map.md
02-closeout-prompts.md
```

其中 `01-knowledge-candidate-map.md` 是 Agent 生成的 draft，包含：

- `主题核心`：主题主线里最值得长期保留的机制、约束和取舍。
- `旁路技能`：旁路但可迁移的工程技能。
- `基础薄弱点`：学习中反复暴露的底层知识缺口。
- `工程模式`：demo 中形成的可复用实现模式。
- `外部对比`：外部资料和最佳实践对比。
- `学习方法`：可提升到 daedalus 全局的学习方法。

每个候选至少包含：

```text
标题：
类型：
价值：
证据：
状态：新增 / 补充已有 / 修正已有 / 对比已有 / 暂无可关联
掌握度：已吸收验证 / 需要用户确认 / 仅外部参照
范围：主题核心 / 旁路技能 / 跨主题方法
建议动作：归档 / 延后 / 复习 / 删除
```

`02-closeout-prompts.md` 基于知识候选地图生成，用来引导用户写回顾。

它不应是通用问卷，而应贴合当前 专题，例如：

- 这个 专题 的主线问题是什么？
- demo 中最关键的三个设计选择是什么？
- 哪些候选知识你已经真正掌握？
- 哪些候选知识只是听过、做过一次，但还不稳？
- 哪些薄弱点在未来 专题 中应该重点复习？

## 对现有规则的改造点

### `closeout-flow.md`

把 `KnowledgeGate` 前移：

```text
ChallengeResolved 后才第一次挖知识
```

改为：

```text
主体学习完成后先生成 知识候选地图草稿
再用 知识候选地图 驱动 回顾
```

### `archive-knowledge.md`

明确三层区别：

```text
知识候选地图：Agent 贪心挖掘，全部是草稿。
回顾：用户主动回顾，验证理解。
知识库：用户确认后的精选归档。
```

### `human-owned-notes.md`

补充：

- Agent 可以生成 知识候选地图草稿。
- Agent 不可以把 draft 当成用户理解。
- 用户 回顾 要围绕 draft 做确认、否定、补充和筛选。

### `reflection/closeout.md` 模板

增加一句：

```text
进入回顾前，daedalus 应先给你一份知识候选地图草稿。
你不需要照单全收；你要用回顾判断哪些真的成为了你的理解，哪些只是路过。
```

## 高频薄弱点机制

如果某个基础薄弱点在多个 专题 中反复出现，例如：

- Tokio / async runtime。
- sandbox / OS isolation。
- 网络协议。
- 数据库事务。
- 编译原理。
- Rust 生命周期和并发。

Agent 应标记为高频薄弱点，并建议：

- 进入 review plan。
- 进入 backlog。
- 补充已有知识库条目。
- 启动独立学习专题。

这能让 daedalus 不只是记录“学过什么”，还知道“用户反复卡在哪里”。

## 当前 Codex Topic 的迁移方式

`tools-permissions` 已经写完 回顾，所以不要求用户重写。

本次按新流程补做：

```text
1. 扫描回顾 / guides / notes / demo / tests / external references。
2. 生成 知识候选地图草稿。
3. 用已经完成的 回顾 和对话 追问校准 修订 知识候选地图。
4. 输出最终候选清单。
5. 用户筛选。
6. 只归档用户确认的高价值条目。
```

## 验收标准

- Agent 不再只从 回顾 提 1-3 个候选。
- 进入 回顾 前，能先生成 知识候选地图草稿。
- 回顾提示能引用知识候选地图草稿，而不是通用问卷。
- 知识候选地图能覆盖主题核心、旁路技能、基础薄弱点、工程模式和外部对比。
- 用户可以删除、延后或确认候选。
- 知识库 只写用户确认后的精选条目。
- 如果同一薄弱点重复出现，Agent 会标记为高频薄弱点。

## 风险

- draft 太大，反而增加用户压力。
  - 解决：知识候选地图 分层展示，先给总览，再展开细节。
- Agent 把 draft 写成结论。
  - 解决：所有 draft 文件必须显式标记 `draft / 待用户确认`。
- 知识库 变胖。
  - 解决：最终归档前必须有用户筛选，默认宁缺毋滥。
- 回顾 被 知识候选地图 带偏。
  - 解决：回顾提示 仍围绕学习目标、demo 决策、trade-off、架构图和薄弱点，不让旁路盖过主线。

## 实施顺序

1. 更新 common prompts：`回顾-flow.md`、`归档-knowledge.md`、`human-owned-notes.md`。
2. 更新 `10-reflection.md`，把 10 阶段拆成 draft mining、回顾 prompt、追问校准、筛选、归档。
3. 更新 `reflection/回顾.md` 模板。
4. 更新专题模板，初始化 `reflection/` 的知识候选地图和回顾提示入口。
5. 更新 CLI validate，检查 10 阶段的知识候选地图草稿、回顾、筛选和知识归档证据。
6. 更新 TUI，让学习者能看到 `10-reflection` 当前循环位置、draft map、回顾提示、筛选 状态和下一步动作。
7. 为当前 Codex 专题生成知识候选地图草稿，并按新流程完成筛选和归档。
