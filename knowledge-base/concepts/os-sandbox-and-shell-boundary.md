---
kind = "concept"
slug = "os-sandbox-and-shell-boundary"
status = "verified"
source = "workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/guides/08-demo-coder/15-sandbox-first-principles.md"
created_at = "2026-06-13"
---

# OS Sandbox 与 Shell 执行边界

## 回忆钩子

Sandbox 限制的是进程能触碰的资源；shell 解析决定了到底启动什么进程、用什么参数、做什么重定向。

## 现实问题

Agent 执行本地命令时，危险不只在命令名。`sh -c`、管道、重定向、heredoc、多命令、环境变量和 `cwd` 都会改变真实执行行为。即使 sandbox 存在，如果构造命令和 profile 的边界不清楚，也会出现误判。

## 第一性原理

隔离的本质是减少被执行代码可触达的资源集合。shell 的本质是把字符串解释成进程图和 I/O 重定向。安全执行必须同时控制“解释出来的是什么”和“解释后能访问什么”。

## 底层原理

- macOS `sandbox-exec` 使用 `-f`、`-n` 或 `-p` 指定 profile，再在该 profile 下执行命令和参数。
- macOS App Sandbox 通过 entitlements 限制 app 对文件系统、网络等系统资源的访问，目标是 containment，而不是通用命令行 runner。
- Linux namespace / container 更偏向隔离进程看到的系统视图；cgroups 更偏向限制和统计资源使用。
- 语言 VM sandbox 限制的是语言运行时内的能力，不等于 OS 级隔离。
- `argv` 调用和 `sh -c` 不同：前者直接给程序参数，后者先交给 shell 解释。
- quoting 错误会让 `sh -c "echo hi > file"` 变成错误的 argv，从而执行失败或绕过预期。

## 关键不变量

- `raw_command` 用于展示和策略分析，`argv` 用于实际进程调用。
- 复杂 shell 语法不能轻易生成宽泛 allow rule。
- sandbox denied 和 command failed 要分开处理。
- sandbox profile 是执行边界，不是审批结论。

## 取舍

真实 OS sandbox 更接近可用 demo，但跨平台差异大。模拟 runner 易测试，但不能证明真实隔离能力。生产系统要根据目标平台选择 sandbox、container、VM、权限 ACL 或事务边界。

## 不要照搬

不要把 macOS `sandbox-exec` 当作跨平台方案。它适合作为 macOS 本地 demo 的真实隔离体验，但生产级 coding agent 需要重新评估平台、兼容性和维护成本。

## 迁移方式

做本地命令执行时，先决定：

```text
命令如何解析 -> 进程如何启动 -> 资源如何隔离 -> 失败如何归因
```

## 证据来源

- [sandbox-first-principles.md](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/guides/08-demo-coder/15-sandbox-first-principles.md)
- [os_execution_runner.rs](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/tool/shell/execution/os_execution_runner.rs)
- [slice-12-sandbox-exec-rust-usage.md](../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/notes/08-demo-coder/11-slice-12-sandbox-exec-rust-usage.md)
- [sandbox-exec man page](https://www.unix.com/man_page/osx/1/sandbox-exec/)
- [Apple App Sandbox documentation](https://developer.apple.com/documentation/xcode/configuring-the-macos-app-sandbox)
- [Linux namespaces man page](https://man7.org/linux/man-pages/man7/namespaces.7.html)

## 复习练习

解释为什么 `argv = ["sh", "-c", "echo hi > check.log"]` 和 `argv = ["sh", "-c", "\"echo hi > check.log\""]` 结果不同。再说明 sandbox denied 为什么不能当作普通 command failed。
