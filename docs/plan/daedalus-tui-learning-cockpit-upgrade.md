# daedalus-tui 学习驾驶舱升级方案

## Summary

`daedalus-tui` 从“只读状态摘要页”升级为“学习现场恢复工具”。学习者打开 TUI 后，应能快速回答：

- 当前在哪个 project / topic / stage / slice？
- 下一步为什么重要？
- 该读哪些文件？
- 做完后如何验收？
- 如果摘要不够，如何完整阅读原始内容？

核心原则：

```text
第一屏负责定向
详情页负责完整阅读
写操作仍全部走 daedalus CLI
```

## First Principles

### 1. 学习者打开 TUI 时，首先需要恢复现场

长程学习最容易丢失的是“我现在在哪里”。TUI 的第一屏应该优先展示当前位置和下一步，而不是把所有状态平均分配到多个面板。

因此第一屏的主信息是：

```text
Current Position
Next Action Card
Evidence / Drift
Reading Entrypoints
```

### 2. 摘要不能替代完整阅读

TUI 面板天然空间有限。如果把长文本硬塞进第一屏，就会产生截断、拥挤和误读。

所以 v1 采用两层结构：

```text
overview = 摘要 + 入口
detail view = 完整内容 + 滚动阅读
```

### 3. TUI 是只读投影，不是状态修改入口

daedalus 的状态源仍是 filesystem artifacts 和 `.daedalus/state.toml`。TUI 只负责读取、组织和展示，不负责推进 stage、修改 todo、执行 validate 或创建 review。

## Product Shape

### Overview

第一屏围绕学习者动线组织：

```text
恢复现场 -> 判断下一步 -> 找到入口文件 -> 识别风险 -> 回到执行
```

建议面板：

- `Current Vector`：project、topic、stage、status、bucket。
- `Next Action Card`：当前目标、缺口、下一步动作、完成后解锁。
- `Evidence / Drift`：缺失产物数、review/knowledge 数、transition 数、推荐 guide、建议验收命令。
- `Reading Entrypoints`：`g` guide、`t` todo、`o` outcome-map。
- `Risks / Missing`：当前缺失产物预览。
- `Review / Knowledge`：复习与知识体系状态预览。
- `Recent Transitions`：状态流转预览。

### Detail View

详情页用于完整阅读，不直接截断：

- 可打开当前 guide。
- 可打开 `.daedalus/todo.md`。
- 可打开 `.daedalus/outcome-map.md`。
- 可查看完整 missing artifacts。
- 可查看完整 review / knowledge focus。
- 可查看完整 transition history。

详情页必须展示来源和滚动位置，让学习者知道自己正在读哪个 artifact。详情内容统一通过 `markdown-tui` 渲染，不手写 markdown parser。

## Interaction Model

键盘交互保持轻量：

| Key | Behavior |
| --- | --- |
| `j/k` 或方向键 | overview 切换焦点；detail 滚动 |
| `enter` | 打开当前焦点详情 |
| `g` | 打开当前 guide |
| `t` | 打开 todo |
| `o` | 打开 outcome-map |
| `u/d` | detail 上下滚动 |
| `PageUp/PageDown` | detail 翻页 |
| `Home/End` | detail 跳到顶部/底部 |
| `r` | 重新读取当前状态 |
| `b` / `backspace` | 返回上一层 |
| `q` / `esc` | 退出 |

## Implementation Notes

- 扩展 `TuiView`，增加只读 detail view。
- 扩展 `TuiApp`，增加 overview focus、detail source、scroll offset 和 refresh 行为。
- 扩展 `TuiOverview`，保留摘要字段，同时提供 guide / todo / outcome-map 等可读入口。
- 使用 `markdown-tui` 将 detail 内容渲染为 ratatui widget；真实 markdown 文件和虚拟详情页都走同一渲染路径。
- 当前推荐 guide 只选择 still-actionable guide，避免把 `Current slice: ... completed` 的旧 guide 当成下一步。
- 第一屏只展示 preview；完整内容通过 detail view 阅读。
- review、knowledge、transition、missing artifacts 在数据层保留完整集合，由渲染层决定预览数量。

## Test Plan

- `build_next_action` 能识别 `08-slice-*.md` 这类顺序命名 guide。
- completed guide 不会被误用为当前下一步。
- detail source 读取完整文件内容，不丢行、不截断。
- detail scroll 在上下滚动、翻页、窗口高度变化时不越界。
- overview 能提供 reading map 和 evidence/drift 摘要。
- `cargo test -p daedalus-cli` 通过。

## Non-Goals

- v1 不在 TUI 内编辑 artifacts。
- v1 不执行 stage transition、review lifecycle、knowledge promotion。
- v1 不自动运行 validate / cargo test。
- v1 不做鼠标支持，不打开外部编辑器。
