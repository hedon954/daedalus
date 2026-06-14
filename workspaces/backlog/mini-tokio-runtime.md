# Mini Tokio Runtime

## Capture

- state: captured
- captured_at: 2026-06-07
- source_kind: self-built-demo / repo
- relation_to_current_work: independent

## Why

- 为什么想学：当前 Codex demo 的 Slice 9 已触及 tool call 并发执行、`join_all`、task 调度、`Waker` 和事件回流；如果只停在 Tokio API 使用层，对 Agent runtime / tool concurrency 的理解还不够扎实。
- 它服务的长期能力：理解 async runtime 如何推进 future、调度 task、保存和触发 waker、处理 timer / channel / OS readiness 等等待来源，并能把这些能力迁移到自己的 Agent / CLI runtime 设计中。

## Possible Outcome

- 最终产物：一套 Tokio 底层实现原理笔记、一个教学版 mini runtime demo、可迁移到 Agent runtime 的知识条目。
- 最小 demo：实现一个能 `spawn` 任务、维护 ready queue、通过自定义 `Waker` 重新调度 task、支持简单 timer 或 timer simulation 的 mini runtime。
- 验收方式：能用测试证明 future 返回 `Pending` 后不会阻塞线程，wake 后会重新进入 ready queue，并最终被 executor 再次 poll 到完成。

## First-Principles Questions

- `async fn` 为什么不是线程？它编译后到底交给谁推进？
- `Future::poll` 为什么必须返回 `Pending` 并安排 wake，而不是阻塞等待？
- `Waker` 里面最小需要保存什么，才能把某个 task 放回 ready queue？
- executor、scheduler、reactor、timer driver 的边界分别是什么？
- `tokio::spawn` 和 `join_all` 的本质差异是什么？
- 取消一个 future 和取消一个 OS 子进程为什么不是一回事？

## Start Gate

- 启动前需要完成什么：当前 `tools-permissions` topic 至少完成 Slice 9，或明确阶段性暂停当前 Codex demo 主线。
- 是新 project 还是现有 project topic：倾向新 learning project；它不是 Codex project 的直接 topic。
- 当前 WIP 是否允许切换：当前 WIP 仍是 Codex `tools-permissions`，因此现在只捕获为 backlog。

## Notes

- 捕获时的上下文：用户希望后续由 coach 带着学习 Tokio 底层实现原理，并实现一个 mini Tokio。这个愿望来自当前 Slice 9 对 Tokio runtime / `Future::poll` / `Waker` / OS event queue 的讨论，但学习对象独立于 Codex。
