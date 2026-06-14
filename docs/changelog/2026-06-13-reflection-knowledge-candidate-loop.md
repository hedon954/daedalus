# 2026-06-13 Reflection 知识候选循环

## 变更

- 将 `candidate-map.md` 从 10 阶段的事后总结表调整为贯穿 01-09 的滚动候选表。
- `10-reflection` 不再从零生成候选，而是基于已有候选表查漏补缺、去重、降噪、确认状态，并驱动回顾、追问校准和知识归档。
- `reflection/` 只保留 `candidate-map.md` 和 `closeout.md` 两个核心入口，避免收尾阶段被过多文件拖重。
- `daedalus validate` 现在会检查专题是否具备 reflection loop 的关键结构，避免进入收尾阶段时缺少可恢复坐标。
- TUI 在 `10-reflection` 阶段会优先指向 `reflection/candidate-map.md`，并展示 reflection loop 状态。
- 知识候选表的状态收敛为 `候选中`、`总结中`、`已归档`、`已忽略` 四种，避免状态文本发散。
- 当前 Codex `tools-permissions` closeout topic 已补齐新的 `reflection/` 结构，并同步渲染 `state.md`。

## 设计意图

- 回顾继续保持用户主动回顾，不由 Agent 替用户总结。
- 知识候选表可以由 Agent 滚动维护，但必须保持简洁，只作为对话确认入口。
- 知识库仍然只接收用户对话确认后的少数高价值、可迁移、可复习条目。

## 验证

- `cargo test --manifest-path crates/Cargo.toml -p daedalus-cli`
- `daedalus validate workspaces/projects/openai-codex-cli-deep-learning`
- `daedalus topic validate --project-dir workspaces/projects/openai-codex-cli-deep-learning tools-permissions`
