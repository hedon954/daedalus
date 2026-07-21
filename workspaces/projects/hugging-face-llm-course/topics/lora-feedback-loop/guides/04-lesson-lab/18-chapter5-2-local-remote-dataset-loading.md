# Chapter 5/2：本地与远程数据加载排障

## Lesson Lab

- Lesson：用 `load_dataset("json", data_files=..., field="data")` 加载 SQuAD-it。
- 本节概念目标：区分 loader format、数据来源、split mapping 与嵌套字段入口。
- 用户先预测：同一组文件分别从 GitHub URL、最终 raw URL 和本地 `.json.gz` 加载时，应得到相同的 `DatasetDict`。
- 必须打印：实际 URL 的 `repr()`、Python 可执行文件、`datasets` 版本、split、rows 与 columns。
- Agent 实测：三种来源都得到 `train=442`、`test=48`；这不是用户已经完成的观察证明。
- 下一迁移：换一个远程 CSV 或 JSON 数据集，独立验证 format、source、split 与 schema。

## 这次 `FileNotFoundError` 说明了什么

用户遇到：

```text
FileNotFoundError: Unable to find 'https://github.com/.../SQuAD_it-train.json.gz'
```

这个异常发生在 `load_dataset()` 创建 builder、解析 `data_files` 的阶段，还没有进入 JSON 内容解析。因此它表达的是“当前这次 URL 探测没有把输入解析成可用文件”，不等于服务器上的文件永久不存在。

课程原代码仍在官方页面中。2026-07-21 使用 traceback 指向的同一个项目 `.venv` 重新运行时，以下三种输入均成功：

```text
GitHub /raw/ URL       -> train 442, test 48
raw.githubusercontent -> train 442, test 48
local .json.gz         -> train 442, test 48
```

所以当前证据不支持“Datasets API 已经变了”或“该链接失效”。更符合证据的判断是：先前那次远程文件探测受到了临时网络响应、GitHub 跳转链路或 notebook 内核现场的影响。

另外，聊天中显示的 `[URL](URL)` 是界面自动链接化；检查 notebook 后，实际变量中保存的是纯 URL，不是 Markdown 字符串。

## 为什么浏览器或 `wget` 能下载，loader 仍可能失败

它们不是同一条调用链：

```text
浏览器 / wget
  -> HTTP 请求
  -> 跟随 GitHub 重定向
  -> 下载字节

load_dataset
  -> 识别 json loader
  -> 解析 data_files
  -> 远程文件存在性与路径探测
  -> 下载和解压
  -> 从 field="data" 构建 Arrow dataset
```

浏览器成功只证明 URL 在那次请求中可下载，不保证 loader 的预检在另一次请求中一定成功。

## 推荐恢复顺序

先原样重跑；如果仍失败，打印真实运行现场：

```python
import sys
import datasets

print(sys.executable)
print(datasets.__version__)
print(repr(data_files["train"]))
```

然后改用不经过 GitHub 页面跳转的最终 raw 地址：

```python
from datasets import load_dataset

url = "https://raw.githubusercontent.com/crux82/squad-it/master/"
data_files = {
    "train": url + "SQuAD_it-train.json.gz",
    "test": url + "SQuAD_it-test.json.gz",
}
squad_it_dataset = load_dataset("json", data_files=data_files, field="data")
```

如果远程链路仍不稳定，就使用已经下载的压缩文件；Datasets 会自动解压：

```python
data_files = {
    "train": "SQuAD_it-train.json.gz",
    "test": "SQuAD_it-test.json.gz",
}
squad_it_dataset = load_dataset("json", data_files=data_files, field="data")
```

最后检查数据契约：

```python
print(squad_it_dataset)
print(squad_it_dataset["train"].column_names)
print(squad_it_dataset["train"][0].keys())
```

预期是两个 split，行数分别为 442 和 48，顶层列为 `title`、`paragraphs`。`field="data"` 的作用是进入原始 JSON 顶层的 `data` 数组，而不是选择 DatasetDict 的 split。

## 来源

- [Hugging Face LLM Course Chapter 5/2](https://huggingface.co/learn/llm-course/en/chapter5/2)
- [Hugging Face Datasets：Load](https://huggingface.co/docs/datasets/loading)
