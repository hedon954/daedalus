# Syllabus Mapper Migration Bridge

本文件是从 repo-learning 阶段迁移到 course-learning 阶段后的桥接记录。

## Migration Decision

- 旧阶段：`02-repo-scout`
- 新阶段：`02-syllabus-mapper`
- 迁移理由：当前项目的主学习材料是 Hugging Face LLM Course，不是真实代码仓库深读。旧 `02-repo-scout` 实际承担的是课程材料入口、默认模型、训练环境和第一轮 lab 范围收敛。

## Historical Evidence

- [`../02-repo-scout/README.md`](../02-repo-scout/README.md)
- [`../02-repo-scout/01-finetuning-lab-primer.md`](../02-repo-scout/01-finetuning-lab-primer.md)
- [`../02-repo-scout/02-schema-and-engineering-shape.md`](../02-repo-scout/02-schema-and-engineering-shape.md)
- [`../02-repo-scout/03-first-lab-default-plan.md`](../02-repo-scout/03-first-lab-default-plan.md)

## Course Shared Context

- [`../../../../shared/syllabus-map.md`](../../../../shared/syllabus-map.md)
- [`../../../../shared/course-progress.md`](../../../../shared/course-progress.md)
- [`../../../../shared/concept-map.md`](../../../../shared/concept-map.md)

## Next Rule

后续新增课程学习指南不再使用 `repo-scout`、`debugger-guide` 或 `arch-analyzer` 命名；使用 course-learning 阶段名，例如 `04-lesson-lab` 或 `05-mechanism-deep-dive`。
