# 2026-06-16 Transformers Pipeline Smoke Test

### 运行目标

确认 Hugging Face Transformers 的最小 pipeline 能在当前 demo 环境中运行，并验证 Cursor / Pyright 配置与运行时环境的差异。

### 运行环境

- Demo：`demo/hugging-face-course-learning`
- Python env：`.venv`
- 运行方式：`uv run transformer-lib/pipeline.py`
- 关键依赖：`transformers==5.12.0`，`torch>=2.12.0`

### 运行结果

用户已跑通 sentiment/text classification pipeline。

观察到的输出：

```text
[{'label': 'POSITIVE', 'score': 0.9982948899269104}]
```

### 重要现象

1. `pipeline("sentiment-analysis")` 运行时可成功，但 BasedPyright 报 `reportCallIssue`。
2. 原因是 Transformers runtime 支持 `sentiment-analysis` alias，但类型 overload 中 canonical task 是 `text-classification`。
3. demo 已改为 `pipeline("text-classification")`，保持运行结果等价，同时减少静态类型误报。
4. 运行时出现 Hugging Face Hub unauthenticated warning；当前 smoke test 可忽略，后续大量下载或 gated model 再配置 `HF_TOKEN`。

### 当前结论

第一个 HF pipeline 已跑通。下一步不急着进入 LoRA，而是要把 pipeline 的机制拆开：

- task name 如何映射到默认 model。
- model/tokenizer/config 如何被下载和缓存。
- pipeline 输入如何经过 tokenizer、model、postprocess。
- 为什么不指定 model 在生产中不推荐。

### 下一步动作

- 把当前脚本从“能跑”改成“可观察”：打印 pipeline 的 task、model name、tokenizer、config、缓存/下载行为。
- 固定 model name，避免依赖 Transformers 默认模型选择。
- 将运行命令和输出沉淀到 demo README 或 runbook。
