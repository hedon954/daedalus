# TUI Learning Type Adapter 实现方案

> 日期：2026-06-23
> 状态：已完成

## 背景

daedalus 已经支持至少两类长期学习对象：

- `repo-learning`：围绕真实代码仓库、运行路径、源码不变量、mini demo 和业务迁移。
- `course-learning`：围绕课程 syllabus、概念机制、lesson lab、练习复现、迁移练习和掌握度验证。

CLI init / validate / render 已经理解 course-learning，但 TUI 目前只是借用通用 project/topic state 结构显示内容。它能显示 course stage，却没有真正理解学习类型差异：

- 不显示 learning type / source kind / course URL。
- 当前阅读入口仍偏向 repo-learning 的 slice guide / Action Card。
- closeout 逻辑硬编码 `10-reflection`。
- 进度文案只说 stages complete，不能区分 repo stages / course stages。
- TUI 测试没有 course-learning fixture。

这会导致 TUI 表面可用，但语义上仍偏向 repo-learning。

## 目标形态

TUI 应该拆成两层：

```text
state.toml / files
  -> learning-type adapter
  -> TuiOverview
  -> screens render
```

主干 TUI 只依赖统一的 `TuiOverview`，不直接散落 `repo-learning` / `course-learning` 判断。

每种学习类型通过 adapter 提供：

- 类型标签：例如 `Repo Learning`、`Course Learning`。
- 当前单位文案：例如 `Stage`、`Course Stage`。
- 进度单位文案：例如 `repo stages`、`course stages`。
- 当前阅读入口标签：例如 `Current Guide`、`Course Guide`。
- 当前阅读查找策略。
- closeout 阶段 ID 与 closeout 文案。
- source reference 展示策略。

## Adapter 设计

第一版采用轻量结构体，而不是复杂 trait/plugin：

```text
LearningTypeTuiAdapter
  kind
  label
  current_unit_label
  progress_unit_label
  current_reading_label
  closeout_label
  closeout_stage_id
  guide_lookup
```

`repo-learning` adapter：

- 当前阅读优先找 `guides/<stage>/*slice*.md` 且包含 `Action Card`。
- closeout stage 是 `10-reflection`。
- next action 继续解析 `Current slice`、`Current gap`、`Action Card`。

`course-learning` adapter：

- 当前阅读优先找 `guides/<course-stage>/README.md`。
- closeout stage 是 `09-closeout-archive`。
- next action 从 course guide / todo 中提取 lesson、focus、next lesson lab。
- source reference 优先展示 `project.course_url`。

fallback adapter：

- 面向未来未知学习类型。
- 使用 stage README 查找策略。
- 不提供特殊 closeout 语义。

## 解耦原则

- screens 只渲染 `TuiOverview` 字段，不直接判断学习类型。
- repo-learning 和 course-learning 的差异集中在 `crates/daedalus-cli/src/interfaces/tui/learning_types.rs` 的 adapter / guide helper 中。
- 后续新增 learning type 时，不修改 repo/course adapter，只新增该类型 adapter 和注册入口。
- 如果新类型遵守通用 stage README 约定，可以先走 fallback adapter。

## 实现范围

### TUI App

- 新增 `tui/learning_types.rs`，内聚 learning type adapter、guide 查找、next action 构造和 source reference 策略。
- 在 `TuiOverview` 中增加 learning type / source / display labels。
- `load_overview` 根据 `task.kind` 选择 adapter。
- `build_next_action` 通过 adapter 选择 repo action guide 或 course stage README。
- `find_current_guide` 改为 adapter 驱动。
- closeout 逻辑从硬编码 `10-reflection` 改为 adapter closeout stage。

### TUI Screens

- Current panel 显示 learning type。
- Evidence 显示 source kind / source reference。
- Progress label 使用 adapter 的 progress unit。
- Reading map 使用 adapter 的 current reading label。

### Tests

- 保留 repo-learning action guide 行为。
- 新增 course-learning fixture，验证：
  - TUI 识别 `course-learning`。
  - 展示 course URL。
  - 当前阅读入口使用 `guides/04-lesson-lab/README.md`。
  - next action 使用 course stage 文案。

## 验收标准

- `cargo test --manifest-path crates/Cargo.toml -p daedalus-cli --test cli_workflow` 通过。
- TUI app 单元测试覆盖 repo-learning 和 course-learning 两类 guide 查找策略。
- `daedalus validate` 通过。
- 搜索 TUI 主渲染文件时，不出现散落的 repo/course 分支判断。
- course-learning project 在 TUI 中能看到类型、课程入口、course stage 进度和 course guide。

## 非目标

- 不做完整 TUI 视觉重设计。
- 不实现动态插件系统。
- 不把 book-learning / paper-learning 一起实现。
- 不要求所有历史 repo-learning guide 立即改名。
