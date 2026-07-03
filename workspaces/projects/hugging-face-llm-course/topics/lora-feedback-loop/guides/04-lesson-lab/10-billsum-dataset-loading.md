# BillSum Dataset Loading：短名、split 和 notebook 变量

## 数据集 ID

课程旧代码可能写成：

```python
billsum = load_dataset("billsum", split="ca_test")
```

在当前环境里会触发：

```text
HfUriError: Invalid HF URI 'hf://datasets/billsum@...'
Repository id must be 'namespace/name', got 'billsum'.
```

这和前面 SQuAD 的短名问题同类：Hub repo id 需要显式 namespace。BillSum 在 Hugging Face Hub 上的完整 dataset id 是：

```python
billsum = load_dataset("FiscalNote/billsum", split="ca_test")
```

已用本地 `datasets` API 验证：

```text
configs: ['default']
splits: ['train', 'test', 'ca_test']
```

## `split` 参数的作用

`split` 用来选择要加载的数据分区。它不是现场随机切分数据，而是从数据集 repo 已经定义好的 split 中取一份。

对 BillSum 来说：

```python
load_dataset("FiscalNote/billsum")
```

会返回一个 `DatasetDict`，里面有 `train`、`test`、`ca_test` 三个 split。

```python
load_dataset("FiscalNote/billsum", split="ca_test")
```

只返回 `ca_test` 这一份 `Dataset`。这里的 `ca_test` 是 California bills 的测试分区，课程用它通常是因为它比完整训练集小，适合先观察 summarization 数据结构和流程。

`split` 也支持切片，用来做小样本观察：

```python
load_dataset("FiscalNote/billsum", split="train[:100]")
load_dataset("FiscalNote/billsum", split="ca_test[:10%]")
```

## Notebook 重跑时为什么用 `billsum_raw`

`billsum_raw` 不是特殊变量名。它的作用是保留“还没有被再次切分”的原始 `Dataset`，避免 notebook 反复运行时把变量覆盖成另一种类型。

推荐写法：

```python
billsum_raw = load_dataset("FiscalNote/billsum", split="ca_test")
billsum = billsum_raw.train_test_split(test_size=0.2)
```

这里两个变量的类型不同：

```text
billsum_raw: Dataset
billsum: DatasetDict, with train/test
```

如果复用同一个变量：

```python
billsum = load_dataset("FiscalNote/billsum", split="ca_test")
billsum = billsum.train_test_split(test_size=0.2)
```

第一次从上到下运行通常没问题。但如果在 notebook 里只重跑第二行，或者某个 cell 里写的是：

```python
billsum = billsum.train_test_split(test_size=0.2)
```

第二次运行时，`billsum` 已经不是原始 `Dataset`，而是上一次切分后的 `DatasetDict`。`DatasetDict` 没有 `.train_test_split(...)` 方法，于是会报：

```text
AttributeError: 'DatasetDict' object has no attribute 'train_test_split'
```

所以这里的原则是：原始数据和加工后数据分开命名。`*_raw` 保存原材料，普通变量名保存下一步要训练或处理的数据。

## 来源

- [BillSum dataset on Hugging Face](https://huggingface.co/datasets/FiscalNote/billsum)
- [Hugging Face Datasets loading methods](https://huggingface.co/docs/datasets/en/package_reference/loading_methods)
