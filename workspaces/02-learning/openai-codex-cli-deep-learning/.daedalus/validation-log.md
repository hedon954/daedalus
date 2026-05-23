# Daedalus 验证日志

记录本学习任务对 daedalus 教学引导能力的反向验证结果。

## 使用规则

- 不要把普通学习笔记写到这里；学习笔记写入 [`notes/`](../notes)。
- 这里只记录 daedalus 本身的引导效果、缺口和改进建议。
- 每完成一个阶段，至少复盘一次“Agent 是否代替用户做太多”。

## 2026-05-09 15:58:52

- 阶段：`01-goal-aligner`
- 有效引导：通过 3 个结构化问题将“深入学习 Codex 源码”收敛为 OpenAI Codex CLI、Agent loop/沙箱权限/整体架构、runbook+笔记+mini demo。
- 用户亲自完成的实践：用户选择了学习对象、学习重点和输出形态。
- Agent 代替用户过多的地方：Agent 代写了任务卡初稿和指南；后续需要用户确认是否接受运行风险，并亲自执行或批准源码拉取。
- Prompt/template/CLI/docs 改进建议：`source/pull_source.sh` 默认 `PINNED_REF=main` 便于启动，但正式学习前应固定 commit。
- 是否足以进入下一阶段：目标已经足以进入 `02-repo-scout`；完成 repo 选择前还需要用户确认风险接受和源码拉取方式。

## 2026-05-09 16:00:31

- 阶段：`02-repo-scout`
- 有效引导：对用户指定的 `openai/codex` 做了目标匹配、学习密度、运行风险和 mini demo 可能性评估，并将选择证据写入 `notes/repo-selection.md`。
- 用户亲自完成的实践：用户确认由 Agent 使用 `source/pull_source.sh` 拉取源码，并选择“先用 main 拉取，再记录当前 commit”。
- Agent 代替用户过多的地方：Agent 代为执行了源码拉取；该行为经过用户明确确认，且通过脚本执行并记录 commit。
- Prompt/template/CLI/docs 改进建议：模板脚本初始没有可执行位，直接 `./pull_source.sh` 会失败；本任务已对该脚本补充可执行位并复验通过，后续可考虑让模板保留可执行位或在 README 中说明调用方式。
- 是否足以进入下一阶段：已足以进入 `03-socratic-coach`，下一步应生成问题路线图。

## 2026-05-09 16:18:00

- 阶段：`03-socratic-coach`
- 有效引导：用户将 Codex 学习问题收敛为三类失败点：代码修改遗漏、语法/运行错误、长上下文丢失；并提出 CLI、agent 构建、LLM adapter、ReAct loop、权限、tool gateway、hooks 的核心链路假设。
- 用户亲自完成的实践：用户回答了第一轮 3 个问题，且给出了 mini demo 必须保留的不变量：会话、工具体系、审核、沙箱、事件流、上下文压缩。
- Agent 代替用户过多的地方：Agent 只负责将回答整理为可验证假设；没有把源码结论提前写死。
- Prompt/template/CLI/docs 改进建议：问题设计有效，但后续需要明确区分“用户猜测”和“源码验证结论”，避免把 master-sub agents、多 provider adapter、REPL pause 等假设提前当成事实。
- 是否足以进入下一阶段：足以进入 `04-debugger-guide`，优先用 `codex exec` 建立可复现运行与调用链追踪。

## 2026-05-09 16:26:00

- 阶段：`04-debugger-guide`
- 有效引导：runbook 将最小验证收敛为 `codex --help`、`codex exec --help` 和 `cargo test -p codex-exec`，用户执行后确认入口可构建、非交互入口存在、`codex-exec` 测试通过。
- 用户亲自完成的实践：用户亲自执行三条命令，并反馈 `codex-exec` 测试结果为 61 passed、0 failed。
- Agent 代替用户过多的地方：Agent 没有代跑命令，只整理用户输出并形成阶段结论。
- Prompt/template/CLI/docs 改进建议：runbook 下一版可要求用户保存完整 help 输出，避免命令清单被截断导致后续引用缺失。
- 是否足以进入下一步：足以继续在 `04-debugger-guide` 内追踪源码调用链；真实模型调用、认证和工具执行尚未验证。

## 2026-05-10 13:15:00

- 阶段：`05-arch-analyzer` / `06-code-reader`
- 有效引导：用户先提出自己对 agent loop、工具权限、上下文压缩和 rollout 恢复的理解，Agent 再用源码校准，并将结论沉淀到 `notes/codex-agent-loop-architecture.md` 和 `notes/codex-context-and-compaction.md`。
- 用户亲自完成的实践：用户阅读源码片段，梳理 `run_pre_sampling_compact`、mid-turn compact、compact prompt、`ToolOutput::to_response_item`、`drain_in_flight`、`reconstruct_history_from_rollout` 等机制，并提出 future 顺序保证问题。
- Agent 代替用户过多的地方：进入上下文管理专题时，Agent 曾先直接生成总结，违背“先现实约束、再问题、再源码验证”的教练原则；已把“问题 -> 用户回答 -> Agent 校准 -> 验证路径 -> 验证状态”写入公共 prompt 和 repo-learning skill。
- Prompt/template/CLI/docs 改进建议：阶段产物命名与 state 模板不完全一致，例如架构产物实际为 `notes/codex-agent-loop-architecture.md`，而 state 模板期望 `notes/architecture.md`；后续可考虑允许 artifact-index 中声明等价产物。
- 是否足以进入下一步：当前足以保持在 `06-code-reader`，继续选择一个专题深读，或开始提炼 mini demo 不变量。

## 2026-05-16

- 阶段：`06-code-reader`
- 有效引导：用户指出当前学习缺少“以终为始”的路径感，容易陷入无穷源码细节；本次将 repo learning 协议重构为 outcome-map 驱动，并为当前任务新增 `outcome-map.md` 与 `demo/design.md` 草案。
- 用户亲自完成的实践：用户明确提出最终产物、当前位置、障碍和解锁关系必须持续可见，并给出一套可执行的重构方案。
- Agent 代替用户过多的地方：此前 Agent 容易把“下一步源码点”当成学习目标本身，而没有持续说明它服务哪个最终产物。
- Prompt/template/CLI/docs 改进建议：`repo-learning-coach`、`06-code-reader`、`07-demo-architecture`、`resume.md`、`todo.md` 模板和 workspace 校验都应围绕 outcome map 工作。
- 是否足以进入下一步：足以继续留在 `06-code-reader`，但只能补 `outcome-map.md` 中列出的 3 个 demo 缺口；补完后必须进入 `07-demo-architecture`。

## 2026-05-16 目录化复盘

- 阶段：`06-code-reader`
- 有效引导：用户指出 `guides/` 和 `notes/` 不应每阶段挤在一个 markdown 文件中；本次将 active 阶段增量迁移为 `guides/06-code-reader/README.md` 与 `notes/06-code-reader/README.md`。
- 用户亲自完成的实践：用户提出阶段内部也需要可导航的信息架构，避免单文件越来越长。
- Agent 代替用户过多的地方：历史上 Agent 容易把多个专题堆入 `notes/code-reading.md`，降低恢复和定位质量。
- Prompt/template/CLI/docs 改进建议：后续新任务应使用 `guides/<stage-id>/README.md` 和 `notes/<stage-id>/README.md` 作为阶段入口，专题拆到同目录文件。
- 是否足以进入下一步：足以继续 Runtime request assembly；后续新内容应写入 `guides/06-code-reader/` 和 `notes/06-code-reader/`。

## 2026-05-16 Orchestrator retry 复盘

- 阶段：`06-code-reader`
- 有效引导：用户先复述 sandbox denied 后的 retry 分支，Agent 再按源码校准 `AskForApproval::Never` 和 `already_approved` 的含义，避免把运行时 retry 逻辑简化成“失败后问不问用户”。
- 用户亲自完成的实践：用户定位到 `ToolOrchestrator::run` 的 sandbox denied 分支，概括出 network approval context、approval policy gate、retry reason、approval 后 second attempt 四个关键步骤。
- Agent 代替用户过多的地方：本轮没有先给结论；Agent 只做源码核对和边界校准。
- Prompt/template/CLI/docs 改进建议：`06-code-reader` 的 outcome gate 有效，三个缺口补齐后应主动进入 `07-demo-architecture`，不能继续开放式阅读权限系统。
- 是否足以进入下一步：足以进入 `07-demo-architecture`，定稿 `demo/design.md`。

## 2026-05-16 08 实现阶段越界复盘

- 阶段：`08-demo-coder`
- 有效引导：`07-demo-architecture` 已形成清晰的 Phase 1/Phase 2 蓝图和 08 子地图，用户确认可以继续进入实现阶段。
- 用户亲自完成的实践：用户指出 Agent 不应在 08 阶段直接代写代码，而应该一步步带用户实现。
- Agent 代替用户过多的地方：Agent 将“继续”误解为允许直接编码，擅自创建 demo crate、README 和领域模型代码，违背 repo learning 的练习边界。
- Prompt/template/CLI/docs 改进建议：已补强 `repo-learning-coach`、`08-demo-coder` 和 `coach-questioning`，新增 Implementation Practice Gate：08 默认只给行动卡和验证目标，除非用户明确说“你来实现/代写/apply patch”，否则不得编辑实现文件。
- 是否足以进入下一步：实现代码已回滚；任务仍处于 `08-demo-coder`，下一步应输出 Slice 0 行动卡，等待用户亲手创建工程骨架。

## 2026-05-21 Slice 6 LLM streaming adapter 检查点

- 阶段：`08-demo-coder` / Slice 6 Agent Orchestrator。
- 有效引导：用户决定从 scripted model 改为真实 OpenAI-compatible streaming model，当前已新增 `agent::llm`、`OpenAiCompatibleLlm`、`StreamEvent` 和 Rust/OpenAI integration guide。
- 用户亲自完成的实践：用户实现了 `async_stream::try_stream!` 包装 `reqwest::bytes_stream()` 的流式 adapter，并通过真实 raw stream 样本摸索 Chat Completions chunk 中的 `content`、`reasoning_content` 和 `tool_calls` 字段。
- Agent 代替用户过多的地方：Agent 只做 review、指出边界和 parser 风险；未直接修改 LLM adapter 实现。
- 当前验证：`cargo test --manifest-path demo/Cargo.toml` 通过，结果为 24 passed、2 ignored；ignored 测试为真实联网 LLM 调用。
- 当前待解决：guide 已收敛为 DeepSeek / OpenAI-compatible Chat Completions stream。`take_sse_events` 目前有意限定为常见 `data: ...` stream，不作为当前 blocker，但需要 recorded fixture 单测锁定边界；`Default` 缺 env 时 panic；parser 和 tool-call 聚合缺少默认 fixture 单测；orchestrator 尚未串起 approval / sandbox / retry。
- 是否足以进入下一步：可以提交当前检查点，但 Slice 6 不能标完成。下一步应先补 parser / map_chunk fixture tests，再接 orchestrator。
