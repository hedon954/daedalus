# 2026-06-11 知识库 Skill-First 演进

## 变更

- 删除 CLI 中的知识萃取、晋升、导出式内容生成能力。
- `daedalus knowledge` 只保留确定性底座：`template`、`index`、`list`、`link-check`、`validate`。
- 新增 `knowledge-base/` 八类知识条目结构：概念、技能、模式、问题、案例、来源映射、技能树、练习。
- 新增知识条目模板与 `knowledge-base/index.toml` 机器索引。
- 新增 `.claude/skills/daedalus-knowledge-*`，把萃取、检查、缺口分析、晋升、重组放到 skill 工作流。
- 新增 `agents/skills/` 软链接入口，方便其他 coding agent 复用同一套 skill。
- 将 Codex tools-permissions demo 的安全命令执行经验沉淀为首批知识库样本。

## 边界

- CLI 不负责替用户生成知识结论。
- 需要解释、判断、批判、归纳、重写的工作交给 skill。
- 需要结构稳定、索引、链接检查和最低质量门槛的工作交给 CLI。

## 验证

- `cargo test --manifest-path crates/Cargo.toml -p daedalus-cli`
- `daedalus knowledge index`
- `daedalus knowledge validate`
- `daedalus knowledge link-check`
- `daedalus validate workspaces/projects/openai-codex-cli-deep-learning --all-topics --reviews --knowledge`
