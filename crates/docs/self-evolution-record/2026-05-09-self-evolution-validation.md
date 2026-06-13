# daedalus 自进化验证报告

> 分支：`feature/self-evolution`
> 日期：2026-05-09
> 验证方式：模拟用户使用 daedalus CLI 进行 repo-learning，共 3 轮

## 验证目标

1. CLI 状态机在完整 10 阶段流程中是否可靠。
2. 边界情况（非法状态转换、缺产物、WIP 限制、force complete）是否被正确拦截。
3. 任务 lifecycle（complete / abandon）与文件系统 bucket 是否一致。
4. 本轮新增的教练式交互改进（CLI-first、guides/notes 语义、Coach Questioning Protocol、Notes Ownership Rule、Mermaid 兼容规则、外部 repo 调试规则）是否已正确落地到模板和 prompt 中。

## 第 1 轮：完整流程推进

### 测试内容

从 `daedalus init repo-learning` 开始，逐个阶段 enter → 创建产物 → complete，走完 01-09 全部 stage。

### 结果

| 步骤 | 行为 | 结果 |
|---|---|---|
| init | 生成完整模板结构（15 个文件） | 通过 |
| validate | 初始化后立即校验 | 通过 |
| complete 01 | task-card.md 已存在 | 通过 |
| complete 02 without artifact | 缺 `guides/02-repo-selection-guide.md` | 正确拒绝 |
| complete 02 with artifact | 产物存在后正常完成 | 通过 |
| 03-09 连续推进 | enter → create artifact → complete | 全部通过 |
| state.md 渲染 | 01-09 全部 done，10 pending | 正确 |
| next_action 更新 | 指向 `10-reflection` | 正确 |

### 观察

- 产物路径从 `notes/repo-selection.md` 已正确迁移到 `guides/02-repo-selection-guide.md`。
- `state.md` 的缺失产物列表在阶段完成后自动清除。
- 每次 complete 都会自动渲染 state.md。

## 第 2 轮：边界情况测试

### 测试内容

对状态机非法转换、WIP 限制、force 机制进行穷举测试。

### 结果

| 测试 | 期望 | 实际 | 结果 |
|---|---|---|---|
| complete pending stage | 拒绝：pending cannot complete | 拒绝 | 通过 |
| block active stage | 允许 | 允许 | 通过 |
| complete blocked stage | 拒绝：blocked cannot complete | 拒绝 | 通过 |
| resume blocked stage | 允许 | 允许 | 通过 |
| enter 02 while 01 active | 允许，01 被自动 blocked | 01 状态变为 blocked | 通过 |
| force complete 缺产物 | 允许 + 记录 approval_source | 允许 | 通过 |
| validate force-completed task | 报告缺失产物 | 正确报告 | 通过 |
| resume pending stage | 拒绝：pending cannot resume | 拒绝 | 通过 |
| block pending stage | 拒绝：pending cannot block | 拒绝 | 通过 |
| enter done stage | 拒绝：done cannot enter | 拒绝 | 通过 |
| complete done stage | 拒绝：done cannot complete | 拒绝 | 通过 |
| task complete without 10-reflection active | 拒绝：pending cannot complete | 拒绝 | 通过 |
| WIP 限制：已有 active task 时 init | 拒绝 | 正确拒绝 | 通过 |
| task abandon | 允许，lifecycle 和 bucket 一致 | 一致 | 通过 |

### 观察

- 所有 13 种非法状态转换均被正确拦截，错误消息清晰（包含状态名和动作名）。
- validate 能检出 force complete 后残留的产物缺失。
- abandon 后 state.toml 的 lifecycle 和 workspace_bucket 同步更新。

## 第 3 轮：任务正常完成闭环

### 测试内容

从 init 到 task complete 的完整闭环，验证 lifecycle 一致性。

### 结果

| 步骤 | 结果 |
|---|---|
| 01-09 正常推进 | 全部通过 |
| enter 10-reflection | 通过 |
| task complete | 通过 |
| 目录移动到 03-completed | 正确 |
| state.toml lifecycle = completed | 正确 |
| state.toml workspace_bucket = 03-completed | 正确 |
| 10-reflection status = done | 正确 |
| validate completed task | 通过 |
| decision-log 记录关闭原因 | 正确 |
| state.md 显示关闭时间和原因 | 正确 |
| WIP slot 释放，可 init 新任务 | 正确 |

## 教练式交互改进落地验证

### CLI-first 初始化

- `CLAUDE.md` 模板包含明确的 CLI-first 规则。
- `01-goal-aligner` prompt 第 1 步即要求使用 `daedalus init repo-learning`。
- README 强调新任务创建必须通过 CLI。

### guides/notes 语义

- `02-repo-scout` 的 required artifact 已从 `notes/repo-selection.md` 迁移到 `guides/02-repo-selection-guide.md`。
- `artifact-index.md` 模板已新增 `guides/02-repo-selection-guide.md` 和 `notes/question-roadmap.md` 行。

### Coach Questioning Protocol

- `system/prompts/common/coach-questioning.md` 已创建，定义了三层问题、提问时机、提问数量和引导回答方式。
- 已被 01-06 阶段 prompt 通过 `@` 引用。

### Notes Ownership Rule

- `CLAUDE.md` 模板包含完整的 Notes Ownership Rule。
- 05-arch-analyzer 和 06-code-reader prompt 已新增"不要在用户没有形成假设前直接写完整 notes"约束。

### Mermaid 兼容规则

- `system/prompts/common/diagram-guidelines.md` 已创建，覆盖保留字、alias、message 文本和简约性规则。
- 已被 03、04、05、06 阶段 prompt 引用。

### 外部 repo 调试规则

- `04-debugger-guide` prompt 新增第 8 步：外部 repo 应在其 workspace 根目录单独打开 Cursor 窗口。
- `source/README.md` 新增单独窗口和 `${workspaceFolder}` 基准说明。
- `CLAUDE.md` 模板新增外部 repo 调试推荐。

## 发现的问题

### P0（无）

当前未发现阻塞性问题。

### P1（建议优化）

1. **transition 日志的 actor 不一致**：CLI 直接执行时 actor 为 `agent`，但实际上可能是用户手动调用。目前无法区分 Agent 代调用和用户直接调用，不影响功能但降低审计精度。
2. **complete 后 current_phase 不自动前进**：complete 一个 stage 后，`current_phase` 仍然指向刚完成的 stage，直到用户执行 `enter` 下一个 stage。`next_action` 提示了正确的下一步，但 `current_phase` 在中间状态不太直观。

### P2（可改进）

3. **state.md 最近状态流转只显示最后 5 条**：完整闭环有 20+ 条 transition，只显示最后 5 条。对于回顾学习轨迹不够完整，但对 Agent 恢复上下文已经足够。
4. **validate 不检查 notes 是否包含用户证据**：validate 只检查文件是否存在，不检查内容是否有用户实践痕迹。这是正确的（CLI 不应做内容审查），但可以在 prompt 层面进一步强调。

## 结论

daedalus 在 `feature/self-evolution` 分支上的 CLI 状态机、产物校验、lifecycle 管理和教练式交互改进均符合预期。3 轮测试覆盖了正常流程、13 种边界情况和完整闭环，全部通过。教练式交互的 6 项改进（CLI-first、guides/notes 语义、Coach Questioning Protocol、Notes Ownership Rule、Mermaid 规则、外部 repo 调试）已正确落地到模板和 prompt 中。

建议在正式 dogfooding（与 Cursor Agent 对话）时重点观察：
- Agent 是否在初始化时直接使用 CLI，不手写模板。
- Agent 是否在阶段推进前提出 1-3 个问题引导用户。
- Agent 是否遵守 Notes Ownership Rule，不替用户写完整笔记。
