---
status = "verified"
source = "workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions"
---

# Sandbox 的第一性原理

Sandbox 不是一个具体工具，而是一类隔离思想：在不完全信任被执行代码的前提下，限制它能看到什么、能访问什么、能消耗什么、能向哪里产生副作用。

## 从第一性原理看

执行程序时，进程默认继承宿主机的一部分世界：

- 文件系统视图。
- 用户和组权限。
- 网络能力。
- 环境变量。
- 当前工作目录。
- 子进程能力。
- 可见进程和 IPC。

Sandbox 的第一性原理就是缩小这个世界：

```text
不是问“这个命令会不会作恶”
而是让它即使作恶，也只能在被允许的边界内行动
```

这和 approval 是不同层级。approval 是决策；sandbox 是执行边界。

## 几类 sandbox 的底层差异

### macOS `sandbox-exec`

`sandbox-exec` 可以用 profile 执行命令，限制文件、网络等能力。它适合命令行 runner 这类“临时把一个进程包进限制环境”的场景。

它的问题也明显：profile 语言不是稳定公开的一等 API，跨平台不可用，错误信息和权限模型都需要额外封装。

### macOS App Sandbox

Apple App Sandbox 是面向 app 的长期隔离模型。它通过 entitlement 声明 app 需要哪些能力，目标是限制 compromised app 对系统资源和用户数据的伤害。

它适合签名 app，不是 `run this shell command under this temporary policy` 的直接替代品。

### Linux namespaces

Linux namespaces 把全局系统资源分割成不同视图。不同 namespace 中的进程可以看到不同的 PID、mount、network、user、IPC 等资源。它解决的是“进程看到的系统世界是什么”。

### cgroups / containers

namespace 主要隔离视图，cgroups 主要限制和统计资源。Docker 这类 container runtime 通常组合 namespaces、cgroups、文件系统层、capabilities、seccomp 等机制，形成更完整的隔离环境。

### 语言 VM sandbox

语言级 sandbox 限制的是语言 runtime 内能力，例如禁用某些 API、限制模块导入或解释器能力。它通常挡不住 native escape、进程级权限和系统调用层面的副作用，因此不能替代 OS sandbox。

## 在 Codex demo 中的体现

demo 先实现了 `SimulatedSandboxRunner`，再进入 `OsExecutionRunner`。这个顺序是合理的：

- simulated runner 用来验证状态机、approval、retry、event。
- OS runner 用来暴露真实进程调用、profile、cwd、argv、shell quoting、stderr 分类等问题。

相关实现：

- [`tool/shell/execution/os_execution_runner.rs`](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/tool/shell/execution/os_execution_runner.rs)
- [`tool/shell/execution/`](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/demo/src/tool/shell/execution/)
- [sandbox-exec Rust 使用笔记](../../../workspaces/projects/openai-codex-cli-deep-learning/topics/tools-permissions/notes/08-demo-coder/11-slice-12-sandbox-exec-rust-usage.md)

这次学习的关键结论：

```text
demo 可以先模拟 sandbox
但真正理解 sandbox，必须至少接一次 OS 级隔离
```

因为只有真实 OS runner 才会暴露 argv、shell、cwd、profile、权限错误分类这些工程边界。

## 现实工程取舍

选择 sandbox 方案时，不要先问“哪个更安全”，而要问：

- 隔离对象是 shell command、app、container、脚本，还是业务 action？
- 要限制的是文件、网络、进程、资源、系统调用，还是语言 API？
- 是否需要跨平台？
- 用户授权和 sandbox policy 如何组合？
- 出错后能否给用户清晰解释？
- 是否需要可观测事件证明没有裸跑？

对 local coding agent 来说，一个合理路径是：

```text
capability policy
-> sandbox first execution
-> denied classification
-> approval-gated no-sandbox retry
-> audit event
```

## 关联

- [Agent 本地命令执行安全](../../ai-agents/safety-and-permissions/local-command-execution.md)
- 外部资料：
  - [`sandbox-exec` man page](https://www.unix.com/man_page/osx/1/sandbox-exec/)
  - [Apple App Sandbox](https://developer.apple.com/documentation/security/app-sandbox)
  - [Configuring the macOS App Sandbox](https://developer.apple.com/documentation/xcode/configuring-the-macos-app-sandbox)
  - [Linux namespaces](https://man7.org/linux/man-pages/man7/namespaces.7.html)
  - [Docker user namespace isolation](https://docs.docker.com/engine/security/userns-remap/)
