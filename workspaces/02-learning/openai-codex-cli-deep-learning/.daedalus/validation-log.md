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
