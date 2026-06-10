# 04 运行调试指南

## 目标

先跑通最小、可观察的 Codex CLI 路径，再进入架构分析和核心代码精读。第一条建议路径是 `codex exec`，因为它比 TUI 更容易记录命令、日志和事件输出。

## 环境依赖

- 操作系统：macOS 12+、Ubuntu 20.04+/Debian 10+，或 Windows 11 WSL2。
- Rust：需要可用的 Rust toolchain，建议包含 `rustfmt` 和 `clippy`。
- 工作目录：`source/codex/codex-rs`。
- 认证：真实模型调用可能需要 ChatGPT 登录或 API key；本阶段先允许用 `--help`、构建和测试验证入口，不强制真实请求模型。

## 推荐最小命令

在任务根目录下：

```bash
cd source/codex/codex-rs
cargo run --bin codex -- --help
cargo run --bin codex -- exec --help
cargo test -p codex-exec
```

如果要尝试真实非交互请求：

```bash
cd source/codex/codex-rs
RUST_LOG=codex_exec=debug,codex_core=debug cargo run --bin codex -- exec "用一句话说明当前仓库是什么"
```

## 预期观察

- `cargo run --bin codex -- --help` 应展示 multitool 命令，包括 `exec`、`login`、`mcp`、`mcp-server`、`sandbox`、`debug`、`resume` 等。
- `cargo run --bin codex -- exec --help` 应展示 headless/non-interactive 运行参数。
- `cargo test -p codex-exec` 用于在不依赖真实模型调用的情况下验证 `exec` crate 的局部行为。
- 真实 `codex exec` 可能因未登录、网络、模型权限或配置缺失失败；失败信息本身也要记录，因为它能暴露认证和配置边界。

## 核心入口

- Multitool 入口：`codex-rs/cli/src/main.rs`
- 非交互入口：`codex-rs/exec/src/main.rs`
- 非交互主逻辑：`codex-rs/exec/src/lib.rs`
- 交互式 TUI 入口：`codex-rs/tui/src/main.rs`
- 核心 thread/session：`codex-rs/core/src/thread_manager.rs`、`codex-rs/core/src/codex_thread.rs`、`codex-rs/core/src/session.rs`

## 断点建议

先追 `codex exec`：

1. `codex-rs/cli/src/main.rs`：确认 `Subcommand::Exec` 如何转到 `codex_exec`。
2. `codex-rs/exec/src/main.rs`：确认 root CLI overrides 如何合并到 exec CLI。
3. `codex-rs/exec/src/lib.rs` 的 `run_main`：观察配置、认证、初始 prompt、json/event output 的准备。
4. `codex-rs/core/src/thread_manager.rs`：观察 `ThreadManager` 如何创建 thread，并持有 `SkillsManager`、`PluginsManager`、`McpManager`、model provider。
5. `codex-rs/core/src/codex_thread.rs`：观察 `submit` / `steer_input` 如何把用户输入交给 session loop。

## Cursor 交互式断点调试

### 前置条件

- Cursor 安装 Rust Analyzer。
- Cursor 安装 CodeLLDB 扩展，调试配置里的 `type` 使用 `lldb`。
- 因为交互式 Codex 是 Ratatui TUI，调试时必须使用 terminal 运行，配置里要设置 `"terminal": "integrated"` 或 `"terminal": "external"`。
- 工作目录必须指向 `source/codex/codex-rs`，否则 Cargo workspace、配置和相对路径容易错位。

### 推荐窗口与 launch 配置

推荐单独用 Cursor 打开 `source/codex/codex-rs`，并把 `.vscode/launch.json` 放在 `codex-rs` 根目录。这样 `${workspaceFolder}` 就是当前学习 repo 的 Rust workspace 根目录，后续学习其他 repo 时也能保持“一个 repo 一个窗口”的聚焦方式。

本任务已经写入：

```text
source/codex/codex-rs/.vscode/launch.json
```

配置内容如下：

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Debug Codex TUI interactive",
      "type": "lldb",
      "request": "launch",
      "cargo": {
        "args": ["build", "--bin", "codex"],
        "filter": {
          "name": "codex",
          "kind": "bin"
        }
      },
      "args": [
        "-c",
        "log_dir=${workspaceFolder}/.codex-log",
        "--sandbox",
        "read-only"
      ],
      "cwd": "${workspaceFolder}",
      "env": {
        "RUST_LOG": "codex_core=debug,codex_tui=debug,codex_app_server_client=debug"
      },
      "terminal": "integrated"
    },
    {
      "name": "Debug Codex exec help",
      "type": "lldb",
      "request": "launch",
      "cargo": {
        "args": ["build", "--bin", "codex"],
        "filter": {
          "name": "codex",
          "kind": "bin"
        }
      },
      "args": ["exec", "--help"],
      "cwd": "${workspaceFolder}",
      "env": {
        "RUST_LOG": "codex_exec=debug,codex_core=debug"
      },
      "terminal": "integrated"
    }
  ]
}
```

### 断点顺序

先用 `Debug Codex TUI interactive`：

1. `codex-rs/cli/src/main.rs` 的 `match subcommand`：验证无 subcommand 时进入交互式 TUI。
2. `codex-rs/cli/src/main.rs` 的 `run_interactive_tui`：验证 CLI 参数如何转给 `codex_tui::run_main`。
3. `codex-rs/tui/src/lib.rs` 的 `run_main`：观察 TUI 如何加载配置、认证、app-server client 和 runtime。
4. `codex-rs/core/src/thread_manager.rs` 的 `ThreadManager::new`：观察 skills、plugins、MCP、model manager 等运行时组件何时进入 thread manager。
5. `codex-rs/core/src/codex_thread.rs` 的 `submit` / `steer_input`：观察用户输入如何进入 session。

再用 `Debug Codex exec help` 或把 args 改成 `["exec", "你的测试 prompt"]`：

1. `codex-rs/cli/src/main.rs` 的 `Some(Subcommand::Exec(...))`。
2. `codex-rs/exec/src/main.rs` 的 `run_main(inner, arg0_paths)`。
3. `codex-rs/exec/src/lib.rs` 的 `run_main`。

### 常见问题

- 如果 TUI 画面在 Debug Console 里乱码，说明没有用 terminal 启动；改用 integrated/external terminal。
- 如果出现路径重复，例如 `codex-rs/workspaces/02-learning/.../codex-rs`，说明当前窗口已经打开在 `codex-rs`，但配置仍按 daedalus 根目录拼路径；应使用本节这种 `cwd: "${workspaceFolder}"` 的配置。
- 如果启动后停在登录或认证流程，这是预期现象；先用断点验证配置和入口链路，不必立刻完成真实模型调用。
- 如果断点没有命中，先确认正在 debug 的 binary 是 `codex`，不是 `codex-tui` 或 `codex-exec`。
- 如果只想验证交互式入口，不要传 `exec` subcommand；不带 subcommand 才会走 TUI。

## 要验证的假设

- 用户猜测的 “coding agent 构建层” 很可能对应 `ThreadManager` 和 session 创建路径。
- “LLM client adapter” 需要沿 `codex_model_provider::create_model_provider` 和 model manager 继续验证。
- “tools/MCP/skills/plugins 扩展体系” 在 `ThreadManagerState` 中已经出现，但它们是否属于同一抽象层仍需后续架构分析。
- “权限、审核、沙箱” 需要沿 `AskForApproval`、`PermissionProfile`、`SandboxPolicy` 和 `codex-sandboxing` 继续验证。
- “hooks 收尾” 暂未在第一条入口里验证，后续单独查 `hooks` crate 和配置触发点。

## 用户记录要求

运行每条命令后，在 `notes/04-debugger-guide/README.md` 记录：

- 实际命令。
- 是否成功。
- 关键输出或错误。
- 你对调用链的新理解。
- 与 `notes/03-socratic-coach/README.md` 中假设相符或冲突的地方。
