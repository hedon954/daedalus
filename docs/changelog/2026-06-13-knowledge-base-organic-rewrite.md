# 知识库有机重写

> 日期：2026-06-13
> 状态：已完成

## 背景

上一版 knowledge-base 把内容拆进 `concepts`、`patterns`、`problems`、`skills` 等固定类型桶，并强制套模板。结果是知识被拆碎，读起来像表格填空，难以唤起完整理解。

## 变更

- 删除固定知识模板，只保留 `daedalus knowledge template <path>` 创建最小空白笔记骨架。
- 知识库改成可演化的层级知识树，例如 `ai-agents/`、`computer-systems/`、`rust/`。
- 重写 Codex `tools-permissions` 的知识归档，合并为 5 篇自然笔记：
  - Agent 本地命令执行安全
  - ReAct 工具运行时
  - Sandbox 的第一性原理
  - Rust 流式 Agent 与 Tokio 运行时
  - 终端 Agent UI 的事件循环
- `knowledge index` 改为记录目录层级 `level`，不再记录旧 `kind`。
- `knowledge validate` 会检查 `index.toml` 是否陈旧，避免目录迁移后机器索引仍指向旧文件。
- 更新 repo-learning 和 common prompts，要求知识归档按候选内容选择知识树位置、查外部资料校准、从第一性原理和底层原理组织正文。

## 验证

- `cargo test --manifest-path crates/Cargo.toml -p daedalus-cli`
- `daedalus knowledge validate`
- `daedalus knowledge link-check`
- `daedalus validate workspaces/projects/openai-codex-cli-deep-learning --all-topics --reviews --knowledge`
