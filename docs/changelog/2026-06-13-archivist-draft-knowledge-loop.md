# 2026-06-13 Archivist Draft Knowledge Loop

## 变更

- 将 `10-archivist` 从“closeout 后再提炼知识”调整为“先生成 draft knowledge candidate map，再驱动 closeout、challenge、selection 和 archive evidence”。
- 新增 topic 模板目录 `guides/10-archivist/`，包含候选地图、closeout prompts、用户筛选和归档证据 4 个入口。
- `daedalus validate` 现在会检查 topic 是否具备 archivist loop 的关键结构，避免进入收尾阶段时缺少可恢复坐标。
- TUI 在 `10-archivist` 阶段会优先指向 `01-knowledge-candidate-map.md`，并在 Review / Knowledge 卡片中展示 archivist loop 状态。
- 当前 Codex `tools-permissions` closeout topic 已补齐新的 10-archivist guide 结构，并同步渲染 `state.md`。

## 设计意图

- closeout 继续保持用户主动回顾，不由 Agent 替用户总结。
- candidate map 可以由 Agent 贪心扫描生成，但它只是 draft，不代表用户已经掌握。
- knowledge-base 仍然只接收用户筛选后的少数高价值、可迁移、可复习条目。

## 验证

- `cargo test --manifest-path crates/Cargo.toml -p daedalus-cli`
- `daedalus validate workspaces/projects/openai-codex-cli-deep-learning`
- `daedalus topic validate --project-dir workspaces/projects/openai-codex-cli-deep-learning tools-permissions`
