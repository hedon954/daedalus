# Topic discovery 选题探索层

> 日期：2026-06-14
> 状态：已完成

## 背景

daedalus 原有 `clarify-goal` 和 `gatekeeper` 假设用户已经能说清现实问题、预期产物和学习目标。但真实选题场景里，用户往往只有模糊冲动、职业焦虑、能力短板、候选材料吸引力和互相冲突的方向。

因此，选题不能只做目标匹配或 accept / defer / reject。真实诉求需要先被探索出来。

## 变更

- 新增 `system/prompts/common/topic-discovery.md`，在任务卡和 gatekeeper 前增加选题探索层。
- 新增 `system/templates/discovery/item.md` 和 `workspaces/discovery/README.md`，用于保存 pre-topic discovery 记录。
- CLI 稳定 workspace layout 现在会创建 `workspaces/discovery`。
- `clarify-goal` 在诉求仍靠 Agent 猜测时会先要求进入 discovery。
- `gatekeeper` 不再替代 discovery；真实诉求不清楚时应 defer 并要求先探索。
- repo learning `01-goal-aligner` 在目标不清时先继承 discovery，不直接推荐 repo 或初始化 project/topic。
- 全局契约、repo-learning skill、README 和模板索引补充了 discovery / backlog / active topic 的语义分层。
- `.codex/plans/` 的规则收紧为只保留带 `status` 和 `todos` 的 Codex tracking card；详细正式计划放在 `docs/plan/`。

## 验证

- `cargo test --manifest-path crates/Cargo.toml -p daedalus-cli`
- `daedalus validate workspaces/projects/openai-codex-cli-deep-learning --all-topics --reviews --knowledge`
