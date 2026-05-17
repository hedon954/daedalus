# Guides / Notes 文件夹化改造计划

## Summary

把 `guides/` 和 `notes/` 从“阶段单文件”升级为“阶段文件夹 + README 入口 + 专题文件”的结构。目标是让阶段内部也可导航：README 负责路径、状态和索引，专题文件承载具体问题、源码证据、用户回答和 demo 决策。

## Key Changes

- 新增约定：阶段产物目录使用 `guides/<stage-id>/README.md` 与 `notes/<stage-id>/README.md` 作为稳定入口；专题文件按问题命名，例如 `runtime-request-assembly.md`、`decision-composition.md`。
- 更新模板与 prompt：所有新任务和后续阶段引导，不再默认写入 `guides/06-code-reader-guide.md` 或 `notes/code-reading.md` 这类万能文件，而是写入阶段目录；README 只做导航和验收索引。
- 更新 `state.toml` 模板：阶段 `required_artifacts` 指向 README 入口，例如 `notes/06-code-reader/README.md`，CLI 仍只检查路径存在，不引入复杂内容校验。
- 当前 Codex 学习任务采用增量迁移：保留已有历史文件不删除，新增 `guides/06-code-reader/` 与 `notes/06-code-reader/` 作为后续入口，并在 README 中索引旧文件与新专题，避免大规模链接震荡。
- 更新 `artifact-index.md`、`todo.md`、`outcome-map.md` 相关链接，使当前学习路径指向阶段目录入口。

## Implementation Details

- 在 repo 学习规则中明确三层结构：
  - `outcome-map.md`：全程终点地图。
  - `notes/<stage-id>/README.md`：阶段已验证结论索引。
  - `notes/<stage-id>/<topic>.md`：专题证据与学习轨迹。
- 修改阶段 prompt 的写入规则：
  - Agent-only 预读写入 `guides/<stage-id>/<topic>.md`。
  - 用户回答、校准、源码证据写入 `notes/<stage-id>/<topic>.md`。
  - 每次新增专题后同步更新阶段 README。
- 当前任务先迁移 active 阶段：
  - `guides/06-code-reader/README.md`
  - `guides/06-code-reader/runtime-request-assembly.md`
  - `notes/06-code-reader/README.md`
  - 后续 Decision 合成、Runtime request assembly、Orchestrator retry 都写入该目录。
- 旧文件保留为 legacy artifact，并由 README 链接，不在本次做全文拆分或删除。

## Test Plan

- 运行 `cargo test -p daedalus-cli`，确保初始化、校验、状态流转测试仍通过。
- 新建一个临时 repo-learning task，确认模板生成后包含新约定，`daedalus validate` 能正确识别 required README artifact。
- 校验当前 Codex workspace：`daedalus validate workspaces/02-learning/openai-codex-cli-deep-learning`。
- 手动检查 resume 场景：用户问“继续学习”时，应能从 `outcome-map.md + notes/06-code-reader/README.md + todo.md` 恢复当前位置。

## Assumptions

- 本次只改变学习产物组织方式，不改 Rust 状态机枚举。
- `required_artifacts` 继续表示“路径存在”，不校验 README 内部链接完整性。
- 历史单文件不强制拆分，避免破坏现有引用；新内容按文件夹结构写入。
