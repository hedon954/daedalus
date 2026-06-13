# Draft Knowledge Candidate Map

> 状态：draft / pending user confirmation。这里不是知识库正文，只是 daedalus 为你准备的回顾地图。候选能不能归档，必须由你在 closeout 和 selection 中确认。

## Evidence Scanned

- [x] `.daedalus/outcome-map.md`
- [x] `.daedalus/todo.md`
- [x] `guides/`
- [x] `notes/`
- [x] `demo/`
- [x] demo tests / run evidence
- [x] `reflection/closeout.md`
- [ ] external references re-check before archival

## Topic Core

### Local command execution is a host-mediated safety pipeline

- type: topic-core
- why: 本 topic 的主线不是“模型调用 shell”，而是 Host 在 command -> capability -> approval -> sandbox -> retry -> observation 中保持控制权。
- source: `demo/src/tool/runtime.rs`, `demo/src/tool/shell/`, `demo/src/execution/`, `reflection/closeout.md`
- status: 新增
- confidence: 需要用户确认
- suggested action: archive

### Approval and sandbox are different layers

- type: topic-core
- why: Approval 决定“是否允许做”，sandbox 决定“以什么边界做”；二者不能混为一个布尔开关。
- source: Codex exec policy 阅读 notes, `demo/src/approval.rs`, `demo/src/retry.rs`, `demo/src/execution/`
- status: 新增
- confidence: 需要用户确认
- suggested action: archive

### Retry is a policy decision, not a blind rerun

- type: topic-core
- why: sandbox denied 后是否重试取决于 failure kind、ApprovalPolicy、RetryPolicy、approval result 和 execution attempt。
- source: `demo/src/retry.rs`, slice 6/7 guides, retry policy tests
- status: 新增
- confidence: 需要用户确认
- suggested action: archive

### Tool observation must be eventful enough to audit safety

- type: topic-core
- why: 外界需要看到 approval、execution attempt、sandbox/local、retry、tool result，才能判断工具不是被直接裸跑。
- source: `demo/src/event.rs`, `demo/src/event_emitter.rs`, `demo/src/tool/runtime.rs`, `demo/src/agent/react.rs`
- status: 新增
- confidence: 需要用户确认
- suggested action: archive

## Satellite Skills

### Rust async stream and channel orchestration

- type: satellite-skill
- why: ReAct loop、LLM streaming、tool event、approval response 都依赖 async task、channel、stream 边界。
- source: `demo/src/agent/react.rs`, `demo/src/agent/llm/openai.rs`, Tokio guides
- status: 新增
- confidence: 需要用户确认
- suggested action: review

### Ratatui CLI is an event loop over state and renderer

- type: satellite-skill
- why: CLI Phase2 不只是画 UI，而是把 terminal input、agent stream、approval UI、state update、draw 分层。
- source: `demo/src/cli/`
- status: 新增
- confidence: 需要用户确认
- suggested action: archive 或 defer

### OpenAI-compatible streaming needs protocol-level parsing

- type: satellite-skill
- why: SSE 的 `data:` 行、delta 合并、tool call accumulation 和 final message 回灌，是 Agent 接真实模型的基础能力。
- source: `demo/src/agent/llm/openai.rs`, OpenAI integration guides
- status: 新增
- confidence: 需要用户确认
- suggested action: review

## Weak Foundations

### Tokio runtime, Waker, and OS event demultiplexing

- type: weak-foundation
- why: 学习中多次追问 Tokio 并发、spawn 生命周期、waker 与 epoll/kqueue/IOCP 的关系，说明这是可迁移但仍需专门补强的底层主题。
- source: Tokio guides, runtime/tool 并发讨论
- status: 暂无可关联
- confidence: 需要用户确认
- suggested action: defer to mini-tokio backlog

### OS sandbox and local security isolation

- type: weak-foundation
- why: 从 simulated sandbox 走到 `sandbox-exec` 时暴露了 OS 级隔离、profile language、argv/shell quoting、跨平台差异等基础问题。
- source: `demo/src/execution/os_execution_runner.rs`, sandbox-exec guides
- status: 暂无可关联
- confidence: 需要用户确认
- suggested action: archive 或 review

### Shell parsing and process invocation boundary

- type: weak-foundation
- why: `raw_command`、`argv`、`sh -c`、heredoc、multi-command 和 prefix matching 是权限系统最容易误判的边界。
- source: Codex exec policy notes, shell runner discussion, OS runner tests
- status: 新增
- confidence: 需要用户确认
- suggested action: archive

## Engineering Patterns

### Plan-then-execute for tool runtime

- type: engineering-pattern
- why: `ToolRuntime` 先将 tool call 规划成 pure function 或 command path，再进入执行层，降低 ReAct 层和权限层耦合。
- source: `demo/src/tool/runtime.rs`, closeout 架构图
- status: 新增
- confidence: 需要用户确认
- suggested action: archive

### EventEmitter centralizes lifecycle emission

- type: engineering-pattern
- why: 安全事件散落在各处会漏发；集中包装 execution attempt 能让生命周期更可审计。
- source: `demo/src/event_emitter.rs`, event outlet review notes
- status: 新增
- confidence: 需要用户确认
- suggested action: archive

### Tests should verify behavior, not incidental strings

- type: engineering-pattern
- why: 学习中多次删除脆弱字符串断言，形成了 daedalus 全局测试规则。
- source: AGENTS.md, common prompts, CLI/demo test refactors
- status: 补充已有
- confidence: 已吸收验证
- suggested action: archive if not already covered

## External Comparisons

- Codex / Claude Code style session approval: session-scoped allowlist 应绑定 capability scope，而不是永久裸放。
- macOS sandbox-exec / Linux namespaces / container / language VM sandbox：不同方案的第一性原理边界不同，需要在知识库中区分。
- OpenAI-compatible streaming 和 DeepSeek-compatible API：接口兼容不等于事件语义完全相同，Host 应以协议解析结果为准。

## Learning Methods

### Diagrams should express layers and direction, not every edge

- type: learning-method
- why: closeout 画图时发现过度追求完整会牺牲可读性；图负责分层和方向，文字负责解释决策点、例外和 trade-off。
- source: closeout diagram discussion
- status: 新增
- confidence: 已吸收验证
- suggested action: archive

### Closeout focuses on core, knowledge extraction mines greedily

- type: learning-method
- why: closeout 要帮助用户完成主动回顾和理解验证；knowledge candidate map 要贪心挖掘所有可迁移能力，再由用户精选。
- source: daedalus improvement discussion, this archivist plan
- status: 新增
- confidence: 已吸收验证
- suggested action: archive
