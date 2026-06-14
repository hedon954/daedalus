# Assert / Result / Denied 的边界判断

这份笔记沉淀一个通用工程判断：什么时候用 `assert`，什么时候用 `Result`，什么时候在安全边界上返回 `Forbidden` / `Denied`。

它来自 Codex mini demo 中 `decide_approval` 的设计讨论，但适用于更广泛的系统设计。

## 核心判断

`assert` 不是用来处理“可能发生的错误”，而是用来声明调用者已经承诺满足的内部不变量。

一个实用分界：

```text
assert / debug_assert：我方代码刚刚保证过的不变量；失败说明调用契约被破坏。
Result：调用者、配置、用户输入、外部环境可以合理造成的失败。
Forbidden / Denied：安全边界上不能继续执行的输入或内部不一致。
```

## 回到 decide_approval

例子：

```text
request.capability != matched_capability.capability.kind
```

在当前 demo 的正常链路中，这基本是程序员错误或上游组装 bug：

```text
argv -> registry.match_capability(argv) -> matched_capability
matched_capability.capability.kind -> CommandRequest.capability
decide_approval(request, matched_capability)
```

如果链路是单线程、同步、内存内完成的，用户输入再奇怪，也只应该导致：

```text
匹配不到 capability
匹配到 DangerousShell
需要审批
被禁止
```

不应该导致：

```text
request.capability = NetworkInstall
matched.capability.kind = SafeRead
```

所以 mismatch 本质上是不变量被破坏。

## 为什么仍然不直接 panic

`decide_approval` 位于命令授权边界。它回答的是：

```text
这条命令是否允许进入执行路径？
```

安全边界遇到未知或矛盾时，最稳妥的行为不是继续，也不是崩溃，而是 fail closed：

```text
不允许执行。
返回可审计、可观测的拒绝结果。
让事件流和 agent loop 能正常表达失败。
```

因此即使 mismatch 本质上是程序员错误，运行时也应该收敛成 `Forbidden`。

## 推荐折中

```rust
debug_assert_eq!(
    request.capability,
    matched_capability.capability.kind,
    "request capability must match matched capability"
);

if request.capability != matched_capability.capability.kind {
    return ApprovalRequirement::Forbidden {
        reason: format!(
            "request capability {:?} does not match matched capability {:?}",
            request.capability,
            matched_capability.capability.kind
        ),
    };
}
```

这样同时保留两种价值：

```text
开发期：debug_assert 尽早暴露程序员错误。
运行时：Forbidden 把内部不一致收敛成可审计、可观测、不可执行的安全结果。
```

## 何时适合 assert

适合：

```text
纯内部算法的不变量。
状态机不可能状态。
结构性约束已经由构造器或类型系统保证。
测试/开发期定位 bug，但 release 不依赖它保证安全。
```

不适合：

```text
用户输入。
配置错误。
外部环境失败。
权限、安全、执行边界。
跨模块 public API 的常规失败路径。
```

## 一句话原则

```text
越内部、越算法、越由当前模块刚刚保证，越适合 assert。
越外部、越运行时、越靠近权限/执行边界，越应该 Result 或 fail closed。
```
