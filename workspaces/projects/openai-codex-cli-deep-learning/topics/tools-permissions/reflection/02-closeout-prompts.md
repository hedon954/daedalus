# 收尾回顾提示

> 状态：草稿。你已经写过 closeout，本文件现在用于复核：哪些候选确实进入了你的理解，哪些还只是做过一次。

## 主线确认

- 你现在如何用一句话解释这个 topic 的主线？
- “模型调用 shell”和“Host 受控执行本地命令”之间的本质区别是什么？
- 你画的三张图分别解决什么表达问题：工具权限、ReAct loop、CLI 分层？

## Demo 决策

- 你最认可 demo 中哪三个设计决策？为什么？
- 哪些地方是忠实模仿 Codex，哪些地方是学习 demo 的刻意简化？
- `ToolRuntime`、`ApprovalGateway`、`ExecutionRunner`、`EventEmitter` 的边界是否足够清楚？

## 取舍与边界

- 当前 demo 能证明什么？不能证明什么？
- 如果放到生产环境，哪些简化必须被替换？
- 会话级 approval 的边界未来在哪里可能裂开？

## 旁路知识

- Tokio runtime / Waker / OS event queue 是否值得单独开 mini-tokio topic？
- sandbox-exec、shell quoting、process invocation 哪些点需要进入知识库？
- Ratatui CLI 的事件循环和 Agent ReAct loop 有哪些相似与差异？

## 候选筛选

- `01-knowledge-candidate-map.md` 里哪些候选应该归档？
- 哪些候选只需要进入 review/backlog？
- 哪些候选其实不重要，应该删掉，避免污染 knowledge-base？
