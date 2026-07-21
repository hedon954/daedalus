# 知识候选表

> 状态只能使用：`候选中`、`总结中`、`已归档`、`已忽略`。

## 主题核心

| 候选 | 为什么值得看 | 证据 | 状态 |
| --- | --- | --- | --- |
|  |  |  | 候选中 |

## 底层原理

| 候选 | 为什么值得看 | 证据 | 状态 |
| --- | --- | --- | --- |
|  |  |  | 候选中 |

## 工程模式

| 候选 | 为什么值得看 | 证据 | 状态 |
| --- | --- | --- | --- |
| 单仓库多 Python 项目的 IDE 解析边界 | daedalus 作为总仓库会包含多个 topic/demo Python 项目；IDE 解析边界不能只靠“文件夹里有 pyproject/venv”自动推断。直接在根 `.vscode/settings.json` 里绑定某个 demo 的 interpreter，会把根 workspace 误建模为单一 Python project；但只依赖 demo 局部 `.vscode/settings.json`，在用户直接打开 daedalus folder 时又不会生效。更稳的模式是“双路径”：推荐用 multi-root `.code-workspace` 把 daedalus 根和每个 Python demo 都建成独立 workspace folder；同时保留根 `pyrightconfig.json` 作为直接打开仓库根时的静态解析兜底，显式为当前 demo 配置 execution environment 和 `.venv/site-packages`。 | 本 topic 的 `hugging-face-course-learning` demo 中 `.venv` 已安装 `transformers`。第一次直接打开 daedalus 根时，Pylance/BasedPyright 报 `reportMissingImports`；将 Python interpreter 从根 `.vscode/settings.json` 移除后避免了根 workspace 被某个 demo 绑死，但直接打开 daedalus folder 时 demo 局部 `.vscode/settings.json` 不生效，Rust 根配置也需要保留。最终配置分工为：根 `.vscode/settings.json` 只保留 Rust fallback；`daedalus.code-workspace` 作为 multi-root 入口；demo 局部 `.vscode/settings.json` 指向自己的 `.venv`；根 `pyrightconfig.json` 为直接打开根目录时显式添加 demo root 和 `.venv/lib/python3.11/site-packages`。 | 候选中 |
| 动态库 API alias 与静态类型 overload 的差异 | Hugging Face Transformers 这类动态 Python 库运行时会支持 alias 和宽松参数，但类型检查器读取的是 overload / Literal 签名；学习 demo 如果照抄 runtime alias，可能出现“运行成功但编辑器报错”。处理原则不是盲目关闭类型检查，而是优先使用 canonical API 名称，让教学代码同时满足运行时和静态分析。 | `transformers.pipeline("sentiment-analysis")` 能运行成功，因为 `sentiment-analysis` 是 `text-classification` 的 runtime alias；但 Transformers 5.12.0 的 overload 列表只包含 `Literal["text-classification"]`，因此 BasedPyright 报 `reportCallIssue`。将 demo 改为 `pipeline("text-classification")` 后仍输出相同 POSITIVE 结果，并更贴合类型签名。 | 候选中 |
