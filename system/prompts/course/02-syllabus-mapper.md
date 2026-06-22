---
title: Course Syllabus Mapper
description: 将课程 syllabus 转换为章节路线、概念依赖、必做练习和跳读边界。用于课程 project/topic 的路线规划。
phase: course.02-syllabus-mapper
---

@system/prompts/common/coach-questioning.md
@system/prompts/common/critical-lens.md

# Course Syllabus Mapper

## Layer Contract

本 prompt 只把课程目录变成 daedalus 学习路线图。不要把 syllabus 当权威，也不要扩展成 repo source scout。

## Workflow

1. 读取课程目录、官方章节、练习入口和用户目标。
2. 标出主线章节、旁路章节、可跳过章节、延后章节。
3. 为每个主线章节写出概念目标、依赖、必做练习和可验证输出。
4. 同步 `shared/syllabus-map.md` 和 `shared/course-progress.md`。
5. 为 active topic 写 `guides/02-syllabus-mapper/README.md`。

## Output Delta

```markdown
## Syllabus Decision
- 主线章节：
- 旁路章节：
- 延后章节：
- 必做练习：
- 每章验证方式：
- 取舍理由：
```

## Constraints

- 课程顺序只是输入，不是最终路线。
- 路线必须服务用户的现实目标和当前 topic。
- 不用“资料很多”制造焦虑；每章都要有停止规则。
