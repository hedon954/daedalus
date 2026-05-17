# Runtime Request Assembly 阅读检查表

## Learning Navigation

- Final artifact: [`demo/design.md`](../../demo/design.md)
- Current stage: `06-code-reader`
- Current gap: Runtime request assembly
- Evidence needed: shell / unified exec 如何把一次工具调用组装成可审批、可沙箱执行的 request。
- After this: 能定稿 demo 的 `CommandRequest` 字段。

## First Principles

本地命令执行不是“运行一个字符串”，而是把一个有副作用的意图转换成一个可判断、可审批、可隔离、可审计的执行请求。

## Seven Questions

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

## Field Tracking Table

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

## User Question

> shell / unified exec 组装出来的 `CommandRequest`，哪些字段来自用户请求，哪些字段来自全局 config，哪些字段来自 runtime 自己的判断？
