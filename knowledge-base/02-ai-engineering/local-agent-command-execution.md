# Local Agent Command Execution Safety Pattern

## 业务压力

当 Agent 具备本地命令执行能力时，它会从“文本生成器”变成能读写文件、访问网络、安装依赖和启动进程的本地自动化主体。单纯的命令字符串黑白名单无法表达 cwd、文件权限、网络权限、审批策略、沙箱边界和 retry 语义。

## 核心做法

把本地命令执行拆成六个独立层次：

```text
model intent
  -> host capability
  -> approval requirement
  -> sandboxed execution attempt
  -> retry / escalation decision
  -> observable tool result
```

模型只能提出 tool call；host runtime 负责把命令映射到 capability，并根据 request context 生成 approval / sandbox / retry 决策。

## 保护的不变量

- 模型意图不等于执行权限。
- `Allow` 不等于跳过 sandbox。
- 用户审批必须绑定 command prefix、cwd、sandbox profile、network policy 和 persistence。
- sandbox denied 后不能自动裸跑，必须经过 retry gate。
- 未匹配 capability 的命令 fail closed。
- tool / command / approval / retry events 必须对外可观察。

## 代价 / Trade-off

- 结构更复杂，需要维护 capability registry、approval scope、execution runner 和 retry policy。
- 用户可能需要处理更多审批交互。
- prefix matching 易解释，但对复杂 shell 语法不够强。
- session approval 改善体验，但扩大一次点击的影响范围。

## 局限 / Failure Mode

- 用 `split_whitespace` 解析复杂 shell 会误判 pipe、quote、heredoc、subshell。
- simulated sandbox 只能验证状态机，不能证明 OS 隔离。
- no-sandbox retry 在高安全业务里可能不可接受。
- 如果 approval store 跨用户或跨 workspace 复用，会造成授权泄漏。

## 忠实模仿边界

值得模仿 Codex 的核心机制：

- capability / approval / sandbox / retry 分层。
- sandbox-first execution。
- run-scoped approval event。
- session approval 由 CLI session 生命周期承载。
- 工具结果作为 observation 回灌给 Agent。

不必照搬的部分：

- 完整跨平台 sandbox 实现。
- 所有 Codex 内部 event 类型。
- 完整 TUI/MCP elicitation。
- 长期持久化 exec policy 文件。

## 适用边界

适用于本地 coding agent、CLI agent、开发自动化助手、需要运行测试/读取文件/安装依赖的交互式工具。

不适用于无交互高安全自动化环境，除非把 `ApprovalPolicy` 固定为 `Never` 并彻底禁用 no-sandbox retry。

## 迁移步骤

1. 定义最小 capability registry，只开放必要命令。
2. 定义 `CommandRequest`，把 cwd、sandbox、network、approval policy 放进上下文。
3. 定义 `ApprovalScope`，让一次允许绑定执行权限画像。
4. 把 runner 抽成 `ExecutionRunner`，先用 simulated runner 测状态机，再接 OS sandbox。
5. 把 sandbox denied 交给 retry gate，而不是在 runner 内直接提权。
6. 把安全节点转成稳定事件，供 UI、日志和测试消费。

## Evidence

- Topic: `workspaces/02-learning/openai-codex-cli-deep-learning/topics/tools-permissions`
- Demo: `demo/README.md`
- Business transfer: `notes/09-biz-solver/README.md`
- Closeout note: `notes/08-demo-coder/13-slice-13-codex-like-cli-closeout.md`
- Verification: `cargo test --manifest-path workspaces/02-learning/openai-codex-cli-deep-learning/topics/tools-permissions/demo/Cargo.toml -j 2` -> `90 passed; 0 failed; 3 ignored`
