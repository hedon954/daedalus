# 知识候选地图草稿

> 状态：草稿 / 待用户确认。这里不是知识库正文，只是 daedalus 为你准备的回顾地图。候选能不能归档，必须由你在收尾回顾和候选筛选中确认。

## 已扫描证据

- [x] `.daedalus/outcome-map.md`
- [x] `.daedalus/todo.md`
- [x] `guides/`
- [x] `notes/`
- [x] `demo/`
- [x] demo 测试和运行证据
- [x] `reflection/closeout.md`
- [ ] 归档前重新检查外部资料

## 主题核心

### 本地命令执行是一条由宿主控制的安全链路

- 类型：主题核心
- 价值：本专题的主线不是“模型调用 shell”，而是宿主程序在 `command -> capability -> approval -> sandbox -> retry -> observation` 中保持控制权。
- 证据：`demo/src/tool/runtime.rs`、`demo/src/tool/shell/`、`demo/src/execution/`、`reflection/closeout.md`
- 状态：新增
- 掌握度：需要用户确认
- 建议动作：归档

### 审批和沙箱是两个不同层次

- 类型：主题核心
- 价值：审批决定“是否允许做”，沙箱决定“以什么边界做”；二者不能混为一个布尔开关。
- 证据：Codex exec policy 阅读笔记、`demo/src/approval.rs`、`demo/src/retry.rs`、`demo/src/execution/`
- 状态：新增
- 掌握度：需要用户确认
- 建议动作：归档

### 重试是策略判断，不是失败后盲目再跑

- 类型：主题核心
- 价值：sandbox denied 后是否重试，取决于失败类型、审批策略、重试策略、审批结果和执行尝试。
- 证据：`demo/src/retry.rs`、Slice 6/7 guides、retry policy tests
- 状态：新增
- 掌握度：需要用户确认
- 建议动作：归档

### 工具观察事件要足够表达安全决策

- 类型：主题核心
- 价值：外界需要看到审批、执行尝试、沙箱/本地、重试、工具结果，才能判断工具不是被直接裸跑。
- 证据：`demo/src/event.rs`、`demo/src/event_emitter.rs`、`demo/src/tool/runtime.rs`、`demo/src/agent/react.rs`
- 状态：新增
- 掌握度：需要用户确认
- 建议动作：归档

## 旁路技能

### Rust 异步流和通道编排

- 类型：旁路技能
- 价值：ReAct loop、LLM streaming、tool event、approval response 都依赖 async task、channel、stream 边界。
- 证据：`demo/src/agent/react.rs`、`demo/src/agent/llm/openai.rs`、Tokio guides
- 状态：新增
- 掌握度：需要用户确认
- 建议动作：复习

### Ratatui CLI 本质上是状态与渲染之上的事件循环

- 类型：旁路技能
- 价值：CLI 第二阶段不只是画 UI，而是把终端输入、Agent 事件流、审批 UI、状态更新、绘制函数分层。
- 证据：`demo/src/cli/`
- 状态：新增
- 掌握度：需要用户确认
- 建议动作：归档或延后

### OpenAI 兼容流式接口需要按协议解析

- 类型：旁路技能
- 价值：SSE 的 `data:` 行、增量合并、工具调用参数聚合和最终消息回灌，是 Agent 接真实模型的基础能力。
- 证据：`demo/src/agent/llm/openai.rs`、OpenAI integration guides
- 状态：新增
- 掌握度：需要用户确认
- 建议动作：复习

## 基础薄弱点

### Tokio runtime、Waker 和操作系统事件多路复用

- 类型：基础薄弱点
- 价值：学习中多次追问 Tokio 并发、spawn 生命周期、waker 与 epoll/kqueue/IOCP 的关系，说明这是可迁移但仍需专门补强的底层主题。
- 证据：Tokio guides、runtime/tool 并发讨论
- 状态：暂无可关联
- 掌握度：需要用户确认
- 建议动作：延后到 mini-tokio backlog

### 操作系统沙箱和本地安全隔离

- 类型：基础薄弱点
- 价值：从模拟沙箱走到 `sandbox-exec` 时，暴露了 OS 级隔离、profile 语言、argv/shell quoting、跨平台差异等基础问题。
- 证据：`demo/src/execution/os_execution_runner.rs`、sandbox-exec guides
- 状态：暂无可关联
- 掌握度：需要用户确认
- 建议动作：归档或复习

### Shell 解析和进程调用边界

- 类型：基础薄弱点
- 价值：`raw_command`、`argv`、`sh -c`、heredoc、多命令和 prefix matching 是权限系统最容易误判的边界。
- 证据：Codex exec policy notes、shell runner discussion、OS runner tests
- 状态：新增
- 掌握度：需要用户确认
- 建议动作：归档

## 工程模式

### 工具运行时先规划再执行

- 类型：工程模式
- 价值：`ToolRuntime` 先将工具调用规划成纯函数路径或本地命令路径，再进入执行层，降低 ReAct 层和权限层耦合。
- 证据：`demo/src/tool/runtime.rs`、closeout 架构图
- 状态：新增
- 掌握度：需要用户确认
- 建议动作：归档

### 事件发射器集中管理生命周期事件

- 类型：工程模式
- 价值：安全事件散落在各处会漏发；集中包装 execution attempt 能让生命周期更可审计。
- 证据：`demo/src/event_emitter.rs`、事件出口复盘笔记
- 状态：新增
- 掌握度：需要用户确认
- 建议动作：归档

### 测试应该验证稳定行为，而不是偶然文案

- 类型：工程模式
- 价值：学习中多次删除脆弱字符串断言，形成了 daedalus 全局测试规则。
- 证据：`AGENTS.md`、common prompts、CLI/demo test refactors
- 状态：补充已有
- 掌握度：已吸收验证
- 建议动作：如果知识库还没有覆盖，则归档

## 外部对比

- Codex / Claude Code 风格的 session approval：会话级 allowlist 应绑定 capability scope，而不是永久裸放。
- macOS `sandbox-exec`、Linux namespaces、container、语言虚拟机沙箱：不同方案的第一性原理边界不同，需要在知识库中区分。
- OpenAI 兼容流式接口和 DeepSeek 兼容 API：接口兼容不等于事件语义完全相同，Host 应以协议解析结果为准。

## 学习方法

### 架构图应该表达分层和方向，而不是每一条边

- 类型：学习方法
- 价值：收尾回顾画图时发现，过度追求完整会牺牲可读性；图负责分层和方向，文字负责解释决策点、例外和取舍。
- 证据：closeout diagram discussion
- 状态：新增
- 掌握度：已吸收验证
- 建议动作：归档

### Closeout 聚焦核心，知识候选挖掘要尽量充分

- 类型：学习方法
- 价值：收尾回顾要帮助用户完成主动回顾和理解验证；知识候选地图要尽量挖掘所有可迁移能力，再由用户精选。
- 证据：daedalus 改进讨论、本次 Reflection 方案
- 状态：新增
- 掌握度：已吸收验证
- 建议动作：归档
