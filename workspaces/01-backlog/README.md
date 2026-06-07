# Backlog Inbox

这里记录尚未进入 active learning 的未来学习候选方向。

## Role

Backlog 是 pre-learning candidate inbox，不是学习现场。它用来保存“未来可能值得学”的方向，避免把学习愿望丢在聊天里，也避免打断当前 WIP。

## Rules

- 一个 backlog candidate 使用一个独立 Markdown 文件，例如 `mini-tokio-runtime.md`。
- 默认使用模板：[`../../system/templates/backlog/item.md`](../../system/templates/backlog/item.md)。
- backlog 文件只记录候选方向，不承载 active project 或 topic 的阶段进度。
- backlog 不创建 `.daedalus/state.toml`，不写 `notes/`、`guides/`、`demo/`。
- 准备启动时，先用 gatekeeper 判断是新 project、当前 project 的新 topic，还是继续暂缓。
- 只有和当前 repo/source 直接相关的后续方向，才放进该 project 的 `topic-board.md`。

## States

- `captured`：刚记录，目标和产物可能还粗糙。
- `shaped`：目标、预期产物、启动前置条件已经初步清楚。
- `ready`：通过 gatekeeper，等待进入 active learning。
- `promoted`：已经创建 active project 或 topic。
- `deferred`：值得学，但当前 WIP、前置知识或时间不允许。
- `rejected`：不再保留为候选方向。
