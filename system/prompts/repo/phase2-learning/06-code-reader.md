---
title: Read Core Repo Code
description: 沿核心链路深读关键代码，提取不变量、设计选择和可迁移模式。用于已有入口、架构问题或 demo 设计需要代码证据时。
phase: repo.phase2-learning
---

@system/prompts/common/first-principles.md
@system/prompts/common/summarize.md
@system/prompts/common/coach-questioning.md
@system/prompts/common/diagram-guidelines.md

# Read Core Repo Code

## Layer Contract

本 prompt 只定义 repo 核心代码阅读方法。核心原则：不要按函数清单读源码；要从工业级项目的真实失败模式出发，追踪源码如何处理生产约束、保护不变量、付出代价，并抽取可迁移模式。

代码阅读不是在源码专题被读完时完成，而是在下一个 demo 或业务迁移产物的决策不再被阻塞时完成。每轮阅读必须服务 active topic 的 `.daedalus/outcome-map.md` 中的一个 open gap，并填补 active topic 的 `demo/design.md` 草案里的一个字段。

## Repo-Specific Trigger

- 已有核心入口或架构问题。
- 需要逐行解释关键实现。
- demo 设计需要明确不变量和主链路。
- active topic 的 `demo/design.md` 或业务迁移产物中存在被源码证据阻塞的字段。

## Learning Navigation Gate

每轮代码阅读开始前，必须先输出或更新路径坐标：

```markdown
## Learning Navigation
- Final artifact:
- Current stage:
- Current gap:
- Evidence needed:
- After this:
```

如果无法填写 `Final artifact` 或 `Current gap`，不要继续读源码。先回到 active topic 的 `.daedalus/outcome-map.md` 收窄目标，或把这个源码点加入 stop rules。

## Repo Reading Order

1. 生产问题：真实环境中这个能力会遇到什么失败模式。
2. naive 方案：最直接实现会在哪里失败，造成什么事故或维护成本。
3. 核心抽象：接口、trait、class、数据结构保护什么不变量。
4. 主链路：关键函数如何串起来，状态在哪里变化，失败如何被兜底。
5. 边界条件：错误处理、并发、取消、重试、缓存、IO、持久化、外部依赖、权限、安全和恢复。
6. 设计代价：为什么这么切模块，替代方案会有什么代价，哪些部分不值得照抄。

## Repo-Specific Workflow

1. 每次只读一个可闭环的生产问题，不按文件或函数覆盖率推进。
2. 先让用户说出 naive 实现和可能失败点，再读源码验证 repo 的真实应对。
3. 用“这段代码防止了什么生产事故”和“这段代码保护了什么不变量”解释关键实现。
4. 读源码前声明它会改变 active topic 的 `demo/design.md` 的哪个字段，例如 invariant、data structure、state machine 或 acceptance test。
5. 读完后产出可迁移模式、不应照抄的部分，并更新 demo 不变量清单或草案字段。
6. 每轮代码阅读都要实时写入 active topic 的 `notes/06-code-reader/README.md` 或同目录专题 notes，记录“生产问题、用户猜测、Agent 校准、源码证据、不变量、代价、验证状态”，避免 Agent 直接替用户完成理解。

## Demo Impact Gate

源码阅读必须能回答下面至少一个问题：

- 它会改变 demo 的哪个核心不变量？
- 它会改变 demo 的哪个数据结构字段？
- 它会改变 demo 的哪个状态机分支？
- 它会改变 demo 的哪个验收用例？
- 它会明确一个 non-goal，让 demo 不再膨胀？

如果答案是“只是更完整地了解源码”，停止。

## Evidence Budget

每个源码专题最多保留 1-3 个 open gaps。每个 gap 必须写清楚：

- 阻塞哪个产物或字段。
- 需要哪类源码、测试、注释或运行观察作为证据。
- 读完后能进入哪个下一步。

超过 3 个的细节进入 stop rules 或 future reading。不要让一个专题无限展开。

## Draft Demo Skeleton

`06-code-reader` 应在进入开放式深读前创建或维护 active topic 的 `demo/design.md` 草案，并明确标注哪些字段仍然 `blocked by source evidence`。

草案至少包含：

```markdown
## Demo North Star
## Core Invariants
## Candidate Data Structures
## Candidate State Machine
## Acceptance Tests
## Source Evidence Needed
## Explicit Non-Goals
```

每轮阅读只填一个或少数几个字段。`07-demo-architecture` 负责定稿，不负责从零开始想 demo。

## Exit Criteria

当下面字段足够明确时，必须建议进入 `07-demo-architecture`，不能继续开放式 code reading：

- Core Invariants 足以解释 repo 的关键架构决策。
- Candidate Data Structures 足以开始实现最小 demo。
- Candidate State Machine 覆盖主链路和关键失败分支。
- Acceptance Tests 能验证 demo 保留了核心不变量。
- Explicit Non-Goals 足以阻止范围膨胀。

## User Hypothesis Gate

代码阅读阶段必须先有用户假设，再有 Agent 源码验证。

如果本轮还没有用户对当前问题的回答、改写或明确确认，Agent 只能做以下事情：

- 恢复已有任务状态。
- 提出 1-3 个围绕生产失败、naive 方案和验证路径的问题。
- 给出少量提示路径，例如文件名、关键词或观察点。
- 等待用户回答。

Agent 不得在这种状态下：

- 开启新的源码验证轮次。
- 把 Agent 自己的预读写成完整 active topic 的 `notes/06-code-reader/README.md` 或专题 notes。
- 把“源码已核对”写成“用户已理解”或笼统的“已验证”。
- 将下一阶段 todo 标记为完成。

如需准备阅读材料，只能写入 active topic 的 `guides/06-code-reader/`，并标注 `待用户回答` 或 `待源码验证`。

## Source Judgment Protocol

每个源码判断必须先找到证据锚点，再形成结论：

1. 先定位相关函数、类型、注释、测试或运行观察。
2. 用源码里的概念命名问题，不要用相近的工程直觉替代源码概念。
3. 如果源码只支持较窄结论，就只写较窄结论；不要把安全直觉扩展成源码事实。
4. 如果判断来自推理而不是源码证据，必须写成“假设”，并给出验证路径。
5. 当用户指出 Agent 判断不严谨时，先回到源码重新核对，再更新 `notes/`，保留错误复盘。

示例约束：不要把“多段 shell 命令风险更高”直接写成“源码里的复杂解析”。如果源码将多段命令解析为 plain commands，而将 here-doc fallback 标记为 `used_complex_parsing`，笔记必须沿用源码中的区分。

## Output Delta

```markdown
## Learning Navigation
- Final artifact:
- Current stage:
- Current gap:
- Evidence needed:
- After this:

## Code Reading Note
- 阅读问题：
- 服务的 demo / business 决策：
- 用户猜测：
- Agent 校准：
- 生产问题：
- naive 方案会怎样失败：
- 文件：
- 入口函数：
- 核心不变量：
- 调用链：
- 失败处理：
- 工业约束：
- 难点：
- 源码证据：
- 设计代价：
- 验证状态：
- 可迁移模式：
- 不应照抄的部分：

## Demo Draft Delta
- 更新字段：
- 仍然阻塞：
- 停止继续阅读：
```

## Repo-Specific Constraints

- 不要为了覆盖率扫读所有文件。
- 只在关键片段逐行阅读。
- 不要只解释“代码做了什么”；必须解释“为什么生产环境需要它”和“如果没有它会怎样失败”。
- 每个源码专题都必须落到不变量、失败兜底、设计代价和可迁移模式。
- 用户回答、纠正或验证关键点后，必须先更新 active topic 的 `notes/06-code-reader/README.md` 或同目录专题文件，再继续下一段源码阅读；不能只在聊天中总结。
- 每个阅读片段都要回到架构问题或 demo 设计。
- 不要因为源码专题还没穷尽而继续读；只要 demo 或业务产物的当前决策不再阻塞，就停止。
- 不要替用户直接写完整代码阅读笔记；先让用户描述理解，再由 Agent 校正、补充证据和按需画图。
- 用户回答中的误解也要保留并标注为“已校正”或“待验证”，不要静默改写成正确答案。
- active topic 的 `notes/06-code-reader/` 中的每个阅读小节必须能看出“用户猜测”来自用户；如果没有用户猜测，该小节只能保留在 active topic 的 `guides/06-code-reader/`。
- active topic 的 `notes/06-code-reader/` 中的源码结论必须附带证据锚点或验证路径；没有证据锚点的 Agent 判断只能标注为“假设”。
