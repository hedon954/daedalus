# Repo Learning 多专题项目化改造报告

日期：2026-05-23

## 背景

原 repo-learning workspace 把一次学习任务、一个专题和一套 10-stage 进度混在同一层目录中。这个结构适合单主题深读，但不适合长期学习同一个 repo 的多个方向，例如先学 Codex 工具/权限系统，再学 sub-agent 调度、prompt engineering 或 context engineering。

本次改造把 repo learning 从 single-topic task 升级为：

- Learning Project：长期学习同一个 repo 或素材的 project。
- Learning Topic：project 下的一个独立专题，每个 topic 自己走完整 10-stage。
- Shared Context：跨 topic 复用的源码索引、术语表、运行手册、证据和迁移模式。

## 关键变更

### 1. Workspace 结构

新结构：

```text
workspaces/02-learning/<project>/
  .daedalus/
    state.toml
    state.md
    project-map.md
    topic-board.md
  shared/
    README.md
    source-index.md
    runbook.md
    architecture-map.md
    glossary.md
    evidence-registry.md
    transfer-patterns.md
  source/
  topics/
    <topic>/
      .daedalus/
      guides/
      notes/
      demo/
```

Project state 只描述 lifecycle、active topic 和 topic 列表；10-stage 学习进度移动到 topic state。

### 2. CLI 能力

新增：

- `daedalus init repo-learning <project> --topic <slug> --title <title>`
- `daedalus topic new/list/activate/complete/abandon/validate`
- `system/bin/audit-daedalus-agent-instructions`

调整：

- `state enter/complete/block/resume/rollback` 默认作用于 active topic。
- `validate` 默认校验 project + active topic，`--all-topics` 校验所有 topics。
- `task complete` 只关闭 project；必须先关闭所有 unfinished topics。
- TUI 最小适配 project + active topic，进度和 todo 来自 active topic。

### 3. Templates / Prompts / Skill

新增 `system/templates/repo-topic/`，project 模板和 topic 模板分离。

更新 repo-learning coach、resume、gatekeeper 和 01-10 阶段 prompt：

- 所有 `guides/`、`notes/`、`demo/` 默认属于 active topic。
- Project root 只维护 project map、topic board 和 shared context。
- 恢复学习时先定位 project、active topic、topic stage、current gap。
- 关闭时先关闭 topic，再关闭 project。

新增指令审计脚本，扫描旧 CLI 和旧路径误导。

### 4. 当前 Codex Workspace 迁移

已迁移：

```text
workspaces/02-learning/openai-codex-cli-deep-learning
```

当前 active topic：

```text
topics/tools-permissions
```

迁移后保留原学习进度：topic 仍处于 `08-demo-coder`，下一步仍是继续 mini demo 实现。

## 重要修正

- 修复 `state_toml` 对 `[topic]` 的硬索引 panic；project state 没有 `[topic]` 是正常情况。
- 一次性升级过程不覆盖已有 `source/README.md`、`source/pull_source.sh`、`source/.gitignore`。
- 一次性升级过程会把 project `next_action` 指向 active topic 的真实进度。
- `validate_topic_workspace` 改为校验 `demo/`、`guides/`、`notes/` 目录存在，不再强制 `.gitkeep` 存在。
- 一次性升级备份目录已加入 `.gitignore`，避免备份污染提交。

## 验证

已通过：

```bash
RUSTC_WRAPPER= CARGO_TARGET_DIR=/private/tmp/daedalus-target cargo fmt --manifest-path crates/Cargo.toml --check
RUSTC_WRAPPER= CARGO_TARGET_DIR=/private/tmp/daedalus-target cargo test --manifest-path crates/Cargo.toml -p daedalus-cli
system/bin/audit-daedalus-agent-instructions .
RUSTC_WRAPPER= CARGO_TARGET_DIR=/private/tmp/daedalus-target cargo run --manifest-path crates/Cargo.toml -p daedalus-cli --bin daedalus -- validate workspaces/02-learning/openai-codex-cli-deep-learning --all-topics
```

测试覆盖：

- repo-learning init 生成 project + initial topic。
- stage transition 操作 active topic。
- topic new / activate / list。
- topic 未关闭时 project complete 被拒绝。
- project complete 在 topic 关闭后释放 WIP。
- legacy single-topic workspace 已完成到 multi-topic project 的一次性整理。

## 后续建议

- TUI 目前是最小适配，后续可增加 project/topic 切换视图。
- 旧的一次性升级入口已从长期 CLI 中移除；后续结构调整直接手动整理具体文件。
- 后续新增专题时，优先沉淀 shared/source-index、shared/glossary 和 shared/evidence-registry，减少重复读源码。
