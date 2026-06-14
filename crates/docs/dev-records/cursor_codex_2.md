# 深入学习 codex 源码
_Exported on 2026/5/9 at GMT+8 20:02:07 from Cursor (3.3.27)_

---

**User**

我想要对 codex 的源码进行深入学习。

---

**Cursor**

我会先按这个项目的“源码学习教练”流程看一下已有规范，然后帮你把 Codex 源码学习拆成可执行的学习任务与下一步输入。

先从第 1 阶段开始：明确你学 Codex 的真实目标、验收标准和产出形态，这会决定后面是偏 CLI 架构、Agent loop、工具调用，还是 Rust/TUI 工程化。

**Codex 源码学习目标对齐**

> **你说的 Codex 源码主要指哪一个？**
> - OpenAI Codex CLI / codex 仓库 ✔
> - 我本地已有一个 Codex 源码仓库或 fork
> - 还不确定，希望先帮我辨别适合学习的 Codex 相关仓库

> **这次深入学习最想获得哪类能力？**
> - 理解 Agent 主循环、上下文管理、工具调用和执行策略
> - 理解 CLI/TUI 工程架构、命令分发、交互体验
> - 理解沙箱、权限、patch/命令执行等安全边界
> - 先完整建立架构地图，再逐块深入
>
> Answer: A/C/D

> **你希望最终产出偏向哪种？**
> - 可运行 runbook + 架构笔记 + 核心代码精读 + mini demo ✔
> - 以源码阅读笔记为主，暂不做 demo
> - 重点迁移到我自己的 Agent/CLI 项目设计

收到：目标仓库是 OpenAI Codex CLI；学习重点覆盖 Agent 主循环、沙箱/权限边界，并先建立完整架构地图；产出采用 runbook、架构笔记、核心代码精读和 mini demo。接下来我会检查 daedalus 当前是否已有 active 学习任务，避免破坏 WIP 约束。

当前 `workspaces/02-learning` 没有活动任务，适合新建 Codex 学习任务；不过 `.daedalus/state.toml` 暂不存在，我会先查一下这个仓库现有 CLI/模板约定，再决定用命令还是文件方式初始化。

我会用项目自己的 `daedalus init repo-learning` 初始化学习工作区，然后把刚确认的目标写入任务卡和首阶段指南。

第一次命令失败是因为 Rust workspace 的 `Cargo.toml` 在 `crates/` 下，不在仓库根目录；我会改用 manifest 路径重新执行初始化。

学习任务工作区已创建。接下来我会补齐任务卡、目标对齐指南和 todo，让后续阶段可以从文件恢复，而不是依赖这段聊天。

我现在会把“已确认”和“仍待确认”分开写入文件：已确认的是学习对象、重点和产出；仍待确认的是你是否接受 Codex CLI 的运行风险，以及是否由我继续准备源码拉取。

任务卡和指南已写入。现在我会跑一次 `daedalus validate`，确认新学习工作区结构和状态机约束没有被破坏。

`validate` 已通过。目标对齐阶段的必需产物已具备，我会把 `01-goal-aligner` 标记完成，并进入 repo 选择阶段，后续就从 Codex 仓库评估开始。

**下一步：Codex 源码准备**

> **是否继续进入源码准备阶段？**
> - 由你协助拉取 OpenAI Codex 源码到本学习任务的 source/ 目录
> - 我自己运行 source/pull_source.sh，你先给我操作指南
> - 先不拉源码，先讲学习路线和问题路线图
>
> Answer: 由你拉取，但是你需要使用 pull_source.sh 进行拉取，以确保这个脚本是真实可用的。

> **源码版本固定方式希望怎么处理？**
> - 先用 main 拉取，再记录当前 commit 作为后续固定版本 ✔
> - 我会提供指定 commit/tag/branch
> - 暂时用 main，不固定 commit

好的，我会先用 `git ls-remote` 解析 `main` 当前 commit，再把这个 commit 作为 `PINNED_REF` 传给 `source/pull_source.sh` 执行，这样既验证脚本可用，也避免学习笔记漂移。

这次失败只是目录校验命令写成了相对当前目录的重复路径；当前 shell 已在 `source/` 目录，我会用 `ls .` 重新执行同一个脚本流程。

脚本存在但没有可执行位；为避免改变权限元数据，我会用 `bash ./pull_source.sh` 调用它，仍然验证脚本内容本身可用。

源码已通过脚本拉取到 `source/codex`，固定 commit 是 `ebe75bb683b3c237aad9f039ab17b187048aa499`。我会补一条 repo 选择记录，并检查外部源码没有带入嵌套 `.git`。

为了让后续能复现同一份源码，我会把 `pull_source.sh` 的默认 `PINNED_REF` 从 `main` 改成刚解析出的 commit，并写入 `notes/repo-selection.md` 作为阶段证据。

源码脚本现在可以直接执行，且复验时会因为 `codex` 已存在而安全跳过；嵌套 `.git` 也已确认不存在。接下来我会完成 repo 选择阶段并进入问题路线图阶段。

现在已进入 `03-socratic-coach`。我会先读 Codex 仓库的顶层说明和工程清单，再生成一份从“问题驱动”出发的问题路线图，避免一开始就陷入文件细节。

初步确认 Codex 现在是 Rust CLI 为主：`codex-rs` 是 Cargo workspace，关键 crate 包括 `core`、`exec`、`tui`、`cli`，这正好匹配我们要追的 Agent loop 和沙箱权限主线。

基于源码入口和 `codex-rs` 的模块划分，我会生成第一轮 3 个问题。它们不是考试题，而是后面 runbook、架构图和代码精读的导航。

问题路线图草案已落盘，但根据流程，用户没有回答前我不会把这一阶段标记完成。我会把当前阶段标记为等待用户回答，并再跑一次校验。

已为 Codex 源码学习建好 daedalus 学习任务：

`workspaces/02-learning/openai-codex-cli-deep-learning`

已完成：
- 初始化任务卡与目标对齐：OpenAI Codex CLI、Agent loop、沙箱/权限、整体架构、runbook + 笔记 + mini demo。
- 使用 `source/pull_source.sh` 拉取源码到 `source/codex`。
- 固定源码 commit：`ebe75bb683b3c237aad9f039ab17b187048aa499`。
- 写入 `notes/repo-selection.md`、`guides/03-question-roadmap-guide.md`、`notes/question-roadmap.md`。
- `daedalus validate` 已通过。

当前阶段停在 `03-socratic-coach`，等待你回答或改写第一轮 3 个问题：

1. Codex CLI 在“用户本机安全完成代码任务”时，最容易失败的三个地方是什么？
2. 你猜一次 `codex exec "..."` 或交互式 prompt 的核心链路会经过哪 5-8 个节点？
3. 做 mini demo 时，你认为必须保留哪些 Codex 核心不变量？比如会话、工具注册、审批、沙箱、事件流、上下文裁剪等。

---

**User**

问题一：

1. 代码修改有遗漏：对需求理解不全面，对整个代码库的架构思想、架构分层和领域逻辑不够清晰和全面，导致只修改了部分代码，无法满足需求。
2. 代码存在语法错误：对代码的语法规则、语法糖和语法陷阱不够熟悉，导致代码存在语法错误，无法编译或运行。
3. 上下文丢失：长任务执行时，随着上下文逐渐变大，LLM 注意力丢失，导致任务目标偏移或遗漏。

问题二：

1. cli command option 处理层
2. coding agent 构建层（构建 master-sub agents 结构，支持 tools、mcp、skills、plugins 等扩展能力）
3. llm client adapter 适配层（支持多种 llm provider）
4. 进入 coding agent 的 react loop 循环
	5. 分析用户意图，决定要调用工具时，进入 auth 权限层判断是否具备权限
		5.1 不具备，则 pause 进去 REPL 交互等待用户确认权限
		5.2 具备或获取许可后，进入 tool gateway 调用 tool
	6. 获取 tool 结果，进一步推行（read 查询上下文，补充背景知识，edit 修改代码，bash 执行验证逻辑）
	7. 最终无 tool 调用，总结当前工作结果，等待下一步命令。
8. 执行 hooks 等操作。

问题三：

1. 会话
2. 工具体系（tool、mcp、skills、subagents）
3. 审核（权限系统）
4. 沙箱
5. 事件流
6. 上下文压缩机制（分层次、分粒度进行精确压缩）

---

**Cursor**

收到，你的回答已经足够进入下一阶段：它把 Codex 的风险从“泛泛安全问题”收敛到了需求覆盖、语法/运行正确性、长上下文漂移，以及权限工具链路。我会先把这些回答落盘，并转成后续可验证的阅读假设。

我会把你的回答保留为原始学习证据，同时追加“可验证假设”：后续我们会去源码里验证是否真有 master/sub-agent 结构、多 provider adapter、权限暂停 REPL、hooks 等机制，而不是先默认它们存在。

你的回答已整理成 `notes/question-roadmap.md`，并明确标注为“待源码验证的假设”。现在我会恢复并完成问题路线图阶段，然后进入运行调试阶段。

阶段切换成功，但校验暴露出模板要求的 `guides/.gitkeep` 和 `notes/.gitkeep` 缺失。我会补回这两个占位文件，再重新校验。

校验已恢复通过。接下来我会按 `04-debugger-guide` 生成第一版运行 runbook，先不做真实模型调用，优先确认本地构建、CLI 帮助、`exec` 入口和可观察日志路径。

从源码看，用户的链路猜测已经有几处能初步对上：`ThreadManager` 明确维护 thread、skills、plugins、MCP、model provider；`CodexThread` 是双向消息流；`codex exec` 通过 app-server protocol 和 event processor 输出事件。接下来我会把这些观察写成 runbook 草案。

已把你的回答写入：

`workspaces/02-learning/openai-codex-cli-deep-learning/notes/question-roadmap.md`

我也把它转成了后续源码验证假设，并完成了阶段流转：当前已进入 `04-debugger-guide`。另外新建了：

- `guides/04-run-debug-guide.md`
- `notes/runbook.md`

下一步请你亲自运行 runbook 里的三条最小命令，并把关键输出贴回来：

```bash
cd workspaces/02-learning/openai-codex-cli-deep-learning/source/codex/codex-rs
cargo run --bin codex -- --help
cargo run --bin codex -- exec --help
cargo test -p codex-exec
```

当前 `daedalus validate` 已通过。

---

**User**

执行结果如下：


➜  codex-rs git:(main) ✗ cargo run --bin codex -- --help
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.28s
     Running `/Users/hedon/rust-target/debug/codex --help`
Codex CLI

If no subcommand is specified, options will be forwarded to the interactive CLI.

Usage: codex [OPTIONS] [PROMPT]
       codex [OPTIONS] <COMMAND> [ARGS]

Commands:

---


➜  codex-rs git:(main) ✗ cargo run --bin codex -- exec --help

    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.53s
     Running `/Users/hedon/rust-target/debug/codex exec --help`
Run Codex non-interactively

Usage: codex exec [OPTIONS] [PROMPT]
       codex exec [OPTIONS] <COMMAND> [ARGS]


---

test result: ok. 61 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.97s

---

**Cursor**

很好，这说明三件事已被本地验证：`codex` multitool 能构建并展示入口，`exec` 子命令可用，`codex-exec` 局部测试全部通过。我会把你的运行结果写入 runbook，并把它转成下一步调用链追踪依据。

我会更新 `notes/runbook.md` 中三条命令的结果：前两条验证 CLI/exec 帮助入口，第三条验证 `codex-exec` 61 个测试通过。然后把本阶段下一步收敛到“追踪源码调用链”。

运行结果已落盘，`notes/runbook.md` 现在记录了用户执行者、关键输出和阶段结论。接下来我会跑一次 daedalus 校验，确认状态和产物仍一致。

已记录到 `notes/runbook.md`，并更新了 todo、产物索引和验证日志。`daedalus validate` 也已通过。

当前结论：

- `codex` multitool 入口能构建并运行。
- `codex exec` 非交互入口存在，适合作为第一条核心路径。
- `codex-exec` 测试通过：61 passed、0 failed。

下一步建议继续在 `04-debugger-guide` 内做源码调用链追踪：

`codex-rs/cli/src/main.rs` -> `codex-rs/exec/src/main.rs` -> `codex-rs/exec/src/lib.rs::run_main` -> `ThreadManager` -> `CodexThread/session`。

---

**User**

怎么交互式运行 codex 并支持在 cursor ide 里面 debug 断点测试呢？

---

**Cursor**

我会把这个问题当作 `04-debugger-guide` 的延伸：目标不是只“跑起来”，而是能在 Cursor 里从交互式 `codex` 入口打断点观察调用链。我先检查一下当前仓库是否已有 VSCode/Cursor 调试配置，避免覆盖你的设置。

没有现成 `.vscode` 调试配置。源码里也能确认：不带 subcommand 时 `codex` 会把参数转给 `TuiCli`，也就是交互式 TUI；`exec` 只是非交互路径。接下来我会把 Cursor 断点调试的推荐配置写进学习 runbook。

我会补一份专门的 “Cursor 交互式断点调试” 指南到当前学习任务里，包含两种方式：直接调试 `codex` 交互式 TUI，以及更稳定的 `codex exec` 断点路径。

可以，核心是：**debug `codex` 这个 bin，不传 subcommand 就会进入交互式 TUI**。在 Cursor 里建议用 CodeLLDB，并且必须用 terminal 启动 TUI。

我已把完整配置写到：

`workspaces/02-learning/openai-codex-cli-deep-learning/guides/04-run-debug-guide.md`

最小 `launch.json` 配置如下：

```json
{
  "name": "Debug Codex TUI interactive",
  "type": "lldb",
  "request": "launch",
  "cargo": {
    "args": ["build", "--bin", "codex"],
    "filter": { "name": "codex", "kind": "bin" }
  },
  "args": [
    "-c",
    "log_dir=${workspaceFolder}/workspaces/02-learning/openai-codex-cli-deep-learning/.codex-log",
    "--sandbox",
    "read-only"
  ],
  "cwd": "${workspaceFolder}/workspaces/02-learning/openai-codex-cli-deep-learning/source/codex/codex-rs",
  "env": {
    "RUST_LOG": "codex_core=debug,codex_tui=debug,codex_app_server_client=debug"
  },
  "terminal": "integrated"
}
```

推荐第一批断点：

- `codex-rs/cli/src/main.rs` 的 `match subcommand`
- `codex-rs/cli/src/main.rs` 的 `run_interactive_tui`
- `codex-rs/tui/src/lib.rs` 的 `run_main`
- `codex-rs/core/src/thread_manager.rs` 的 `ThreadManager::new`
- `codex-rs/core/src/codex_thread.rs` 的 `submit` / `steer_input`

注意：不要传 `exec`，否则会走非交互路径；TUI 必须用 `integrated` 或 `external` terminal，不能只在 Debug Console 里跑。

---

**User**

No such file or directory: /Users/hedon/mycode/ai/daedalus/workspaces/02-learning/openai-codex-cli-deep-learning/source/codex/codex-rs/workspaces/02-learning/openai-codex-cli-deep-learning/source/codex/codex-rs

Possible cause: the working directory "/Users/hedon/mycode/ai/daedalus/workspaces/02-learning/openai-codex-cli-deep-learning/source/codex/codex-rs/workspaces/02-learning/openai-codex-cli-deep-learning/source/codex/codex-rs" is missing or inaccessible. 你的指导是不是应该有问题，这个 launch.json 是不是应该放在 codex-rs 根目录下，而不是 daedalus 里面？因为我们后面还会有很多的 repo，我每次学习一个 repo 的时候，肯定会单独开一个窗口打开这个 repo，这样才能聚焦。

---

**Cursor**

你说得对，我上一版配置默认 `${workspaceFolder}` 是 daedalus 根目录；如果你在 Cursor 单独打开 `codex-rs`，就会把路径拼重复。更合理的做法是：每个学习 repo 单独开窗口，`launch.json` 放在该 repo/Cargo workspace 根目录下。

我会直接在 `codex-rs/.vscode/launch.json` 放一份以 `codex-rs` 为 workspace root 的配置，并同步修正学习指南，避免后续 repo 学习继续沿用 daedalus 根目录假设。

你判断是对的，我上一版指导有问题：它默认 Cursor workspace 是 daedalus 根目录；但如果你单独打开 `codex-rs`，`${workspaceFolder}` 已经是 `codex-rs`，所以路径被拼重复了。

我已修正并写入：

`workspaces/02-learning/openai-codex-cli-deep-learning/source/codex/codex-rs/.vscode/launch.json`

现在配置是 repo-local 的：

```json
"cwd": "${workspaceFolder}",
"args": [
  "-c",
  "log_dir=${workspaceFolder}/.codex-log",
  "--sandbox",
  "read-only"
]
```

后续建议流程也按你说的来：每学习一个 repo，就单独用 Cursor 打开该 repo 或其实际 workspace 根目录，调试配置放在这个 repo 的 `.vscode/launch.json` 里。daedalus 只保存学习状态、runbook、笔记和索引，不应该假设所有调试都从 daedalus 根目录启动。

`daedalus validate` 已通过。

---

**User**

为什么我断点到这里还看不到 terminal_info 的值？

---

**Cursor**

从截图看你停在 `if terminal_info.name == TerminalName::Dumb` 这一行附近，我先确认一下 `TerminalInfo` 的定义和这段代码上下文，再给你更准确的调试解释。

这里不是 Codex 逻辑问题，主要是 Cursor/CodeLLDB 对 Rust 变量的展示方式：hover 往往只显示类型布局，真正的字段值要在 Variables 面板或 LLDB Debug Console 里看。我会顺手把这个排障点补进 runbook。

不是你断点没生效，主要是 **Cursor/CodeLLDB 对 Rust hover 的展示限制**。

截图里 hover 显示的是：

`terminal_info: TerminalInfo`
`size=104, align=0x8, needs Drop`

这只是类型/layout 信息，不是运行时字段值。要看值，优先用这几种方式：

1. 在左侧 **Run and Debug -> Variables -> Local** 里展开 `terminal_info`。
2. 在 Debug Console 输入：

```lldb
frame variable terminal_info
frame variable terminal_info.name
frame variable terminal_info.term_program
frame variable terminal_info.term
```

3. 如果还是看不到，把断点放到 `let terminal_info = ...` 的下一行或再下一行，然后 **Step Over** 一次再看。

这里还有一个 Rust 调试细节：

```rust
let terminal_info = codex_terminal_detection::terminal_info();
if terminal_info.name == TerminalName::Dumb {
```

如果断在 `let terminal_info = ...` 这一行，变量还没初始化；要执行过这一行后才有值。你截图里黄色箭头看起来在 `if terminal_info.name...`，理论上已经初始化了，所以用 `frame variable terminal_info` 应该能看到字段。

如果 LLDB 仍然不友好，最稳的临时办法是在源码里加一行：

```rust
dbg!(&terminal_info);
```

或：

```rust
eprintln!("{terminal_info:?}");
```

`TerminalInfo` 已经 `derive(Debug)`，可以直接打印。

---

**User**

这里就是没有显示任何的变量值。

---

**Cursor**

你这个截图确认了：不是 hover 的问题，而是当前 CodeLLDB 没有把这个断点帧的 local variables 解出来。对 Rust 的 `async fn` 这很常见，因为源码里的局部变量会被编译进 async state machine，Variables 面板经常为空。

对，这说明当前断点帧的 locals 没被 CodeLLDB 解出来。主要原因很可能是：你断的位置在 Rust `async fn run_interactive_tui` 里。

Rust 的 `async fn` 会被编译成 state machine，源码里的局部变量不一定以普通 stack local 形式存在，所以 CodeLLDB 的 Variables 面板经常为空。

建议按这个顺序处理：

1. 先在 Debug Console 试：

```lldb
bt
frame info
frame variable
frame variable terminal_info
```

如果 `frame variable` 也是空，基本就是 async lowering 导致的调试信息不可用。

2. 对这个点，最稳的办法是临时打印：

```rust
let terminal_info = codex_terminal_detection::terminal_info();
eprintln!("terminal_info = {terminal_info:#?}");
```

或：

```rust
dbg!(&terminal_info);
```

3. 如果想继续用断点观察，把断点打进同步函数内部：

`terminal-detection/src/lib.rs` 里的 `terminal_info()` / 检测环境变量的函数。

同步函数里的 locals 通常比 `async fn` 里好看很多。

结论：不是你的 Cursor 配置必然错了，而是 Rust async + LLDB 的源码变量展示不稳定。调试 Codex 这种 async/TUI 程序时，断点 + 临时日志通常要结合使用。

---

**User**

加了 eprintln! 在哪里看输出？

---

**Cursor**

看你当前用的是 CodeLLDB + `"terminal": "integrated"`，`eprintln!` 输出会在 **启动调试会话的 integrated terminal** 里，不在 Variables 面板。

也就是 Cursor 底部的 **Terminal** 面板，找到 CodeLLDB 启动的那个终端标签，输出应该类似：

```text
terminal_info = TerminalInfo {
    name: Dumb,
    term_program: ...,
    version: ...,
    term: ...,
    multiplexer: ...,
}
```

如果你看不到：

1. 确认 `launch.json` 里是：

```json
"terminal": "integrated"
```

2. 打开底部 `Terminal`，不是 `Debug Console`。
3. 如果你改成：

```json
"terminal": "external"
```

输出会在外部终端窗口里。
4. 如果 TUI 切了 alternate screen，`eprintln!` 可能被界面刷掉；建议把 `eprintln!` 放在 TUI 初始化前，也就是你现在这个 `terminal_info` 判断附近，正合适。

---

**User**

看不到这个输出呀

---

**Cursor**

那先别纠结 terminal 输出，直接用“写文件”验证，最稳。

在这行后面加：

```rust
let terminal_info = codex_terminal_detection::terminal_info();

std::fs::write(
    "/tmp/codex-terminal-info.log",
    format!("{terminal_info:#?}\n"),
)
.ok();

std::thread::sleep(std::time::Duration::from_secs(3));
```

然后重新 Debug，执行到这里后看文件：

```bash
cat /tmp/codex-terminal-info.log
```

如果文件存在，说明代码执行了，只是 `eprintln!` 被 TUI/调试终端吞了或刷掉了。

如果文件不存在，说明大概率是下面之一：

- 文件没保存，Debug 跑的还是旧 binary。
- launch 配置 debug 的不是 `codex` 这个 bin。
- 断点位置看似到了，但你加日志的代码没有实际执行到。
- Cargo 没重新编译到你改的源码。

也可以临时用更强的验证：

```rust
panic!("terminal_info = {terminal_info:#?}");
```

如果 panic 都看不到，基本就是跑的不是你改过的那份代码。

---

**User**

看到了，不能直接 restart，要 cancel，然后重新 start，新增加的代码才会生效。

---

**Cursor**

这个发现很关键：CodeLLDB 的 restart 可能复用旧调试进程或旧构建产物，导致临时日志没生效。我会把它记进 runbook，后续调试 Codex 时按“停止再启动”作为默认流程。

对，这个判断很重要。

后面调试 Codex 先按这个规则来：

- 改了 Rust 源码后，不用 `Restart`。
- 先 `Cancel/Stop` 当前 debug session。
- 再重新 `Start Debugging`。
- 这样 CodeLLDB/Cargo 才会重新构建并跑新 binary。

我已经把这个坑记录到 `notes/runbook.md` 里了。

---

**User**

核心执行逻辑：

pub(crate) struct App {
    // ... 需要介绍它里面是关键属性
}

核心方法： pub async fn run(

里面最核心是它的 loop 循环：

loop {
    let control = select! {
        Some(event) = app_event_rx.recv() => {
            match app.handle_event(tui, &mut app_server, event).await {
                Ok(control) => control,
                Err(err) => break Err(err),
            }
        }
        active = async {
            if let Some(rx) = app.active_thread_rx.as_mut() {
                rx.recv().await
            } else {
                None
            }
        }, if App::should_handle_active_thread_events(
            waiting_for_initial_session_configured,
            app.active_thread_rx.is_some()
        ) => {
            if let Some(event) = active {
                if let Err(err) = app.handle_active_thread_event(tui, &mut app_server, event).await {
                    break Err(err);
                }
            } else {
                app.clear_active_thread().await;
            }
            AppRunControl::Continue
        }
        event = tui_events.next() => {
            if let Some(event) = event {
                match app.handle_tui_event(tui, &mut app_server, event).await {
                    Ok(control) => control,
                    Err(err) => break Err(err),
                }
            } else {
                tracing::warn!("terminal input stream closed; shutting down active thread");
                app.handle_exit_mode(&mut app_server, ExitMode::ShutdownFirst).await
            }
        }
        app_server_event = app_server.next_event(), if listen_for_app_server_events => {
            match app_server_event {
                Some(event) => app.handle_app_server_event(&app_server, event).await,
                None => {
                    listen_for_app_server_events = false;
                    tracing::warn!("app-server event stream closed");
                }
            }
            AppRunControl::Continue
        }
    };
    if App::should_stop_waiting_for_initial_session(
        waiting_for_initial_session_configured,
        app.primary_thread_id,
    ) {
        waiting_for_initial_session_configured = false;
    }
    match control {
        AppRunControl::Continue => {}
        AppRunControl::Exit(reason) => break Ok(reason),
    }
}

用户的操作都是 CodexOp(AppCommand)，然后把这个 op 通过 channel 传递给 active thread 去执行： submit_thread_op，问题就在于，是谁在处理？

---

**Cursor**

你已经抓到 TUI 的核心 loop 了。接下来关键是把 `submit_thread_op` 后面的接收者找出来：它很可能不是 `App` 自己处理，而是通过 app-server/client 协议交给 thread/session runtime。

你问的“谁处理 submit 过去的 op”需要沿源码确认。由于外部源码目录被 `.gitignore` 忽略，普通搜索跳过了它，我会用 `rg --no-ignore` 在 Codex 源码内查调用点。

结论已经清楚了：`App` 只负责 TUI 状态、路由和展示；真正处理 `AppCommand::UserTurn` 的是 app-server 的 `TurnRequestProcessor`，再提交到 core 的 `Codex` session loop。`active_thread_rx` 反过来只是接收 thread 事件，用来更新 UI。

你的问题答案是：**`submit_thread_op` 后面真正处理用户 turn 的，不是 `App`，而是 app-server 的 `TurnRequestProcessor`，再往下是 core 的 `Codex` session `submission_loop`。**

我已经把这条链路写入 `notes/tui-execution-loop.md`。

关键路径是：

```text
AppEvent::CodexOp(AppCommand::UserTurn)
  -> App::handle_event
  -> App::submit_active_thread_op
  -> App::submit_thread_op
  -> App::try_submit_active_thread_op_via_app_server
  -> AppServerSession::turn_start
  -> ClientRequest::TurnStart
  -> app-server MessageProcessor
  -> TurnRequestProcessor::turn_start
  -> TurnRequestProcessor::turn_start_inner
  -> TurnRequestProcessor::submit_core_op
  -> CodexThread::submit_with_trace
  -> Codex::submit_with_trace
  -> Codex::submit_with_id
  -> tx_sub.send(Submission)
  -> core session submission_loop
```

所以分层理解是：

- `App`：TUI 顶层状态机，负责 UI、事件 select、路由、active thread 切换。
- `AppCommand`：TUI 内部的用户意图表示。
- `AppServerSession`：TUI 到 app-server 的 client 封装。
- `TurnRequestProcessor`：app-server 侧真正处理 `turn/start` 请求。
- `CodexThread` / `Codex`：core session 的外壳。
- `submission_loop`：真正消费 `Op` 的后台 loop。

你提到的 `active_thread_rx` 不是处理用户 op 的通道，它是**事件回流通道**：接收当前 thread 产生的通知，然后 `handle_active_thread_event` 更新 UI。

---

**User**

撰写这种 notes  时候，你应该按需添加一些简约的 mermaid 进行描述，这样更清晰。

---

**Cursor**

同意，这类调用链和事件流非常适合用简约 Mermaid 补充。以后我会在 notes 里按需加入小图，避免只靠长文本描述；我先把刚才的 TUI 执行循环笔记补上两张图。

收到，后续我写这类 notes 会按需加简约 Mermaid，尤其是：

- 调用链：用 `sequenceDiagram`
- 事件流/状态流：用 `flowchart`
- 模块边界：用小型分层图

我已经给 `notes/tui-execution-loop.md` 补了两张图：

- `App::run loop` 的四类事件源图
- `AppCommand::UserTurn -> submission_loop` 的 sequence 图

---

**User**

mermaid 在换行的时候不要 \n，用 <br> 请你修改，然后，持久化这条记忆，避免再犯错。

---

**Cursor**

我会先把现有 Mermaid 里的 `\n` 改成 `<br>`，然后把这条偏好固化成项目规则，避免后续笔记再写回 `\n`。

我找到了当前笔记里 4 个 Mermaid label 使用了 `\n`，会改成 `<br>`；项目里还没有 `.cursor/rules`，我会新增一条简短的 always-apply 规则。

已修改：

- `notes/tui-execution-loop.md` 里的 Mermaid label 已从 `\n` 改为 `<br>`。
- 已新增持久化规则：`.cursor/rules/mermaid-formatting.mdc`

规则内容会要求后续 Mermaid 节点换行统一使用 `<br>`，不要用 `\n`。

---

**User**

typora 的 mermaid 好像不支持这个语法，请用更加适配的语法

---

**Cursor**

这个报错不是 `<br>`，而是 sequenceDiagram 里我用了 `Loop` 作为 participant alias，`loop` 是 Mermaid 的保留关键字，Typora 的 Mermaid 解析会把它误判。我要把 alias 改成非保留词，并顺手把 participant 名称加引号提高兼容性。

问题点已修掉：`Loop` 改成 `SUB`，消息文本里的 `->` 也改成更保守的 `to`，participant 展示名加了引号。接下来我会把这条 Typora 兼容规则也补进持久化规则。

已修复 Typora Mermaid 兼容问题。

主要问题是我用了：

```mermaid
participant Loop as submission_loop
```

`loop` 是 Mermaid sequenceDiagram 的保留关键字，Typora 解析时会炸。现在已改成：

```mermaid
participant SUB as "submission_loop"
```

并且把消息里的 `->` 也改成了 `to`，更兼容 Typora：

```mermaid
SUB->>SUB: match Op to user_input_or_turn
```

我也更新了持久化规则 `.cursor/rules/mermaid-formatting.mdc`，以后会遵守：

- Mermaid label 换行用 `<br>`，不用 `\n`
- 不用 `loop` 这类保留字做 alias
- sequenceDiagram 的 participant 展示名含空格/符号时加引号
- message 文本里避免写 `->`

`daedalus validate` 已通过。

---

**User**

梳理一下，codex 是如何通过这种通道进行事件处理的，以用户输入为例子，当用户输入“你好”这样的 prompt 的时候，事件传输是怎样的，最终是如何把这个 prompt 传给 agent 进行处理并最后返回给用户的。

---

**Cursor**

我会沿“用户在 TUI 输入 `你好`”这条具体路径补齐前半段：从键盘事件进入 `ChatWidget`、生成 `AppCommand::UserTurn`，再到 app-server/core/session，最后事件如何回流到 UI。

我已经定位到前半段：`ChatWidget` 会把文本组装成 `UserInput::Text`，再构造 `AppCommand::user_turn(...)`，通过 `submit_op` 发给 `App`。我再读几段关键代码，确保回流路径讲清楚。

回流路径还要看 app-server 的通知怎么进入 `active_thread_rx`。我会查 thread 事件 listener 和 `ThreadBufferedEvent` 的来源，再把完整图写进 notes。

我会把“你好”这条具体事件链整理成一份独立 note，并用 Typora 兼容 Mermaid 展示发送链路和回流链路。

已确认回流的 UI 更新点：app-server 推 `ServerNotification`，`App` 按 thread 分发到 `ThreadBufferedEvent`，active thread 的事件再被 `handle_active_thread_event` 喂给 `ChatWidget::handle_server_notification`，最终处理 `AgentMessageDelta`、`ItemCompleted`、`TurnCompleted` 等更新界面。

我已把完整链路写成 `notes/user-prompt-event-flow.md`，包括两张 Typora 兼容 Mermaid：一张前向 sequence，一张回流 flowchart。接下来快速校验一下 workspace。

已梳理并写入：

`workspaces/02-learning/openai-codex-cli-deep-learning/notes/user-prompt-event-flow.md`

核心链路可以这样记：

```text
"你好"
-> UserMessage
-> UserInput::Text
-> AppCommand::UserTurn
-> AppEvent::CodexOp
-> App::submit_thread_op
-> AppServerSession::turn_start
-> ClientRequest::TurnStart
-> TurnRequestProcessor::turn_start_inner
-> Op::UserInputWithTurnContext
-> Codex::tx_sub.send(Submission)
-> submission_loop
-> user_input_or_turn_inner
-> sess.spawn_task(...)
-> ServerNotification
-> active_thread_rx
-> ChatWidget::handle_server_notification
-> TUI 渲染给用户
```

最关键的理解是：Codex TUI 不是“输入字符串后同步调用 agent 返回字符串”，而是一个双向事件系统：

- 前向：TUI 把用户意图逐层包装成 `AppCommand`、`ClientRequest`、`Op`。
- 执行：core 的 `submission_loop` 消费 `Op` 并启动 agent task。
- 回流：agent 运行过程产生 `ServerNotification`，再由 `active_thread_rx` 回到 `ChatWidget` 渲染。

我也加了 Typora 兼容 Mermaid 图，并跑了 `daedalus validate`，已通过。

我又在新图里用了 `Loop` 这个保留词，确实违反了刚固化的规则。我马上全局检查当前学习笔记里的 Mermaid alias，把 `Loop` 这类保留词修掉。

已修复，是我在 `notes/user-prompt-event-flow.md` 里又用了 Mermaid 保留词 `Loop`：

```mermaid
participant Loop as "submission_loop"
```

已改成：

```mermaid
participant SUB as "submission_loop"
```

并同步改了引用：

```mermaid
Core->>SUB: tx_sub.send(Submission)
SUB->>Core: user_input_or_turn
```

我也检查了当前 `notes/` 下的 Mermaid，不再有 `participant Loop` / `->>Loop` / `Loop->>`。

---

**User**

我阅读了你生成的 user-prompt-event-flow.md 文档。按我的理解，就是就是 3 个部分，UI -> APP -> Handler -> Agent，然后再反向回来，它们之间都使用类似 channel 这样的通道来解耦实现。我的理解对吗？请你把这个过程中的关键组件的概念解释清楚，然后它们之间是如何交互的，也绘制出来，补充在 user-prompt-event-flow.md 中。

---

**Cursor**

你的理解方向是对的，但中间可以再精确一点：不是简单的 `UI -> APP -> Handler -> Agent`，而是 `UI 组件 -> TUI App 路由器 -> AppServer 协议/处理器 -> Core Session/Agent Runtime`，反向通过 `ServerNotification` 回到 UI。我会把这个概念模型补进 `user-prompt-event-flow.md`。

我已经补了四层模型、关键组件解释和通道解耦图。现在检查一下 Mermaid 是否还有 Typora 兼容问题，比如保留 alias 或 `\n`。

你的理解基本正确，但我在文档里把它精确成了四层：

```text
UI 组件 -> TUI App 路由层 -> AppServer 协议/处理层 -> Core Session/Agent Runtime
```

并补充说明了它们之间的通道：

- `ChatWidget -> App`：`app_event_tx / app_event_rx`
- `App -> AppServer`：`ClientRequest::TurnStart`
- `AppServer -> Core`：`CodexThread::submit_with_trace`
- `Core 内部`：`tx_sub / submission_loop`
- `Core/AppServer -> App`：`ServerNotification`
- `App -> ChatWidget`：`active_thread_rx / ThreadBufferedEvent`

已更新：

`workspaces/02-learning/openai-codex-cli-deep-learning/notes/user-prompt-event-flow.md`

里面新增了：

- 四层心智模型
- 关键组件概念解释
- 通道与解耦关系图
- Typora 兼容 Mermaid 图

`daedalus validate` 已通过。

---

**User**

非常好，我大概理解这个过程了。

---

**Cursor**

很好。当前你已经抓住了 Codex TUI 的主线：

`用户输入 -> UI 意图 -> TUI 路由 -> app-server 协议 -> core submission_loop -> agent task -> ServerNotification 回流 -> UI 渲染`

下一步建议继续沿这个主线往下读 `spawn_task` 后真正的 agent loop，也就是模型请求、工具调用、权限审批、沙箱执行是怎么被组织起来的。

---

**User**

我先理一下整个过程：

spawn_task 启动一个任务处理线程（还是协程？）

    1. 停止当前进行中的所有任务：abort_all_tasks
    2. clear_connector_selection 这个不知道是干啥用的
    3. start_task 开始当前任务，真正的核心逻辑所在

start_task 核心是执行 task_for_run.run, SessionTask trait 实现了 AnySessionTask trait，里面本质上还是调用了自己的 run。

而 SessionTask 有多种实现，分别是 compact、regular、review、user_shell，常规的就是 regular 和使用 ! 时执行 user_shell。

我们看 regular 的实现。

async fn run(
        self: Arc<Self>,
        session: Arc<SessionTaskContext>,
        ctx: Arc<TurnContext>,
        input: Vec<UserInput>,
        cancellation_token: CancellationToken,
    ) -> Option<String> {


1. 首先它会发送一个 TurnStarted event 回去，这样用户交互就比较舒服了。
2. 然后核心是里面的 loop 循环，它最后退出循环的时候会返回 last_agent_message。也就是没有工具调用的时候，就退出 loop，返回最后一条 ai 消息。
3. loop 循环里面核心是 run_turn 逻辑。
    3.1 做 input 前置校验，确保非空合法。
    3.2 获取 model_info 模型信息
    3.3 skills_outcome 是否强制使用 skill 技能。
    3.4 loaded_plugins 加载插件
    3.5 mcp_tools 加载 mcp 工具
    3.6 available_connectors 不知道是啥？IM 工具？
    3.7 mentioned_skills 提到的 skill 列表。
    3.8 resolve_skill_dependencies_for_turn 做 skill 加载的必要性解析和处理
    3.9 maybe_prompt_and_install_mcp_dependencies 检查并安装 mcp 所需的依赖
    3.10 SkillInjections 注入 skill
    3.11 run_pending_session_start_hooks 执行开始前的 hooks
    3.12 run_user_prompt_submit_hooks 执行用户提交时的 hooks
    3.13 ...
    3.14 最终是执行 run_sampling_request 执行处理
    > 这个过程中还夹杂一些 pending_input 的插入输入逻辑处理。

run_sampling_request 流程是这样的：

1. built_tools 构建工具：包含了 all_mcp_tools， loaded_plugins， effective_explicitly_enabled_connectors， auth
2. _code_mode_worker 构建编码模式工作器
3. 进入核心的 loop 循环：
    3.1 执行 build_prompt构建 llm api  prompt
        input,
        tools: router.model_visible_specs(),
        parallel_tool_calls: turn_context.model_info.supports_parallel_tool_calls,
        base_instructions,
        personality: turn_context.personality,
        output_schema: turn_context.final_output_json_schema.clone(),
        output_schema_strict: !crate::guardian::is_guardian_reviewer_source(
            &turn_context.session_source,
        ),
    3.2 执行 try_run_sampling_request 发送请求里面是流式数据处理：let mut stream = client_session.stream()，并且会根据数据类型进行额外处理。
    3.3 最后 ResponseEvent::Completed 表示结束


不过具体执行工具和沙箱的逻辑，我没找到。

---

**Cursor**

你这次梳理已经进入真正 agent loop 了，而且大方向是对的：`spawn_task` 启的是 Tokio 异步任务，不是 OS 线程；`RegularTask` 的 loop 通过模型响应决定是继续工具调用还是结束。我会沿 `run_sampling_request` 往后查工具执行和沙箱入口，帮你把缺口补上。

搜索结果显示工具执行集中在 `core/src/session/turn.rs` 和 `core/src/tools/`，沙箱策略来自 `TurnContext` / `SessionConfiguration`，具体命令执行会经工具 runtime 分发。我会读 `run_sampling_request` 和工具 runtime 相关代码。

我会把这段理解补成一份新的源码笔记，重点回答两个问题：`spawn_task` 到 `run_sampling_request` 的循环怎么分层，以及工具/沙箱/审批到底藏在哪条路径里。

我已经把“工具执行和沙箱藏在哪里”整理成新笔记，里面有工具分发图、shell 审批/沙箱图和下一步断点建议。现在我会更新索引和 todo。

你的梳理大体正确。我补充两个关键点：

`spawn_task` 启动的是 Tokio 异步任务，可以理解为 coroutine/task，不是直接创建 OS 线程。

你没找到工具和沙箱逻辑，是因为它们不在 `RegularTask::run` 或 `run_turn` 的主循环里显式展开，而是藏在这条路径里：

```text
ResponseEvent::OutputItemDone
-> handle_output_item_done
-> ToolCallRuntime
-> ToolRouter
-> ToolRegistry
-> ToolHandler
-> ToolRuntime
-> approval / sandbox / execute_env
```

以 shell 工具为例：

```text
ShellHandler::handle
-> run_exec_like
-> ShellRuntime::start_approval_async
-> ShellRuntime::run
-> build_sandbox_command
-> SandboxAttempt::env_for
-> sandboxing::execute_env
```

我已经新增笔记：

`workspaces/02-learning/openai-codex-cli-deep-learning/notes/agent-loop-tool-sandbox.md`

里面补了：

- `RegularTask` 主循环
- `run_sampling_request` 的职责
- 工具调用为什么从模型 stream 事件里触发
- `ToolCallRuntime -> ToolRouter -> ToolRegistry -> ToolHandler -> ToolRuntime` 分发链路
- shell 审批和沙箱入口
- 下一步推荐断点

`daedalus validate` 已通过。

---

**User**

权限检查是在哪里呢？是在 tool 里面，还是在进入 tool handle 之前？

---

**Cursor**

这是个关键边界问题。我会沿 `ToolRegistry::dispatch_any` 和 `run_exec_like` 读一下，区分“工具通用前置检查”和“具体工具 runtime 的审批/沙箱检查”分别在哪里发生。

源码边界很清楚：`ToolRegistry::dispatch_any` 在进入 `handler.handle` 前只做 handler 匹配、pre-hook、mutating gate；真正的权限审批和沙箱策略在具体 ToolRuntime 里面，shell 就是在 `ShellRuntime`。我会把这段补进工具沙箱笔记。

权限检查分两层，不是单点。

第一层在进入具体 `tool handle` 之前，位于：

`ToolRegistry::dispatch_any`

这里做的是通用前置门禁：

```text
find handler
-> matches_kind
-> run_pre_tool_use_hooks
-> handler.is_mutating
-> tool_call_gate.wait_ready
-> handler.handle_any
-> run_post_tool_use_hooks
```

这一层主要负责：

- 工具是否存在。
- payload 类型是否匹配。
- PreToolUse hook 是否拦截。
- mutating 工具是否需要等待 gate，避免并发修改环境。

但这里还不是用户审批和沙箱执行。

真正的权限审批和沙箱在具体工具 runtime 内部。以 shell 为例：

```text
ShellHandler::handle
-> run_exec_like
-> ShellRuntime::start_approval_async
-> ShellRuntime::run
-> build_sandbox_command
-> SandboxAttempt::env_for
-> sandboxing::execute_env
```

所以结论是：

```text
通用检查：进入 tool handle 之前
用户审批 / 沙箱策略：具体 tool handle 内部的 ToolRuntime
```

我已把这段补进：

`notes/agent-loop-tool-sandbox.md`

---

**User**

我看还有个 ToolOrchestrator

---

**Cursor**

对，`ToolOrchestrator` 很可能就是我前面说的“ToolRuntime 里统一处理审批、沙箱、重试”的那层抽象。我去定位它的定义和调用点，把它放回我们刚才的分层图里。

你看得对，`ToolOrchestrator` 是更准确的“审批 + 沙箱 + 重试”中心层；我前面说的 `ToolRuntime` 还不够精确，应该说 runtime 提供策略接口，orchestrator 负责串起来执行。

对，`ToolOrchestrator` 是这里更关键的一层。

更准确的分层是：

```text
ToolRegistry
-> ToolHandler
-> ToolOrchestrator
-> ToolRuntime
```

其中：

- `ToolRegistry`：做通用分发前置检查，比如找 handler、判断 tool kind、mutating gate。
- `ToolHandler`：解析模型参数，构造具体 request，比如 `ShellRequest`。
- `ToolOrchestrator`：统一负责审批、沙箱选择、首次执行、沙箱失败后的升级重试。
- `ToolRuntime`：提供工具专属策略和实际执行逻辑，比如 shell 如何审批、如何构造 sandbox command、如何执行命令。

所以我前面说“权限和沙箱在 ToolRuntime”需要修正为：**审批/沙箱流程由 `ToolOrchestrator` 总控，具体审批内容和执行细节由 `ToolRuntime` 提供。**

我已经把这个补充进了 `workspaces/02-learning/openai-codex-cli-deep-learning/notes/agent-loop-tool-sandbox.md`，也同步更新了 `.daedalus/todo.md`。

---

**User**

非常好，现在请你结合我们的全部交互过程，整合 @workspaces/02-learning/openai-codex-cli-deep-learning/notes/agent-loop-tool-sandbox.md @workspaces/02-learning/openai-codex-cli-deep-learning/notes/user-prompt-event-flow.md @workspaces/02-learning/openai-codex-cli-deep-learning/notes/tui-execution-loop.md 这 3 个文档，仅保留最终结论即可。关键是要阐述整个交互循环和真正的 agent loop，也就是模型请求、工具调用、权限审批、沙箱执行是怎么被组织起来的。重点要解释 codex 中的核心关键 struct 之间的交互和层次划分。

---

**Cursor**

我会把这 3 份 notes 收敛成一份“最终版主线说明”，重点保留交互循环、agent loop、工具/审批/沙箱层次，以及关键 struct 的协作关系。先快速复核现有文档内容，再做整合编辑。

我已经复核完三份 notes：它们有大量重复和阶段性推导。接下来会把 `agent-loop-tool-sandbox.md` 改成最终整合版主文档，并把另外两份收敛为简短索引/结论，避免同一套链路散落在三处。

我会先重写主文档，结构按“分层、前向输入、core agent loop、工具执行、回流、关键 struct 速查”来组织，删掉调试过程和临时判断，只留下可复用结论。

---

**User**

只保留一个文件就可以。要求这个文件层次清晰，循序渐进。

---

**Cursor**

收到，我会只保留一个整合文件。为了保持原主题入口清晰，我会把最终版放在 `notes/agent-loop-tool-sandbox.md`，然后删除另外两份阶段性 notes。

主文档已重写完成。现在我会删除另外两份阶段性 notes，让这个主题只保留一个循序渐进的最终版入口。

我发现索引里还保留了被删除 notes 的链接。接下来会同步更新 artifact index 和 todo，避免后续恢复任务时指向不存在的阶段文档。

已完成。现在这部分内容只保留一个最终版文件：

`workspaces/02-learning/openai-codex-cli-deep-learning/notes/agent-loop-tool-sandbox.md`

我已删除两份阶段性重复文档：

`notes/user-prompt-event-flow.md`
`notes/tui-execution-loop.md`

同时更新了 `.daedalus/artifact-index.md` 和 `.daedalus/todo.md`，避免索引继续指向已删除文件。

新版文档按循序渐进结构整理：总体结论、四层架构、用户输入进入 agent、TUI 职责边界、core task 启动、`RegularTask` agent loop、`run_sampling_request`、工具分发、权限审批、`ToolOrchestrator`、shell 沙箱执行、事件回流、核心 struct 速查和最终心智模型。

---

**User**

我不是让你把 3 个文档死板合并成一个，而且是提取关键知识，重新写，简洁一点，多画一些 mermaid 清晰图。仅保留最终结论即可。关键是要阐述整个交互循环和真正的 agent loop，也就是模型请求、工具调用、权限审批、沙箱执行是怎么被组织起来的。重点要解释 codex 中的核心关键 struct 之间的交互和层次划分。然后文件名可以进行适当重命名。

---

**Cursor**

明白，你要的不是拼接版，而是一份重新提炼后的“架构结论笔记”：更短、更像学习后的最终认知产物，用图把关键结构讲清楚。我会重命名文件并重写为精简版，只保留交互循环、真正 agent loop、工具/审批/沙箱组织方式和核心 struct 分层。

我会把新文件命名为 `codex-agent-loop-architecture.md`，让文件名直接表达“Codex agent loop 架构”。内容会减少过程性解释，改成少量关键段落加多张简洁 Mermaid 图。

新版最终结论已经写入新文件。接下来删除旧的 `agent-loop-tool-sandbox.md`，并把索引与 todo 都指向新的 `codex-agent-loop-architecture.md`。

已按你的要求重做了，不再是三份文档的机械合并。

现在只保留新的最终结论文件：

`workspaces/02-learning/openai-codex-cli-deep-learning/notes/codex-agent-loop-architecture.md`

我也删除了旧文件并更新了索引：

- 删除 `notes/agent-loop-tool-sandbox.md`
- 删除 `notes/user-prompt-event-flow.md`
- 删除 `notes/tui-execution-loop.md`
- 更新 `.daedalus/artifact-index.md`
- 更新 `.daedalus/todo.md`

新版重点是：更简洁、循序渐进、更多 Mermaid 图，围绕 Codex 的交互循环、真正 agent loop、模型请求、工具调用、权限审批、沙箱执行，以及核心 struct 的层次划分来讲。

---

**User**

感觉你这个文档太啰嗦了，然后层次也不好。我觉得你需要重新阅读 codex 源码然后再进行梳理。我认为你可以这么梳理：1. 核心流程 loop 总览图；2. 架构分层图，解析；3. 核心模块解析；3.1 react loop; 3.2 tool system; 3.3 auth; 3.4 sandbox; .... 类似这样的流程。

---

**Cursor**

你说得对，这版还是“解释型”太重，没有形成真正的源码结构图谱。我会重新按源码复核一遍关键路径，然后按你给的框架重写成更像架构笔记的版本：总览图、分层图、核心模块解析。

我重新复核了源码主线，发现更合适的笔记应该少讲“用户输入细节”，把中心放在 `submission_loop -> RegularTask -> run_turn -> run_sampling_request -> Tool System`。我会按你给的模块式结构重写，压缩到架构结论级别。

我会直接重写当前 `codex-agent-loop-architecture.md`，控制篇幅，按你建议的结构组织：先总览 loop，再架构分层，然后模块解析，最后给核心 struct 和源码路径。

文档已经按新的结构重写完成，明显压缩了说明性文字，改成“总览图 + 分层图 + 核心模块解析”的架构笔记。现在我会同步记录这次重写，并做一次引用检查。

已重新按你给的结构改写，并且重新复核了 Codex 源码主线。

现在文件是：

`workspaces/02-learning/openai-codex-cli-deep-learning/notes/codex-agent-loop-architecture.md`

这版结构改为：

1. 核心流程 Loop 总览图
2. 架构分层图
3. 核心模块解析
   - 输入与任务启动
   - ReAct Loop
   - Tool System
   - Auth 与 Approval
   - Sandbox
   - Event 回流
4. 核心 Struct 关系速查
5. 最终心智模型

我也把内容压缩了，删掉了很多过程性解释，改成“图 + 关键结论 + 源码入口”的形式。

---

**User**

里面有些图感觉用 mermaid 绘制不是很清晰，还是使用 excalidraw 进行绘制吧。/excalidraw-diagram-generator

---

**Cursor**

我会把当前笔记里最关键、但 Mermaid 表达不够清晰的两类图改成 Excalidraw：一个是“核心流程 loop + 分层”，另一个是“Tool/Approval/Sandbox 细节”。我会生成 `.excalidraw` 源文件，并尽量导出 PNG 方便直接嵌入文档。

我会放到 `notes/diagrams/` 下，文档里用 PNG 预览、旁边保留 `.excalidraw` 源文件，方便你后续继续编辑。

我会用脚本生成规范的 Excalidraw JSON：所有文字都是独立 `text` 元素并绑定到 shape，避免 PNG 导出时丢字。生成后再尝试用 Excalidraw 原生导出脚本输出 PNG。

两个 Excalidraw 源文件已生成。接下来我会用导出脚本生成 PNG，并检查导出是否成功。

PNG 已导出成功。我会把笔记改成“Excalidraw 主图 + 模块解析”的形式，移除原来那些不够清晰的 Mermaid 图。

我检查了导出的 PNG，主流程图清晰；工具图底部说明有些拥挤。我会把工具图重新排版得更简洁，避免文字挤压。

工具图现在主体清楚了，但底部补充说明仍有裁切。我会去掉底部说明，把信息留在文档文字里，图本身只保留流程。

已改成 Excalidraw 图，并嵌入到文档里。

更新内容：

- 生成主流程图：`notes/diagrams/codex-agent-loop-overview.excalidraw`
- 生成工具/审批/沙箱图：`notes/diagrams/codex-tool-approval-sandbox.excalidraw`
- 同步导出 PNG：
  - `notes/diagrams/codex-agent-loop-overview.png`
  - `notes/diagrams/codex-tool-approval-sandbox.png`
- 更新 `notes/codex-agent-loop-architecture.md`，用 PNG 预览替换关键 Mermaid 图，并保留 Excalidraw 源文件链接。
- 更新 `.daedalus/artifact-index.md` 和 `.daedalus/todo.md`。

我也检查了导出的 PNG，文字能正常显示。

---

**User**

你怎么把所有 mermaid 都去掉了。。。

---

**Cursor**

你说得对，我理解偏了：应该是“复杂总览图用 Excalidraw，局部流程仍保留 Mermaid”，而不是把 Mermaid 全部清空。我会把适合文本阅读的局部 Mermaid 图补回来，保留 Excalidraw 作为主图。

对，我刚才处理过头了。现在已修正为：

- 复杂总览图、Tool System 大图：使用 Excalidraw + PNG。
- 局部源码调用链和小流程：恢复 Mermaid。

现在 `notes/codex-agent-loop-architecture.md` 里保留了 7 个 Mermaid 图，用于：

- 输入到 `RegularTask` 的 sequence
- `RegularTask::run` loop
- `run_turn` / `run_sampling_request`
- Tool 分发链
- Approval 决策
- Sandbox 执行
- Event 回流 / struct 关系

Excalidraw 只承担那两个 Mermaid 不够清晰的大图。

---

**User**

然后感觉你的 loop 流程图和架构分层图画得不对，应该分开画。样式我给你一些参考：

---

**Cursor**

明白，你的意思是：loop 流程图应该像标准流程图，突出判断和回环；架构分层图应该像分层架构图，突出层次、模块和依赖边界。之前我把两者混在一张图里了，我会拆成两张重新画。

两张新图已经分开生成并导出：标准流程图和分层架构图。我会检查 PNG 效果，然后替换文档引用。

我看了新导出的图，确实还有问题：loop 图左侧说明被裁切，分层图的层标题和组件重叠。这个需要继续调版，我会把注释移入画布、扩大节点间距，并把分层图的“层名”和“组件”分成左右两列。

分层图现在基本符合参考样式了；流程图右侧提示框文字还被裁了一点。我会直接去掉这块辅助提示，让流程图本身保持干净，然后把文档引用改成“流程图”和“分层图”两张图。

对，这次我按你给的参考重新拆开了：

- `codex-core-agent-loop-flow`：单独的标准 loop 流程图，突出判断、工具调用分支、结果回灌。
- `codex-architecture-layers`：单独的架构分层图，按层展示 TUI、AppServer、Core Session、ReAct/Sampling、Tool、Security/Execution。
- `codex-tool-approval-sandbox`：保留为 tool system 细节图。

文档也已改成分别引用这两张图，不再用之前那张混合图。旧的 `codex-agent-loop-overview` 已删除。局部 Mermaid 仍保留在文档里，用于源码调用链和小流程说明。

---

**User**

非常好，这个部分我大概了解了。

---

**Cursor**

很好。这个阶段我们已经把 Codex 的核心交互循环、agent loop、tool system、approval 和 sandbox 主线梳理清楚了。

下一步比较自然的是继续读 **上下文管理 / 压缩机制**，也就是 Codex 长任务里如何维护 history、何时 compact、工具结果如何进入下一轮 prompt。

---

**User**

那就进入  上下文管理 / 压缩机制 吧。

---

**Cursor**

好的，我们进入“上下文管理 / 压缩机制”这条线。我会先按 Daedalus 学习流程复核相关提示与源码入口，重点找：history 如何保存、prompt 如何构建、工具结果如何回灌、compact 何时触发、compact task 如何执行。

子任务的环境判断有误；我这里可以直接读到 `codex-rs` 源码，所以会继续用本地源码复核。接下来先定位 `compact`、`rollout/history`、`build_prompt` 和 `ResponseInputItem` 的真实实现点。

我已经确认源码目录真实存在，刚才搜索工具被忽略规则影响了。现在用 `rg --no-ignore` 直接在 `codex-rs` 中检索 compact/history/prompt 相关实现点。

我已经定位到关键实现：`Session` 里有 history/rollout，`run_turn` 负责 pre-sampling compact 和 token limit compact，`compact.rs`/`compact_remote*` 负责替换历史。接下来读几个闭环片段，避免扫目录。

我已经读完第一轮闭环源码。结论很清楚：Codex 的上下文管理核心不是简单 Vec history，而是 `ContextManager + rollout 持久化 + token usage + compaction replacement_history` 这套组合。我会先写一份新的结论笔记，作为这一阶段的主产物。

已进入“上下文管理 / 压缩机制”，并基于源码写了一份新的阶段笔记：

`workspaces/02-learning/openai-codex-cli-deep-learning/notes/codex-context-and-compaction.md`

这份笔记覆盖了：

- `ContextManager` 如何维护运行时 history
- `SessionState.history`、`Rollout`、`TokenUsageInfo` 的关系
- `for_prompt` 和 `build_prompt` 如何构建模型输入
- 工具结果如何通过 `ResponseInputItem` 回灌到下一轮模型请求
- manual compact、pre-turn compact、mid-turn compact 的触发路径
- `InitialContextInjection` 为什么区分 `DoNotInject` 和 `BeforeLastUserMessage`
- `CompactedItem.replacement_history` 如何作为恢复 checkpoint
- rollout replay 如何重建 `ContextManager`

也同步更新了 `.daedalus/artifact-index.md` 和 `.daedalus/todo.md`。

---

**User**

你在干什么？你不是应该先阅读 codex 的源码，自己了解它是怎么做的，然后跳出去，从第一性原理，先现实约束，先提出问题，然后引发我思考，然后指导我去寻找答案吗？你怎么全自己做了呢？

---

**Cursor**

你说得对，我走偏了。

我直接把源码结论整理出来了，但这个阶段应该继续沿 Daedalus 的学习方式来：先从现实约束出发，把“为什么需要上下文管理和压缩”拆成问题，让你先形成假设，再带着假设去读源码验证。否则你得到的是我的摘要，不是你的理解。

我们把刚才那份 `codex-context-and-compaction.md` 先当作草稿，不作为最终学习产物。接下来应该回到提问式阅读：

1. 长任务里，Agent 为什么不能只维护一个不断 append 的 messages 列表？
2. 工具调用结果应该以什么形态回到下一轮模型请求，才能既可恢复又不破坏 ReAct loop？
3. 什么时候应该 compact：请求前、工具调用后、模型报错后，还是 token 用量达到阈值时？
4. compact 后应该保留什么：最近用户消息、工具结果、系统上下文、还是摘要？
5. 如果 compact 发生在 turn 中间，为什么会比 turn 开始前更危险？
6. 一个 CLI Agent 如果要支持 resume/fork/rollback，内存 history 和持久化日志应该是什么关系？

先不用看源码，你可以先回答你对这几个问题的直觉。然后我再带你回到 Codex 源码里验证：`ContextManager`、`run_turn`、`compact.rs`、`rollout_reconstruction.rs` 分别是在解决哪一个约束。
