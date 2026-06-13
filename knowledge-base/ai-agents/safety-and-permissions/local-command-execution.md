---
status = "verified"
source = "workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions"
---

# Agent 本地命令执行安全

Agent 执行本地命令的核心问题不是“怎么调用 shell”，而是“怎么让一个不可信的模型意图变成受控副作用”。

在 Codex `tools-permissions` topic 里，我最终把链路理解成：

```text
tool call
-> capability match
-> approval requirement
-> sandbox first
-> retry / escalation
-> observation / event
```

这条链路对应的不是某个项目的代码风格，而是一组安全边界。

## 从第一性原理看

本地命令之所以危险，是因为它不是普通函数调用。函数调用通常只影响进程内状态；本地命令会继承宿主环境，触碰文件系统、网络、进程、环境变量、当前工作目录和用户权限。

所以第一性原理是：

```text
模型只能表达意图
Host 必须拥有解释、授权、隔离、执行和审计的最终控制权
```

OpenAI 的 function calling 文档也把工具调用描述成多步流程：模型产生 tool call，应用侧执行代码，再把 tool output 发回模型，而不是模型自己执行工具。这一点和 Codex demo 的边界一致。

Rust 的 `std::process::Command` 也能说明这件事的底层形态：执行命令本质上是在构造并启动一个子进程。这个子进程带着 program、args、cwd、env、stdin/stdout/stderr 等执行上下文。权限系统如果只看 command string，就会漏掉真正的副作用边界。

## 底层原理

这条安全链路可以拆成四个层级：

1. `capability`：这个请求属于什么能力。
   例如纯函数工具、本地命令、安全读、危险 shell、未知工具。匹配不到能力时应该 fail closed。

2. `approval`：用户或策略是否允许这件事发生。
   审批回答的是“可不可以做”。它应该绑定 scope，例如命令范围、cwd、network、sandbox profile、session persistence。

3. `sandbox`：即使允许做，也要决定以什么边界做。
   审批通过不等于裸跑。`Skip approval` 也不等于 `bypass sandbox`。

4. `retry`：沙箱失败后是否允许提权重试。
   不能把所有失败都当成权限不足。命令自身错误应该返回失败；只有 sandbox denied 这类边界问题，才进入 retry policy 和再次审批。

`raw_command` 和 `argv` 的区别也在这里变得关键。`Command::new("echo").arg("hi")` 和 `sh -c "echo hi > file"` 的风险边界不同：后者把解析权交给 shell，会引入重定向、管道、变量展开、heredoc、多命令等语义。权限系统要么能解析这些语义，要么保守处理。

## 在 Codex 学习中的体现

Codex 源码给我的关键启发是：权限判断不是一个字符串白名单，而是要把命令和上下文一起判断。demo 中对应到：

- [`tool/runtime.rs`](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/tool/runtime.rs)：先把 tool call 规划成纯函数或 command path。
- [`tool/shell/`](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/tool/shell/)：把 `run_command` 转成 command request。
- [`tool/shell/approval.rs`](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/tool/shell/approval.rs)：决定 skip / needs approval / forbidden。
- [`tool/shell/retry.rs`](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/tool/shell/retry.rs)：区分命令错误、sandbox denied、policy 是否允许重试。
- [`tool/shell/execution/`](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/tool/shell/execution/)：把 sandbox 和 host execution 放在同一个 execution attempt 抽象下。

closeout 中确认的不变量是：

- 模型只产生意图，Host 才能做权限判断。
- capability 匹配不到不能默认放行。
- approval 和 sandbox 是不同层。
- sandbox denied 后不能自动裸跑。
- 用户授权必须绑定 scope。
- 审批、执行、重试和结果必须对外可观测。

## 现实工程取舍

Codex 的命令级权限模型适合 local coding agent，但不必照搬到所有 Agent。

如果 Agent 的工具是业务 action，权限 scope 可能应该绑定：

- 用户。
- workspace。
- 业务资源。
- action 类型。
- 数据敏感等级。
- 审批持久化策略。

真正可迁移的是安全分层：

```text
intent -> capability -> approval -> isolation -> retry -> observation
```

而不是“所有系统都要用 shell prefix 做权限规则”。

## 关联

- Topic closeout: [Codex Tools-Permissions Closeout](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/reflection/closeout.md)
- Candidate map: [知识候选表](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/reflection/candidate-map.md)
- 外部资料：
  - [OpenAI Function Calling](https://developers.openai.com/api/docs/guides/function-calling)
  - [Rust `std::process::Command`](https://doc.rust-lang.org/std/process/struct.Command.html)
