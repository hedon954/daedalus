# 运行调试记录

## 当前目标

围绕 `codex exec` 建立第一条可复现核心路径：CLI 入口 -> exec 主逻辑 -> thread/session 创建 -> 事件输出。

## Agent 预读结论

- `codex-rs/README.md` 说明 Rust CLI 是当前维护版本。
- `codex-rs/cli/src/main.rs` 是 `codex` multitool 入口，`exec` 是非交互子命令。
- `codex-rs/exec/src/main.rs` 调用 `codex_exec::run_main`。
- `codex-rs/exec/src/lib.rs` 包含 `run_main`，并处理 config、认证、prompt、event processor、app-server protocol。
- `codex-rs/core/src/thread_manager.rs` 的 `ThreadManagerState` 持有 `SkillsManager`、`PluginsManager`、`McpManager`、model manager、thread store、environment manager。
- `codex-rs/core/src/codex_thread.rs` 将 thread 描述为组成 Codex thread 的双向消息流 conduit。

## 用户运行记录

### 命令 1

- 命令：`cd source/codex/codex-rs && cargo run --bin codex -- --help`
- 执行者：用户
- 结果：成功
- 关键输出：`Codex CLI`；`If no subcommand is specified, options will be forwarded to the interactive CLI.`；`Usage: codex [OPTIONS] [PROMPT]` 和 `codex [OPTIONS] <COMMAND> [ARGS]`。
- 新理解：`codex` 是 multitool 入口；没有 subcommand 时进入交互式 CLI，有 subcommand 时进入对应命令。

### 命令 2

- 命令：`cd source/codex/codex-rs && cargo run --bin codex -- exec --help`
- 执行者：用户
- 结果：成功
- 关键输出：`Run Codex non-interactively`；`Usage: codex exec [OPTIONS] [PROMPT]` 和 `codex exec [OPTIONS] <COMMAND> [ARGS]`。
- 新理解：`exec` 是非交互入口，适合作为第一条可复现核心路径。

### 命令 3

- 命令：`cd source/codex/codex-rs && cargo test -p codex-exec`
- 执行者：用户
- 结果：成功
- 关键输出：`test result: ok. 61 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.97s`。
- 新理解：`codex-exec` 的局部行为有较完整测试保护，后续可以把这些测试当作理解非交互路径的辅助入口。

## 待验证链路

1. `codex-rs/cli/src/main.rs` 的 `Subcommand::Exec` 如何进入 `codex_exec`。
2. `codex-rs/exec/src/lib.rs` 的 `run_main` 如何加载配置、认证和 prompt。
3. `ThreadManager` 如何构造 thread，并把 skills、plugins、MCP、model provider、thread store 放入运行时。
4. `CodexThread` 如何把用户输入提交给 session loop。
5. 权限、沙箱、工具调用、hooks 在一次 turn 中分别在哪里发生。

## Cursor 断点调试计划

### 交互式 TUI 路径

- 启动方式：在 Cursor CodeLLDB 中 debug `codex` binary，不传 subcommand。
- 必须使用 terminal：`"terminal": "integrated"` 或 `"terminal": "external"`。
- 首批断点：
  1. `codex-rs/cli/src/main.rs` 的 `match subcommand`。
  2. `codex-rs/cli/src/main.rs` 的 `run_interactive_tui`。
  3. `codex-rs/tui/src/lib.rs` 的 `run_main`。
  4. `codex-rs/core/src/thread_manager.rs` 的 `ThreadManager::new`。
  5. `codex-rs/core/src/codex_thread.rs` 的 `submit` / `steer_input`。

### 非交互 exec 路径

- 启动方式：debug `codex` binary，args 传 `["exec", "--help"]` 或 `["exec", "测试 prompt"]`。
- 首批断点：
  1. `codex-rs/cli/src/main.rs` 的 `Some(Subcommand::Exec(...))`。
  2. `codex-rs/exec/src/main.rs` 的 `codex_exec::run_main` 调用。
  3. `codex-rs/exec/src/lib.rs` 的 `run_main`。

### 待用户验证

- Cursor 是否已安装 CodeLLDB。
- `Debug Codex TUI interactive` 能否在 integrated terminal 中打开 TUI。
- 断点是否能在 `run_interactive_tui` 和 `codex_tui::run_main` 命中。

### 配置修正

- 用户反馈：按上一版 daedalus-root 配置，在单独打开 `codex-rs` 时会得到重复路径：`codex-rs/workspaces/02-learning/.../codex-rs`。
- 修正结论：后续 repo 学习应单独打开源码 repo 或其 Rust workspace 根目录；`launch.json` 应放在当前 repo 根目录，而不是 daedalus 根目录。
- 已写入配置：`source/codex/codex-rs/.vscode/launch.json`。
- 新配置假设：`${workspaceFolder}` 等于 `source/codex/codex-rs`，因此 `cwd` 使用 `${workspaceFolder}`，日志目录使用 `${workspaceFolder}/.codex-log`。

### LLDB 变量查看问题

- 用户观察：断在 `run_interactive_tui` 的 `terminal_info` 判断附近时，hover 只看到 `TerminalInfo` 类型、大小和 layout，看不到字段值。
- 解释：CodeLLDB/Rust 的 hover 经常只展示静态类型信息；当前行黄色箭头表示程序停在“下一条将要执行/正在执行”的源码位置，字段值应优先从 Variables 面板或 LLDB 命令查看。
- 推荐查看方式：在 Debug Console 输入 `frame variable terminal_info`、`frame variable terminal_info.name`、`frame variable terminal_info.term_program`、`frame variable terminal_info.term`。
- 如果仍看不到：把断点放到 `let terminal_info = ...` 的下一行或再下一行，Step Over 一次；或者临时加 `dbg!(&terminal_info);` / `eprintln!("{terminal_info:?}")` 辅助确认。
- 用户进一步观察：Variables 面板的 `Local` 分组为空。
- 补充判断：`run_interactive_tui` 是 `async fn`，Rust 会把局部变量编译进 async state machine；CodeLLDB 在这类帧里经常无法稳定展示源码级 locals。遇到这种情况，优先用临时日志、断到同步 helper 函数内部，或断到 `codex_terminal_detection::terminal_info()` 这类普通同步函数里观察。
- 用户进一步验证：新增 `eprintln!` 后，直接 Restart 调试会话看不到新输出；Cancel/Stop 后重新 Start，新增代码才生效。
- 调试约定：修改源码后不要依赖 Restart，优先 Stop/Cancel 当前调试会话，再重新 Start，确保 Cargo 重新构建并运行新 binary。

## 已发现问题

- `cargo run --bin codex -- --help` 和 `cargo run --bin codex -- exec --help` 输出被用户截断在 Commands/Usage 附近，后续如需完整命令清单可重新运行并保存完整输出。
- 目前尚未执行真实 `codex exec "..."` 模型调用；核心链路已通过 help 和测试完成第一轮入口验证，但真实 turn 的认证、模型请求、工具调用和事件流仍需后续验证。

## 阶段结论

- 本地 Rust workspace 能构建 `codex`。
- `codex` multitool 入口和 `exec` 非交互入口均可运行。
- `codex-exec` 测试通过，说明先沿 `codex-rs/exec` 阅读是合理的。
- 下一步不急于真实调用模型，先沿源码追踪 `Subcommand::Exec -> codex_exec::run_main -> ThreadManager -> CodexThread/session`。
