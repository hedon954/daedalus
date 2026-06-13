# 知识库有机重写方案

> 日期：2026-06-13
> 状态：已实施

## 背景

当前知识库归档暴露出三个根本问题：

- 条目硬套模板，读起来像表格填空，不像真正能唤起理解的笔记。
- 分类过度拆散，把一个完整认知拆成 `problems`、`patterns`、`concepts`、`skills` 等碎片，导致用户很难沿着一个主题重新理解。
- 归档时只是在“放入某个预设目录”，没有真正思考这条知识属于哪棵知识树、应该如何和既有知识体系连接。

这违背了 daedalus 的目标：知识库不是资料仓库，而是用户能力结构的外化。

## 核心决策

废弃固定知识库模板和过细的预设分类。

新的知识库采用“传统知识体系 + 有机演化”的方式：

```text
知识库目录不是先验真理
而是随着学习内容不断生长、拆分、合并和重组的能力地图
```

每篇笔记按内容需要自由组织，不强制包含固定字段。Agent 只能保证它具备这些品质：

- 有清晰问题入口。
- 从第一性原理出发。
- 讲清底层原理。
- 说明现实约束和 trade-off。
- 关联学习项目、topic、demo、源码或外部资料。
- 能帮助用户未来复习、迁移或继续追问。

## 目录模型

知识库不再使用 `concepts / skills / patterns / problems / cases / source-maps / drills` 作为顶层结构。

改为按知识体系自然分层，例如：

```text
knowledge-base/
  README.md
  index.md
  computer-systems/
    operating-systems/
    computer-networking/
    databases/
    compilers/
    programming-languages/
  software-engineering/
    architecture/
    design-principles/
    testing/
    observability/
  ai-agents/
    tool-use/
    context-engineering/
    safety-and-permissions/
    agent-runtime/
  rust/
    async-runtime/
    cli-and-tui/
    process-and-io/
  learning-methods/
```

这只是初始方向，不是永久模板。新增知识时允许：

- 新建更合适的目录。
- 合并重复目录。
- 调整层级。
- 把旧笔记迁移到更准确的位置。
- 在索引页记录知识树的当前结构。

## 归档流程

针对 `reflection/candidate-map.md` 的每一项，归档前必须完成：

1. 判断它是否真的值得进入知识库。
2. 判断它属于哪棵知识树、哪个目录。
3. 查阅外部资料校准事实，优先官方文档、论文、源码仓库和成熟工程实践。
4. 从第一性原理解释这个知识为什么存在。
5. 讲清底层原理，而不是只写项目里的表层做法。
6. 说明它和本次学习项目、topic、demo、源码或 closeout 的关系。
7. 判断它是新增知识、补充旧知识、修正旧知识，还是只需要建立链接。

归档不是“为每个候选生成一篇文章”。多个候选如果属于同一个问题域，应合并成一篇更完整的主题笔记。

## Markdown 组织方式

笔记按需组织，不套固定模板。

推荐但不强制的写法：

```markdown
# 标题

这篇笔记解决什么问题。

## 从第一性原理看

## 底层原理

## 在本次 Codex tools-permissions 学习中的体现

## 现实工程中的取舍

## 关联
```

如果某篇笔记更适合用流程、对比表、代码片段、案例叙事或复习题，就按实际需要写，不为模板服务。

## 当前 Codex 知识库重写

删除上一轮生成的模板化知识条目：

```text
knowledge-base/problems/agent-local-command-execution.md
knowledge-base/patterns/agent-command-safety-pipeline.md
knowledge-base/patterns/tool-observability-react-feedback.md
knowledge-base/skills/rust-async-streaming-agent.md
knowledge-base/concepts/os-sandbox-and-shell-boundary.md
knowledge-base/cases/codex-tools-permissions-demo.md
knowledge-base/source-maps/codex-tools-permissions.md
knowledge-base/drills/agent-command-safety-review.md
```

基于 `candidate-map.md` 重新归档为更少、更完整、更自然的主题笔记。初步方向：

- `ai-agents/safety-and-permissions/local-command-execution.md`
- `ai-agents/tool-use/react-tool-runtime.md`
- `rust/async-runtime/streaming-agent.md`
- `computer-systems/operating-systems/sandbox.md`
- `rust/cli-and-tui/terminal-agent-ui.md`

最终文件数量不固定，以“是否有助于用户理解和复习”为准。

## Daedalus 规则改造

需要同步修改：

- `system/prompts/common/archive-knowledge.md`
  - 删除固定模板式归档要求。
  - 改成按候选选择目录、查资料、写自然笔记、关联项目。
- `system/prompts/common/candidate-map.md`
  - 候选表只负责发现高价值知识点，不决定最终文件结构。
- `knowledge-base/README.md`
  - 改成说明有机知识树如何演化，而不是解释固定目录。
- `system/templates/knowledge/`
  - 删除或降级固定模板，不再作为默认生成方式。
- `daedalus knowledge template`
  - 不再生成固定字段模板；如果保留命令，只生成空白笔记骨架和最小元信息。
- `daedalus knowledge index / validate / link-check`
  - 继续保留确定性检查，但不强制固定标题字段。

## 验收标准

- 知识库顶层结构变成有层次的知识树，而不是扁平类型桶。
- 旧的 8 篇模板化 Codex 条目被删除或重写，不再保留硬套模板的正文。
- 每个进入知识库的 Codex 候选都能说明：
  - 为什么值得保留。
  - 属于哪个知识目录。
  - 查阅了哪些外部资料。
  - 和本次 topic / demo / closeout 的关系。
- `daedalus knowledge validate` 和 `daedalus knowledge link-check` 仍然通过。
- 用户打开知识库时，能沿着“问题域 -> 原理 -> 本次案例 -> 迁移取舍”的路径复习，而不是在许多小条目之间来回跳。
