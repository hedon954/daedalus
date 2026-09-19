# 知识候选表

> 状态只能使用：`候选中`、`总结中`、`已归档`、`已忽略`。

## 主题核心

| 候选 | 为什么值得看 | 证据 | 状态 |
| --- | --- | --- | --- |
| SFT 是条件行为分布的重塑，而不是 JSON 记忆 | 能把 causal LM、teacher forcing、loss mask 与 suggestion policy distillation 串成同一机制；适合作为 SFT 系列的第一篇核心博文。 | `guides/08-review-loop/01-sft-blog-series-mastery-plan.md`；真实 Qwen3 SFT 事实边界 | 候选中 |
| Full SFT 与 LoRA 的选择是优化自由度问题 | 成本只是表层；真正需要讨论数据约束、Base 能力距离、低秩更新、泛化和部署形态，并用公平实验而非单次分数支撑。 | Qwen3 1.7B / 27k Full 与 LoRA 用户确认实践；博文 3 验收计划 | 候选中 |
| Suggestion List SFT 的数据资产是 latent policy | Plan3/Plan0、action 分布、eligibility、组合多样性和 decision boundary 共同决定 student learnability，比样本总数更重要。 | `guides/06-practice-transfer/` 四版 schema；博文 4 验收计划 | 候选中 |
| 评测与反馈闭环必须分离模型质量和系统路由质量 | eligible、invoked、generation 与 end-to-end 指标分层，才能避免 middleware skip 或 Judge 漂移被误归因给模型。 | 博文 5、6 问题库；真实评测细节待脱敏归档 | 候选中 |

## 底层原理

| 候选 | 为什么值得看 | 证据 | 状态 |
| --- | --- | --- | --- |
| 原始数据正确不等于模型训练输入正确 | Chat template、tokenizer、truncation、EOS、labels 和 loss mask 之后的 `(input_ids, labels)` 才是最终事实源；应固化为 Dataset Preflight。 | tokenizer/LM block lesson evidence；历史 SFT 排障线索；博文 2 | 候选中 |
| SFT 与 GRPO 使用不同反馈信号 | SFT 提高示范 token 的似然；GRPO 对模型采样结果做组内相对奖励优化。理解边界可以防止“有 Judge 就上 RL”的错误迁移。 | 用户确认 Chapter 11、12 已读；博文 8 待闭卷验证 | 候选中 |

## 工程模式

| 候选 | 为什么值得看 | 证据 | 状态 |
| --- | --- | --- | --- |
| 单仓库多 Python 项目的 IDE 解析边界 | daedalus 作为总仓库会包含多个 topic/demo Python 项目；IDE 解析边界不能只靠“文件夹里有 pyproject/venv”自动推断。直接在根 `.vscode/settings.json` 里绑定某个 demo 的 interpreter，会把根 workspace 误建模为单一 Python project；但只依赖 demo 局部 `.vscode/settings.json`，在用户直接打开 daedalus folder 时又不会生效。更稳的模式是“双路径”：推荐用 multi-root `.code-workspace` 把 daedalus 根和每个 Python demo 都建成独立 workspace folder；同时保留根 `pyrightconfig.json` 作为直接打开仓库根时的静态解析兜底，显式为当前 demo 配置 execution environment 和 `.venv/site-packages`。 | 本 topic 的 `hugging-face-course-learning` demo 中 `.venv` 已安装 `transformers`。第一次直接打开 daedalus 根时，Pylance/BasedPyright 报 `reportMissingImports`；将 Python interpreter 从根 `.vscode/settings.json` 移除后避免了根 workspace 被某个 demo 绑死，但直接打开 daedalus folder 时 demo 局部 `.vscode/settings.json` 不生效，Rust 根配置也需要保留。最终配置分工为：根 `.vscode/settings.json` 只保留 Rust fallback；`daedalus.code-workspace` 作为 multi-root 入口；demo 局部 `.vscode/settings.json` 指向自己的 `.venv`；根 `pyrightconfig.json` 为直接打开根目录时显式添加 demo root 和 `.venv/lib/python3.11/site-packages`。 | 候选中 |
| 动态库 API alias 与静态类型 overload 的差异 | Hugging Face Transformers 这类动态 Python 库运行时会支持 alias 和宽松参数，但类型检查器读取的是 overload / Literal 签名；学习 demo 如果照抄 runtime alias，可能出现“运行成功但编辑器报错”。处理原则不是盲目关闭类型检查，而是优先使用 canonical API 名称，让教学代码同时满足运行时和静态分析。 | `transformers.pipeline("sentiment-analysis")` 能运行成功，因为 `sentiment-analysis` 是 `text-classification` 的 runtime alias；但 Transformers 5.12.0 的 overload 列表只包含 `Literal["text-classification"]`，因此 BasedPyright 报 `reportCallIssue`。将 demo 改为 `pipeline("text-classification")` 后仍输出相同 POSITIVE 结果，并更贴合类型签名。 | 候选中 |
