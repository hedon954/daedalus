# 06 核心代码阅读指南

## 当前专题

`auth / approval / sandbox`：Codex 如何避免本地 Agent 变成不受控的远程执行器。

本专题不先给结论。先从生产事故出发，让用户形成假设，再进入源码验证。

## 阅读方法

每轮只围绕一个闭环问题推进：

```text
生产问题 -> naive 失败 -> 用户假设 -> 源码验证 -> 不变量 -> trade-off -> 可迁移模式
```

不要按文件清单读源码，也不要只解释调用链。重点回答：

- 这段代码防止什么生产事故？
- 如果没有这层设计，最直接的实现会在哪里失败？
- 它保护了什么不变量？
- 它让系统付出了什么复杂度成本？
- 哪部分值得迁移到自己的 Agent/CLI，哪部分不必照抄？

## 本轮思考问题

### 1. 三种审批结果分别在防什么？

先想三个场景：

- `Forbidden`：什么情况下，即使用户想点确认，系统也不应该继续？
- `NeedsApproval`：什么情况下，系统不是绝对禁止，但必须让用户知道并确认？
- `Skip`：什么情况下，系统可以不问用户，但仍然不代表完全没有约束？

提示：重点区分两件事：**不问用户** 和 **不进沙箱** 不是一回事。

### 2. 为什么 sandbox-first？

对比两个方案：

方案 A：模型要执行命令，先问用户；用户同意后，直接裸跑。

方案 B：模型要执行命令，先尽量在 sandbox 里跑；只有被 sandbox 拦住，再问用户要不要提升权限。

思考角度：

- 哪个方案更符合最小权限原则？
- 哪个方案更容易减少用户被频繁打扰？
- 哪个方案更能区分“命令本身错了”和“权限不够”？
- 如果 sandbox 失败后自动裸跑，会出什么事故？

### 3. `dispatch_any` 的 gate 在防什么？

先不要想审批，先想工具分发本身会出哪些工程事故：

- 模型调用了一个根本不存在的工具，怎么办？
- 模型传了错误类型的 payload，怎么办？
- 企业或用户想在工具执行前加规则拦截，应该插在哪里？
- 两个会修改文件的工具并发执行，可能破坏什么？
- 工具执行后，如果需要记录审计、追加上下文、或者停止任务，应该在哪里处理？

## 建议回答格式

```text
1. Forbidden / NeedsApproval / Skip 分别是...
2. sandbox-first 的原因是...
3. pre_tool_use_hooks / is_mutating / tool_call_gate 分别是...
```

## 回答后的处理

用户回答后，Agent 必须先更新 `notes/06-code-reader/README.md` 或同目录专题文件，记录：

- 问题。
- 用户原始回答。
- Agent 校准或补充。
- 下一步源码验证路径。
- 当前验证状态。

随后再进入源码验证，优先阅读：

- `core/src/tools/sandboxing.rs`
- `core/src/tools/orchestrator.rs`
- `core/src/tools/registry.rs`

## Runtime Request Assembly 阅读检查表

当前缺口：

> shell / unified exec 到底如何组装 `CommandRequest` 的上下文字段。

从第一性原理看，本地命令执行不是“运行一个字符串”，而是把一个有副作用的意图转换成一个可判断、可审批、可隔离、可审计的执行请求。

读源码时先抓住 7 个问题。

### 1. 命令本体是什么

- 原始输入是字符串，还是 argv？
- 有没有 shell wrapper，例如 `bash -lc ...`？
- 什么时候需要解析成 command segments？
- 解析失败或复杂解析时，会不会影响 amendment / allow 复用？

### 2. 它在哪执行

- `cwd` 从哪里来？
- 是用户当前 workspace、工具请求指定目录，还是 fallback cwd？
- 如果命令访问 cwd 外的文件，权限语义是否变化？

### 3. 它带着什么权限意图

- 这次请求是默认 sandbox，还是显式 require escalated？
- 是否请求网络、文件写入、额外路径权限？
- 这些权限是用户显式给的，还是工具/runtime 推导出来的？

### 4. 全局审批策略是什么

- 当前 `AskForApproval` 是 `never / on-request / on-failure / unless-trusted / granular` 哪一种？
- 这个策略影响的是“是否问用户”，还是“是否允许裸跑”？
- `granular` 下，要区分 rule approval 和 sandbox approval。

### 5. approval key 绑定什么

- 用户点一次允许，系统到底记住了什么？
- 是否绑定 command、cwd、sandbox permissions、additional permissions？
- 这决定 demo 里授权缓存的粒度。

### 6. request 和 execution attempt 是否分层

- `CommandRequest` 描述“要做什么、带什么上下文”。
- sandbox attempt 描述“第一次怎么跑”。
- retry / escalation 描述“失败后是否换权限再跑”。
- 不要把这三层混成一个布尔值。

### 7. 这个 request 是否可审计

- 如果之后用户问“为什么这条命令需要审批”，request 里有没有足够信息解释？
- 如果出事故，能否还原 command、cwd、权限、审批策略、sandbox 选择？

## 字段追踪表

读 shell / unified exec 时，用这个表标注每个字段的来源：

| 字段 | 来源 | 如果缺失会怎样 |
| --- | --- | --- |
| `command` | tool call / shell runtime | 无法判断真实副作用 |
| `cwd` | session / tool request | 无法判断文件边界 |
| `approval_policy` | config / session | 不知道能不能问用户 |
| `sandbox_policy` | permission profile / config | 不知道默认隔离边界 |
| `sandbox_permissions` | tool request / runtime | 不知道是否请求提升 |
| `additional_permissions` | network / filesystem extras | 授权粒度过粗 |
| `prefix_rule` | user approval suggestion | 无法生成可复用授权 |
| `approval_key` | normalized request context | 缓存可能越权 |

本轮只回答一个核心问题：

> shell / unified exec 组装出来的 `CommandRequest`，哪些字段来自用户请求，哪些字段来自全局 config，哪些字段来自 runtime 自己的判断？
