# 深入学习 codex 源码
_Exported on 2026/5/9 at GMT+8 15:40:54 from Cursor (3.3.27)_

---

**User**

我想要对 codex 的源码进行深入学习。

---

**Cursor**

我理解你想把 Codex 源码作为一次系统的深度学习任务来推进。我先读取项目里的 repo learning 工作流说明，然后会帮你把目标、选仓库、学习路线和产出形式落到 daedalus 的文件化流程里。

我会先按第 1 阶段处理：确认这次学习的现实目标和验收标准，同时检查当前 daedalus 是否已有一个正在进行的学习任务，避免把 WIP 弄乱。

当前 `workspaces/02-learning` 里没有活跃学习任务，可以开启一个 Codex 源码学习任务；我还需要补齐三个关键信息，才能把它变成可执行的任务卡。

**对齐 Codex 源码学习目标**

> **你说的 Codex 源码具体指哪一个学习对象？**
> - OpenAI Codex / Codex CLI 的当前开源仓库 ✔
> - 我本地已经有一个 Codex 源码目录，接下来给你路径
> - 还不确定，先帮我选最适合深读的 Codex 相关仓库

> **这次深入学习最想服务于哪个现实目标？**
> - 理解并实现自己的 coding agent / CLI agent
> - 能够改造或扩展 Codex 的工具、沙箱、补丁、上下文机制
> - 把 Codex 的设计迁移到 daedalus 的学习/执行工作流里
> - 先建立系统理解，后续再决定业务应用
>
> Answer: 学习 codex 这类优秀的 coding agent 的实现原理，理解它是如何解决现实中的各种问题，如何在 prompt engineering, context engineering 和 harness engineering 各个方面进行优秀设计，从而满足负责多变且具备不确定性的 coding 环境。从而为我在工作中构建各类真实业务的 ai agent 提供指导。

> **你希望第一轮学习交付哪些产物？**
> - 学习任务卡与阶段路线图 ✔
> - 本地运行/调试 runbook ✔
> - 架构图与核心路径说明 ✔
> - 核心代码逐段阅读笔记 ✔
> - 复刻核心机制的 mini demo ✔
> - 可归档到 knowledge-base 的知识条目 ✔

目标已经足够清晰：这不是泛泛读源码，而是围绕 coding agent 在 prompt、context、harness 三层如何应对真实复杂环境来学习，并产出 runbook、架构笔记、代码阅读笔记、mini demo 和知识归档。我接下来会查看 daedalus 现有任务模板和 CLI，用项目约定创建一个可恢复的 active learning 任务。

我会用项目自己的 `daedalus init repo-learning` 来创建任务目录，然后再填充任务卡和首轮路线图。

任务目录已经创建成功。接下来我会把你刚才给出的学习目标写入任务卡，并把 todo 调整成 Codex 源码学习的第一轮可执行计划。

我现在会更新 `.daedalus/task-card.md`、todo、长期上下文和产物索引，把这个学习任务从“空模板”变成可恢复的 Codex 学习计划。

任务卡和配套上下文已经写入；我会运行 daedalus 校验，并在校验通过后把第 1 阶段标记为完成，进入选仓库阶段。

第 1 阶段已完成并进入 `02-repo-scout`。我会确认 OpenAI Codex 仓库的当前入口信息，然后记录选仓库理由，并准备把源码放到任务的 `source/` 目录。

我会把官方仓库固定到一个具体提交，避免后续笔记漂移；然后把 shallow clone 放进当前学习任务的 `source/codex`。

源码已经克隆到任务目录，并固定了当前 `HEAD`：`ebe75bb683b3c237aad9f039ab17b187048aa499`。我会快速查看仓库结构，确保选仓库说明不是只停留在链接层面。

我已经确认官方仓库是 `openai/codex`，本地源码也已放入任务 `source/codex`。现在会补一份 `repo-selection.md`，说明为什么它是首选，以及第一批阅读入口。

仓库选择说明已补齐；我会完成 `02-repo-scout` 并进入问题路线图阶段，然后用一次只读探索来定位第一批核心阅读路径。

只读探索已经给出一条很清晰的主线：从 `cli/tui/exec` 入口进入 `core` 的任务与 turn，再分叉到 context/prompt 与 tools/sandbox/apply-patch。接下来我会把它整理成递进问题路线图，而不是直接开始散读代码。

问题路线图已经写入，并把下一步明确为 run/debug。现在我会校验任务状态，完成 `03-socratic-coach`，进入本地运行与核心链路追踪阶段。

进入 run/debug 后，我先做低成本验证：读取构建说明和 Cargo 配置，确认最小可运行命令；完整构建可能较久，先不把学习卡在全量编译上。

`cargo check -p codex-cli --bin codex` 已开始，当前正在同步 Rust 1.93 工具链和依赖，这一步可能比较久。我先把已验证的启动方式、依赖和核心断点建议写入 runbook，构建结果回来后再补状态。

构建检查仍在跑，输出已经增长到依赖编译阶段。我会看一下当前进度，确认不是认证或配置类失败。

构建检查通过了：`cargo check -p codex-cli --bin codex` 用时约 3 分 48 秒，退出码 0。下一步我会跑一个不需要登录的 `--help`，确认 CLI 二进制入口可执行。

`--help` 触发了真正的编译而不只是 check，所以也比较久；这不是运行失败，而是在生成可执行二进制。我会继续等一段时间拿到最终结果。

`codex --help` 也成功了，证明本地源码可以生成并启动 CLI；帮助信息列出了 `exec`、`review`、`mcp`、`mcp-server`、`app-server`、`sandbox`、`apply`、`resume`、`exec-server` 等关键子命令。接下来我会把这个验证结果补进 runbook。

校验发现模板要求 `notes/.gitkeep` 和 `source/.gitkeep` 仍然存在；新增笔记和源码目录后这两个占位文件缺失了。我会补回占位文件，再重新校验。

已为 Codex 源码学习创建并推进了 daedalus active learning 任务：

`workspaces/02-learning/codex-coding-agent-source-learning`

已完成：
- 创建任务卡：`.daedalus/task-card.md`
- 选择并克隆官方仓库：`source/codex`，pinned HEAD `ebe75bb683b3c237aad9f039ab17b187048aa499`
- 写入仓库选择说明：`notes/repo-selection.md`
- 写入递进问题路线图：`notes/question-roadmap.md`
- 写入初版运行调试手册：`notes/runbook.md`
- 阶段已推进到 `04-debugger-guide`

验证结果：
- `daedalus validate` 通过
- `cargo check -p codex-cli --bin codex` 通过，用时约 3 分 48 秒
- `cargo run -p codex-cli --bin codex -- --help` 通过，用时约 3 分 30 秒

当前下一步是追踪第一条核心链路：

`cli/tui/exec -> core/tasks/regular.rs -> core/session/turn.rs -> core/tools/router.rs`

另外，当前 git 里还有一些我没改的已有变更：`crates/docs/instructions.md`、`crates/docs/instructions/` 和 `.cursor/plans/...`。
