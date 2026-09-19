# Source

这里用于放置当前学习任务的外部源码材料。

默认规则：

- 外部源码目录不会进入 daedalus 仓库版本控制。
- 优先由用户执行 [`pull_source.sh`](pull_source.sh) 拉取源码，并记录自己的运行结果。
- Agent 可以生成或调整拉取脚本、解释错误、给出排障建议，但不要默认代替用户完成源码准备。
- 如果用户明确要求 Agent 代为拉取，完成后应删除外部仓库自带的 `.git` 目录，避免嵌套仓库污染当前知识库。
- 实际运行和调试外部 repo 时，推荐在该 repo 根目录单独打开 Cursor 窗口；`.vscode/launch.json`、断点路径和启动命令都应以被学习 repo 的 `${workspaceFolder}` 为基准。
- daedalus 任务目录只保存拉取脚本、学习指南、用户笔记和复盘，不假设自己就是外部 repo 的运行 workspace。
