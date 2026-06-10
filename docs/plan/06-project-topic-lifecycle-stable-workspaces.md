# 项目与专题生命周期稳定工作区方案

> 日期：2026-06-10
> 状态：拟定

## 背景

旧模型用 `01-backlog`、`02-learning`、`03-completed`、`04-abandoned` 表达项目状态。问题是：一个项目可以有多个专题，专题会完成，也会再次新增。如果用移动目录表达状态，项目路径会反复变化，进而破坏链接、终端界面恢复、模型上下文和编辑器入口。

核心问题不是“如何修复旧路径”，而是如何让系统天然不容易产生旧路径。

## 核心决策

项目路径保持稳定，状态只写入元数据和视图。

```text
路径表达身份
状态表达视图
软链接表达当前焦点
归档目录表达默认不可见
```

目标结构：

```text
workspaces/
  current-project -> projects/<项目>
  current-topic -> projects/<项目>/topics/<专题>
  .daedalus/
    current.toml
    project-index.toml
  backlog/
  projects/
    <项目>/
      topics/
      shared/
      source/
      .archive/
```

## 改造要点

- 所有正式项目放入 `workspaces/projects/`，不再因状态变化移动。
- `current.toml` 是当前学习现场的机器真相。
- `current-project` 和 `current-topic` 是用户可点击入口，可以提交到 Git。
- `current-topic` 只在有活跃专题时存在。
- `current-project` 可以在项目空闲时保留，方便复盘和启动新专题。
- 废弃专题进入项目内部 `.archive/`。
- 暂不考虑 Windows 一等体验；个人 macOS 使用下，软链接是合理取舍。

## 状态模型

项目状态：

```text
active：至少一个专题正在学习
idle：没有活跃专题，但项目仍是正式学习对象
abandoned：项目不进入默认学习视图
```

专题状态：

```text
planned
active
completed
deferred
abandoned
```

聚合规则：

```text
有 active 专题 -> 项目 active
没有 active 专题 -> 项目 idle
废弃必须显式触发
```

## 归档与忽略

`.archive/` 要能被 Git 保存，但默认不进入搜索和模型上下文。

`.gitignore` 不忽略 `.archive/`。

调研结论：

- 各家 coding agent 没有统一的 AI ignore 标准。
- `.gitignore` 是版本控制规则，不应承担“上下文边界”的全部职责。
- `.ignore` 适合约束本地搜索和许多基于 ripgrep 的扫描。
- Cursor、Cline、Roo Code、Aider、Continue、Gemini CLI、Windsurf、Augment 都有各自的 ignore 文件。
- Claude Code 当前更适合用 `.claude/settings.json` 的权限 deny 规则，不应假设 `.claudeignore` 是稳定机制。
- Codex 当前没有确认 `.codexignore` 是官方稳定机制，不应默认生成。
- GitHub Copilot 的排除规则主要在 GitHub 设置中配置，不是 repo 内 `.copilotignore`。

第一批生成这些文件：

```text
.ignore
.cursorignore
.clineignore
.rooignore
.aiderignore
.continueignore
.geminiignore
.codeiumignore
.augmentignore
```

内容：

```ignore
workspaces/projects/**/.archive/
```

Claude Code 另行生成 `.claude/settings.json` 的 deny 配置；Codex 暂不生成 `.codexignore`，后续只有在官方确认支持后再加入。

## 需要实现

- 新增 `workspaces/projects/`、`workspaces/backlog/`、`workspaces/.daedalus/`。
- 新增 `current.toml` 和 `project-index.toml`。
- 新增并维护 `current-project`、`current-topic` 软链接。
- 调整项目、专题的创建、启动、完成、废弃、恢复命令。
- 调整终端界面，从 `current.toml` 恢复当前学习现场。
- 更新模板、提示词、说明文档和迁移脚本。

## 验收标准

- 打开 `workspaces/` 后，可以直接点击 `current-topic` 回到当前专题。
- 项目不会因为专题完成或重启而改变路径。
- 当前学习现场可以从 `current.toml` 和软链接双向校验。
- `.archive/` 默认不进入搜索、终端界面和模型上下文。
- 新增专题不需要移动项目目录。
