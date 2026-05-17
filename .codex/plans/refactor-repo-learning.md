# Repo Learning 指令重构方案：以最终产物驱动学习

## Summary

当前 daedalus 的 repo learning 指令最大问题不是“不够会提问”，而是缺少贯穿全程的 **终点地图**。现有 prompt 有阶段、问题、notes 和 todo，但没有持续回答：

- 最终要交付什么？
- 当前步骤服务最终哪个产物？
- 我们在整条路径的哪里？
- 当前障碍是什么？
- 解决这个障碍后能进入哪个阶段？

建议重构为“终点产物优先”的学习协议：所有阅读、提问、调试、源码验证，都必须绑定到最终 demo / business transfer / knowledge export 中的某个待填空位。否则停止扩展。

## Key Changes

### 1. 新增 `Learning Outcome Map` 作为全程导航产物

新增一个常驻文件，建议路径：

`workspaces/<task>/.daedalus/outcome-map.md`

它不是普通总结，而是学习任务的仪表盘，包含：

- **North Star**：最终要产出的能力与最小 demo。
- **Final Artifacts**：最终必须交付的文件，例如 `demo/design.md`、`demo/README.md`、`notes/business-application.md`、knowledge-base entry。
- **Current Position**：当前处于哪条路径、哪个阶段、正在解决哪个缺口。
- **Artifact Dependency Graph**：当前阅读如何服务后续 demo / business transfer。
- **Open Gaps**：进入下一阶段前还缺哪些证据。
- **Stop Rules**：哪些源码细节不再继续读。

示例结构：

```markdown
# Outcome Map

## North Star
最终实现一个最小 Agent CLI，本地命令执行链路必须保留 Codex 的核心权限/沙箱不变量。

## Final Artifacts
- demo/design.md：待填 / 进行中 / 已完成
- demo/README.md：待填
- notes/business-application.md：待填
- knowledge-base entry：待填

## Current Position
当前阶段：06-code-reader 后半段
当前目标：从 auth/approval/sandbox 源码收敛 demo 不变量
当前障碍：还未把源码结论映射成 demo 状态机和数据结构

## Why This Step Matters
本轮阅读只服务于 demo 的 CommandRequest、ApprovalRequirement、SandboxRetryState 设计。

## Stop Rules
不再扩展 Windows sandbox、MCP elicitation、完整 TUI UI 细节。
```

### 2. 重写 `repo-learning-coach` 总规则：从 10 阶段流水线改成产物管线

现有 10-stage workflow 容易让人感觉“下一阶段是什么”，但不说明“为什么下一阶段值得做”。

建议新增一条总规则：

```markdown
Every repo-learning action must declare:
1. Which final artifact it advances.
2. Which missing field or decision it fills.
3. What evidence is needed.
4. What becomes possible after this step.
```

也就是每次继续学习前，都必须先说：

```text
我们现在不是在“继续读源码”；
我们是在补 demo/design.md 里的某个待定决策。
```

并强制每轮输出一个导航头：

```markdown
## Learning Navigation
- Final artifact: demo/design.md
- Current stage: 06-code-reader
- Current gap: sandbox retry state machine
- Evidence needed: orchestrator sandbox denied branch
- After this: can define demo execution state machine
```

### 3. 强化 `06-code-reader`：代码阅读必须受 demo 缺口约束

当前 `06-code-reader` 已经要求“每轮只读一个生产问题”，但仍可能无限展开。

建议改成：

```markdown
Code reading is not complete when the source topic is exhausted.
Code reading is complete when the next demo or business artifact decision is no longer blocked.
```

新增三个硬门槛：

- **Demo Impact Gate**：读这段源码前，必须说明它会改变 demo 的哪个字段。
- **Evidence Budget**：每个专题最多保留 1-3 个源码缺口；读完即停止。
- **Exit Criteria**：当 demo 不变量、数据结构、状态机、验收用例足够明确时，必须进入 `07-demo-architecture`。

对当前 Codex 任务，`06-code-reader` 的剩余阅读应被压缩为：

```text
只补 3 个缺口：
1. Decision 合成：决定 demo 如何聚合多段命令风险。
2. Runtime request assembly：决定 demo 的 CommandRequest 字段。
3. Orchestrator retry：决定 demo 的执行状态机。
```

### 4. 重写 `todo.md` 模板：从任务列表改为路径看板

当前 todo 有“当前 / 下一步 / 后续”，但用户看不到方向。

建议改成：

```markdown
## North Star
最终产物：

## Current Path
当前正在从 [阶段] 走向 [产物]。

## Now
- 当前问题：
- 为什么现在做它：
- 完成后解锁：

## Gaps Blocking Next Stage
- [ ] gap 1：阻塞 demo/design.md 的哪个决策
- [ ] gap 2：

## Stage Exit Criteria
- [ ] 可以写 demo 核心数据结构
- [ ] 可以写 demo 主链路
- [ ] 可以写 demo 验收用例

## Done
```

这样用户每次打开 todo 都能看到“我正在朝哪里走”。

### 5. 修改 `resume.md`：恢复状态必须输出路径坐标

当前 resume 只要求 5-8 行摘要：目标、阶段、已完成、下一步。这个不够。

建议恢复时必须输出：

```markdown
## 你现在在哪里
- Final artifact:
- Current stage:
- Current gap:
- Why this gap matters:
- What we will stop reading:
- What becomes possible after this:
```

例如当前任务应该恢复成：

```text
你现在在 06-code-reader 后半段。
不是继续泛读 Codex 权限系统，而是把 auth/approval/sandbox 收敛成 demo 不变量。
当前还缺 3 个 demo 决策：Decision 合成、CommandRequest 字段、Retry 状态机。
补完后进入 demo/design.md。
```

### 6. 前移 `07-demo-architecture`：让 demo 设计草案在代码阅读中就出现

当前流程是：

```text
06-code-reader -> 07-demo-architecture
```

这会导致 code reading 没有外部约束。

建议改成：

```text
06-code-reader starts with a draft demo skeleton.
Each code-reading round fills one field.
07-demo-architecture finalizes it.
```

也就是说，在 `06-code-reader` 中就创建 `demo/design.md` 草案，但标注为 `draft / blocked by source evidence`。

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

这样源码阅读每一步都有落点。

## Test Plan

用当前 Codex 学习任务验证新指令是否有效：

1. **Resume test**
   用户说“继续学习”时，Agent 必须先输出路径坐标，而不是直接读源码。
   预期：出现 `Final artifact / Current gap / After this`。

2. **Code-reading gate test**
   Agent 想读一个新源码点时，必须说明它填补 demo 的哪个决策。
   预期：如果无法映射到 demo，就停止或移入“不读清单”。

3. **Anti-swamp test**
   当源码细节继续扩展到 Windows sandbox、MCP、TUI UI 时，Agent 必须判断是否服务当前 demo。
   预期：不服务则明确跳过。

4. **Stage exit test**
   当 `Core Invariants + Data Structures + State Machine + Acceptance Tests` 足够明确时，Agent 必须建议进入 `07-demo-architecture`，不能继续开放式 code reading。

5. **User orientation test**
   用户任意时刻问“我们在哪里”，Agent 应能用 5 行以内回答：
   当前阶段、最终产物、当前缺口、下一步、完成后解锁什么。

## Assumptions

- repo learning 的最终目标不是“读懂全部源码”，而是形成可迁移能力，并通过 mini demo / business transfer / knowledge export 验证。
- 当前最需要改的是流程协议和模板，不一定需要改 Rust CLI 状态模型。
- 可以新增 `.daedalus/outcome-map.md`，也可以先把同样结构并入 `.daedalus/task-card.md` 或 `.daedalus/todo.md`；推荐新增独立文件，避免 task-card 变得臃肿。
- 短期最有效的落地方式是先改 4 类指令：`repo-learning-coach`、`06-code-reader`、`07-demo-architecture`、`resume/todo` 模板。
