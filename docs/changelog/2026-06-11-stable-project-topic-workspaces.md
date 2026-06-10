# 稳定 Project / Topic 工作区落地

> 日期：2026-06-11
> 对应计划：`docs/plan/06-project-topic-lifecycle-stable-workspaces.md`
> 状态：已完成

## 变更摘要

- 将正式学习项目统一放入 `workspaces/projects/`，项目不再因为 topic 完成或重启而移动目录。
- 新增 workspace 级状态投影：
  - `workspaces/.daedalus/current.toml`
  - `workspaces/.daedalus/project-index.toml`
  - `workspaces/current-project`
  - `workspaces/current-topic`
- 将全局 backlog 移到 `workspaces/backlog/`。
- 项目生命周期改为 `active / idle / abandoned`；完成所有 topic 后项目进入 `idle`，不再进入旧 `03-completed` bucket。
- 废弃 topic 移入项目内部 `.archive/`，不污染默认学习视图。
- 新增根目录 AI / 搜索 ignore 文件，默认排除 `workspaces/projects/**/.archive/`。
- 新增 `.claude/settings.json` 的 `permissions.deny`，让 Claude Code 默认不读写 `.archive/`。
- 更新 CLI、TUI、模板、README、AGENTS/CLAUDE 和 repo-learning prompt，使它们使用稳定 project/topic 工作区。

## 当前 Codex 示例

- 当前项目：`workspaces/projects/openai-codex-cli-deep-learning`
- 当前 project 状态：`idle`
- 当前 topic：`tools-permissions`
- 当前 topic 状态：`completed`
- `current-project` 保留，方便用户点击进入项目。
- `current-topic` 已移除，因为当前没有 active topic。

## 验证

已通过：

```bash
cargo test --manifest-path crates/Cargo.toml -p daedalus-cli
cargo run --manifest-path crates/Cargo.toml -p daedalus-cli --bin daedalus -- validate workspaces/projects/openai-codex-cli-deep-learning --all-topics --reviews --knowledge
git diff --check
```

## 设计边界

- 旧 bucket 字符串仅作为迁移或校验输入保留，不再作为默认目录结构出现。
- `workspaces/current-project` 和 `workspaces/current-topic` 是人类入口；机器真相仍是 `workspaces/.daedalus/current.toml`。
- `.archive/` 仍可进入 Git，但默认不进入 AI 上下文和搜索视图。
