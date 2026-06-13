# 2026-06-13 Reflection 知识候选循环

## 变更

- 将 `10-reflection` 从“回顾后再提炼知识”调整为“先生成知识候选地图草稿，再驱动回顾、追问校准、筛选和归档证据”。
- 新增专题模板目录 `reflection/`，包含候选地图、回顾提示、用户筛选和归档证据 4 个入口。
- `daedalus validate` 现在会检查专题是否具备 reflection loop 的关键结构，避免进入收尾阶段时缺少可恢复坐标。
- TUI 在 `10-reflection` 阶段会优先指向 `reflection/01-knowledge-candidate-map.md`，并展示 reflection loop 状态。
- 当前 Codex `tools-permissions` closeout topic 已补齐新的 `reflection/` 结构，并同步渲染 `state.md`。

## 设计意图

- 回顾继续保持用户主动回顾，不由 Agent 替用户总结。
- 知识候选地图可以由 Agent 贪心扫描生成，但它只是草稿，不代表用户已经掌握。
- 知识库仍然只接收用户筛选后的少数高价值、可迁移、可复习条目。

## 验证

- `cargo test --manifest-path crates/Cargo.toml -p daedalus-cli`
- `daedalus validate workspaces/projects/openai-codex-cli-deep-learning`
- `daedalus topic validate --project-dir workspaces/projects/openai-codex-cli-deep-learning tools-permissions`
