# 2026-06-27 Chapter 1/5 Task Lab Sweep 路线调整

### 用户判断

用户希望先把 Hugging Face LLM Course Chapter 1/5 `How Transformers solve tasks` 这一节的 task labs 都完成，再进入 Chapter 2 `Behind the pipeline`。

用户的理由是：这一节覆盖了大模型多个方面的能力；虽然会有很多重复，但对初学者来说，前期机械式重复有助于建立手感。

### 当前结论

接受这个调整。当前阶段机械重复不是问题，前提是每次重复都要抽取同一张结构图：

```text
raw input
  -> tokenizer / processor
  -> model
  -> task-specific head / decoder
  -> logits / generated ids / spans
  -> loss / metric / postprocess
  -> human-readable output
```

Chapter 2 `Behind the pipeline` 暂时后移。先完成 Chapter 1/5 task sweep，再回头拆 pipeline，会更容易理解 pipeline 是如何随 task 改变 processor、model class、head 和 postprocess 的。

官方页面中对应 `Ready to try your hand...` 的 task labs 一共 8 个：

1. Text generation / causal language modeling
2. Text classification
3. Token classification
4. Question answering
5. Summarization
6. Translation
7. Automatic speech recognition
8. Image classification

### 下一步动作

- 新增 guide：[`../../guides/04-lesson-lab/05-chapter1-5-task-lab-sweep.md`](../../guides/04-lesson-lab/05-chapter1-5-task-lab-sweep.md)。
- 下一小步：Token Classification lab。
- 每个 lab 只追求小步跑通和 shape/输出观察，不追求完整训练效果。
