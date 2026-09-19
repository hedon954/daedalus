# Discovery Inbox

这里保存尚未进入 active learning 的选题探索记录。

## Role

Discovery 是 pre-topic exploration space。它记录“用户为什么被某些方向吸引、真实诉求假设是什么、哪些问题仍需澄清”，而不是记录已经决定要学的 backlog candidate 或 active topic 进度。

## Rules

- 一个 discovery 使用一个独立 Markdown 文件，例如 `ai-stack-vs-ddia.md`。
- 默认使用模板：[`../../system/templates/discovery/item.md`](../../system/templates/discovery/item.md)。
- discovery 文件不创建 `.daedalus/state.toml`。
- discovery 文件不写 `notes/`、`guides/`、`demo/` 或 active `todo.md`。
- 当诉求假设能导向可验证产物后，再进入 `clarify-goal` 和 `gatekeeper`。
- 如果只是想保存未来可能学习的方向，而不是探索真实诉求，使用 [`../backlog`](../backlog)。

## States

- `exploring`：还在澄清真实诉求。
- `shaped`：已经形成可进入任务卡的诉求假设。
- `promoted`：已经进入 backlog、project 或 topic。
- `parked`：暂时搁置，不占 active WIP。
