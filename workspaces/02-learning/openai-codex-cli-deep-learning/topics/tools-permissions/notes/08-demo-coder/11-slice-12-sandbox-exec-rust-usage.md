# Slice 12 Sandbox Exec Rust Usage

## Learning Navigation

- Final artifact: [`../../demo/README.md`](../../demo/README.md)
- Current stage: `08-demo-coder`
- Current slice: Slice 12 Phase 2A OsExecutionRunner
- Current gap: 用 macOS `sandbox-exec` 把模拟执行器升级成真实 OS sandbox 执行器。
- Source anchor: [`../../guides/08-demo-coder/14-slice-12-os-execution-runner.md`](../../guides/08-demo-coder/14-slice-12-os-execution-runner.md)

## 这次交互解决了什么

这次不是继续设计权限模型，而是补齐 Phase 2A 的一个具体实现缺口：

```text
CommandRequest / ExecutionAttempt
  -> OsExecutionRunner
  -> tokio::process::Command
  -> /usr/bin/sandbox-exec
  -> ExecutionResult
```

关键收获有两个：

1. Rust 调系统命令时，必须清楚区分“单个 argv”和“一组 argv”。
2. `sandbox-exec` profile 是 macOS sandbox 的规则 DSL，需要由 demo 自己的 `SandboxProfile` 翻译过去。

## `arg` 和 `args` 的区别

遇到的问题：

```text
the trait bound `[std::string::String]: AsRef<OsStr>` is not satisfied
```

触发代码形态：

```rust
command.arg(&request.argv[1..]);
```

原因是 `Command::arg` 只接收一个参数，要求它能转成 `OsStr`。但 `&request.argv[1..]` 是一组参数，类型是 `&[String]`，不是单个 argv。

正确写法：

```rust
command.arg(program);
command.args(&request.argv[1..]);
```

判断规则：

```text
arg(x)   = 追加一个 argv
args(xs) = 追加一组 argv
```

这个点很重要，因为我们不应该默认把 `raw_command` 塞进 `sh -c`。Phase 2A 的安全边界依赖结构化 argv：`program` 是真实 executable，`argv[1..]` 是它的参数。

## `sandbox-exec` profile 是什么

`sandbox_exec_profile` 生成的不是 Rust 语法，也不是 shell 语法，而是 macOS `sandbox-exec` 的 profile 语言。它是 Scheme/Lisp 风格 DSL，用来描述进程允许或拒绝哪些系统能力。

示例：

```scheme
(version 1)
(deny default)
(allow process*)
(allow file-read*)
```

含义：

```text
使用 profile 版本 1
默认拒绝所有能力
允许进程相关操作
允许文件读取
```

常见规则形态：

```scheme
(allow file-write* (subpath "/some/workspace"))
```

表示允许写入某个目录及其子路径。

所以 `sandbox_exec_profile` 的本质不是“执行 sandbox”，而是一个翻译器：

```text
Rust enum SandboxProfile
  -> macOS sandbox profile string
  -> sandbox-exec -p <profile> <program> <args...>
```

## 当前实现时要守住的边界

Phase 2A 先实现单命令真实 sandbox 执行，不急着解决完整 shell 语法。

- 保留结构化 `argv`，不要默认降级成 `sh -c <raw_command>`。
- `sandbox-exec` profile 字符串拼接先可用，但要留下转义 TODO，尤其是路径插入。
- sandbox denied 的识别可以先用 stderr 启发式分类，但测试不应该断言完整错误文案。
- 真实安全边界来自 OS sandbox policy，不来自 stderr 字符串匹配。

## Demo 决策

`OsExecutionRunner` 的第一版应按 `ExecutionAttempt` 分流：

```text
SandboxFirst
  -> /usr/bin/sandbox-exec -p <profile> <program> <args...>

NoSandboxFirst / NoSandboxRetry
  -> <program> <args...>
```

`ExecutionResult` 的分类可以先保持简化：

- `status.success()` -> `Success`
- spawn 失败 -> `CommandFailed`
- stderr 看起来像 sandbox 拒绝 -> `SandboxDenied`
- 其他非零退出 -> `CommandFailed`

这里的 trade-off 是：实现足够贴近 Codex 的“先 sandbox，再按策略决定是否重试”主干，但暂时不追求复刻 Codex 对不同平台 sandbox 的完整兼容层。
