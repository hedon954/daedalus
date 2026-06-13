# 知识库交互网站重构方案

> 日期：2026-06-14
> 状态：实施中

## 背景

之前的知识库网页方向错了：它把 `knowledge-base/` 的 Markdown 转成 HTML 文档站，本质上只是换了一个阅读外壳。这样的网页没有发挥浏览器的优势，也没有降低复杂知识的理解成本。

新的目标不是“生成文档站”，而是“基于知识库内容开发理解型网页”。AI 需要先理解 Markdown、topic closeout、demo 和源码，再把知识重新设计成交互式页面：可以切换场景、推进状态机、对比策略、观察事件流、回看来源。

## 核心原则

```text
Markdown 是知识来源
网页是理解界面
网页内容由 AI 理解后手工策展
不要把 Markdown 自动搬成 HTML
```

网页必须解决 Markdown 难以解决的问题：

- 复杂链路的动态演进，例如 command approval / sandbox / retry。
- 多个策略组合的对比，例如 approval policy 和 retry policy。
- 并发事件的观察，例如 LLM stream、tool event、approval event、UI event。
- 抽象机制的分层，例如 ReAct loop、ToolRuntime、ExecutionRunner、TUI reducer。
- 从知识点回到来源，例如原始 Markdown、topic、demo、源码。

## 产品形态

新增 `apps/knowledge-web/`，使用 `React + Vite + TypeScript` 实现一个本地静态交互网站。

首版不是通用知识库 CMS，而是一个高质量样板：

```text
Codex tools-permissions interactive atlas
```

它围绕当前已经完成的知识库，提供四个理解面板：

- **命令执行安全**：用场景切换和步骤推进展示 capability、approval、sandbox、retry、observation。
- **ReAct 工具闭环**：展示 messages、LLM stream、tool batch、observation 和外部事件的双通道。
- **Sandbox 机制**：对比 approval 和 sandbox 的边界，展示 OS sandbox、container、VM、language sandbox 的区别。
- **Rust async / TUI**：展示 `Future -> Waker -> Stream -> mpsc -> TUI event loop` 的推进关系。

每个面板都要有：

- 当前机制图。
- 可点击状态或场景。
- 对应的解释文本。
- 关键 trade-off。
- 来源链接。

## 内容生产规则

网页内容不从 Markdown 自动生成。迭代流程是：

1. Agent 读取 `knowledge-base/`、topic reflection、demo code 和相关 notes/guides。
2. Agent 判断哪些内容适合做成交互模型，哪些只适合保留为文档来源。
3. Agent 在 React 组件和数据结构中直接写入策展后的网页内容。
4. 网页只保留来源链接，方便追溯，不复制 Markdown 原文。
5. 每完成一个 topic 的知识归档，再决定是否新增一个交互面板、更新已有面板，或只补充索引。

这意味着网页不是知识库的“自动投影”，而是知识库的“教学产品化表达”。

## 目录设计

```text
apps/knowledge-web/
  package.json
  index.html
  src/
    main.tsx
    App.tsx
    styles.css
```

首版保持简单，不引入路由和文档框架。后续如果交互面板增多，再拆分为：

```text
src/
  modules/
    command-security/
    react-runtime/
    sandbox/
    rust-async-tui/
  data/
  components/
```

## 命令

```text
make web
make knowledge-web-build
make knowledge-web-check
```

`make web` 负责安装依赖、构建并启动本地预览。

## 进化迭代机制

新增 `knowledge-web-curator` skill。每次 topic closeout 或知识库新增后，Agent 先阅读知识来源，再判断哪些机制值得做成交互模型。只有当网页交互能降低理解成本时，才更新 `apps/knowledge-web`。

新增 `apps/knowledge-web/scripts/inspect-atlas.ts` 作为确定性检查：

- 禁止把 Markdown 自动渲染成 HTML。
- 检查来源链接是否存在。
- 检查核心学习模块是否保留。
- 检查页面仍然包含交互入口。

这不是替代人的设计判断，而是防止网页退化成文档投影。

## 验收标准

- 页面不是 Markdown 目录站，也不是 Markdown 渲染结果。
- 页面首屏能说明这是 “interactive atlas”，并提供可操作的学习路径。
- 至少有 3 个以上可交互区域，例如场景切换、步骤推进、事件流演示、策略对比。
- 每个交互区域都能把复杂机制拆成可理解的状态变化。
- 每个区域都有来源链接，能回到 `knowledge-base/` 或 demo。
- `make knowledge-web-check` 通过。
- `make web` 能启动本地站点。
- 必须用浏览器验证：首屏、交互状态变化、响应式布局和来源链接。

## 非目标

- 不做 Markdown 自动同步。
- 不做全量知识库搜索引擎。
- 不做通用 CMS。
- 不用网页替代 `knowledge-base/`，Markdown 仍然是长期知识正文源。
- 不要求每篇知识条目都有网页；只有复杂、适合动态表达的知识才产品化成网页模块。
