use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use toml_edit::{Array, ArrayOfTables, DocumentMut, Item, Table, value};

use crate::application::ide::sync_rust_analyzer_linked_projects;
use crate::application::render::render_state;
use crate::domain::{DaedalusError, Result};
use crate::infrastructure::{clock, state_toml, template_fs, workspace_fs};

/// repo learning workspace 迁移参数。
#[derive(Debug, Clone)]
pub struct MigrateRepoLearningOptions {
    pub repo_root: PathBuf,
    pub task_dir: PathBuf,
    pub topic_slug: String,
    pub topic_title: String,
    pub execute: bool,
}

/// 迁移输出。
#[derive(Debug, Clone)]
pub struct MigrateRepoLearningOutput {
    pub project_dir: PathBuf,
    pub topic_dir: PathBuf,
    pub backup_dir: Option<PathBuf>,
    pub execute: bool,
    pub state_md: Option<PathBuf>,
    pub topic_state_md: Option<PathBuf>,
    pub planned_actions: Vec<String>,
}

/// repo/course project 类型迁移参数。
#[derive(Debug, Clone)]
pub struct MigrateCourseLearningOptions {
    pub repo_root: PathBuf,
    pub project_dir: PathBuf,
    pub course_url: String,
    pub execute: bool,
}

/// course learning 类型迁移输出。
#[derive(Debug, Clone)]
pub struct MigrateCourseLearningOutput {
    pub project_dir: PathBuf,
    pub topic_dir: Option<PathBuf>,
    pub backup_dir: Option<PathBuf>,
    pub execute: bool,
    pub state_md: Option<PathBuf>,
    pub topic_state_md: Option<PathBuf>,
    pub planned_actions: Vec<String>,
}

/// 将旧 single-topic workspace 一次性迁移为 project/topic 新结构。
pub fn migrate_repo_learning(
    options: MigrateRepoLearningOptions,
) -> Result<MigrateRepoLearningOutput> {
    let project_dir = options.task_dir;
    let topic_slug = sanitize_slug(&options.topic_slug);
    let topic_title = if options.topic_title.trim().is_empty() {
        topic_slug.clone()
    } else {
        options.topic_title.trim().to_owned()
    };
    let topic_dir = project_dir.join("topics").join(&topic_slug);

    let mut planned_actions = vec![
        format!("create topic dir {}", topic_dir.display()),
        "move notes/ guides/ demo/ into topic".to_owned(),
        "move topic-level .daedalus markdown files into topic".to_owned(),
        "rewrite project .daedalus/state.toml".to_owned(),
        "render project and topic state.md".to_owned(),
    ];

    validate_old_workspace(&project_dir)?;
    if !options.execute {
        planned_actions.push("dry-run only; pass --execute to apply".to_owned());
        return Ok(MigrateRepoLearningOutput {
            project_dir,
            topic_dir,
            backup_dir: None,
            execute: false,
            state_md: None,
            topic_state_md: None,
            planned_actions,
        });
    }

    let old_state_path = state_toml::state_path(&project_dir);
    let old_doc = state_toml::load_state_doc(&old_state_path)?;
    let task_name = state_toml::task_name(&old_doc);
    let migrated_at = clock::now_local_timestamp();
    let task_created_at = old_doc
        .get("task")
        .and_then(|task| task.get("created_at"))
        .and_then(Item::as_str)
        .unwrap_or(&migrated_at)
        .to_owned();
    let backup_dir = project_dir
        .join(".daedalus-migration-backup")
        .join(migrated_at.replace([':', ' '], "-"));
    workspace_fs::ensure_dir(&backup_dir)?;
    backup_if_exists(&project_dir, &backup_dir, ".daedalus")?;
    backup_if_exists(&project_dir, &backup_dir, "notes")?;
    backup_if_exists(&project_dir, &backup_dir, "guides")?;
    backup_if_exists(&project_dir, &backup_dir, "demo")?;
    let preserved_source_files = preserve_existing_files(
        &project_dir,
        &[
            "source/README.md",
            "source/pull_source.sh",
            "source/.gitignore",
        ],
    )?;

    workspace_fs::ensure_dir(&topic_dir)?;
    move_dir_if_exists(&project_dir, &topic_dir, "notes")?;
    move_dir_if_exists(&project_dir, &topic_dir, "guides")?;
    move_dir_if_exists(&project_dir, &topic_dir, "demo")?;
    workspace_fs::ensure_dir(&topic_dir.join(".daedalus"))?;
    for file in [
        "task-card.md",
        "outcome-map.md",
        "todo.md",
        "artifact-index.md",
        "decision-log.md",
        "validation-log.md",
        "long-context.md",
    ] {
        move_file_if_exists(
            &project_dir.join(".daedalus").join(file),
            &topic_dir.join(".daedalus").join(file),
        )?;
    }

    let topic_doc = build_topic_doc(old_doc, &topic_slug, &topic_title);
    state_toml::save_state_doc(&state_toml::state_path(&topic_dir), &topic_doc)?;

    let project_template = options
        .repo_root
        .join("system")
        .join("templates")
        .join("repo");
    template_fs::copy_template_dir(
        &project_template,
        &project_dir,
        &[
            ("{{TASK_NAME}}", &task_name),
            ("{{CREATED_AT}}", &task_created_at),
            ("{{TOPIC_SLUG}}", &topic_slug),
            ("{{TOPIC_TITLE}}", &topic_title),
        ],
    )?;
    restore_preserved_files(&project_dir, preserved_source_files)?;

    let mut project_doc = state_toml::load_state_doc(&state_toml::state_path(&project_dir))?;
    let topic_next_action = state_toml::next_action(&topic_doc);
    state_toml::set_next_action(
        &mut project_doc,
        &format!("继续 active topic `{topic_slug}`：{topic_next_action}"),
    );
    state_toml::append_transition(
        &mut project_doc,
        crate::domain::transition::Transition {
            stage: "project".to_owned(),
            action: "migrate".to_owned(),
            timestamp: migrated_at,
            actor: "daedalus-cli".to_owned(),
            reason: format!(
                "从 single-topic workspace 迁移为 multi-topic project，初始 topic 为 `{topic_slug}`。"
            ),
            approval_source: None,
        },
    );
    state_toml::save_state_doc(&state_toml::state_path(&project_dir), &project_doc)?;

    let state_md = render_state(&project_dir)?.path;
    let topic_state_md = render_state(&topic_dir)?.path;
    workspace_fs::sync_current_workspace(&options.repo_root, Some(&project_dir), Some(&topic_dir))?;
    sync_rust_analyzer_linked_projects(&options.repo_root)?;
    Ok(MigrateRepoLearningOutput {
        project_dir,
        topic_dir,
        backup_dir: Some(backup_dir),
        execute: true,
        state_md: Some(state_md),
        topic_state_md: Some(topic_state_md),
        planned_actions,
    })
}

/// 将已有 multi-topic project 从 repo-learning 语义迁移为 course-learning。
///
/// 该迁移只改变项目类型、共享课程索引和 topic 阶段状态机，不自动重命名历史
/// guides/notes 目录，以免破坏已有学习证据。
pub fn migrate_course_learning(
    options: MigrateCourseLearningOptions,
) -> Result<MigrateCourseLearningOutput> {
    let project_dir = options.project_dir;
    let mut planned_actions = vec![
        "rewrite project .daedalus/state.toml as course-learning".to_owned(),
        "create missing course shared artifacts".to_owned(),
        "rewrite active topic stage graph to course-learning lifecycle".to_owned(),
        "render project and active topic state.md".to_owned(),
    ];

    let project_state_path = state_toml::state_path(&project_dir);
    let project_doc = state_toml::load_state_doc(&project_state_path)?;
    if state_toml::state_kind(&project_doc) != "project" {
        return Err(DaedalusError::InvalidStateDocumentKind {
            expected: "project".to_owned(),
            actual: state_toml::state_kind(&project_doc).to_owned(),
        });
    }
    let active_topic_dir = state_toml::active_topic(&project_doc)
        .and_then(|slug| state_toml::topic_path(&project_doc, &slug))
        .map(|path| project_dir.join(path));

    if !options.execute {
        planned_actions.push("dry-run only; pass --execute to apply".to_owned());
        return Ok(MigrateCourseLearningOutput {
            project_dir,
            topic_dir: active_topic_dir,
            backup_dir: None,
            execute: false,
            state_md: None,
            topic_state_md: None,
            planned_actions,
        });
    }

    let migrated_at = clock::now_local_timestamp();
    let backup_dir = project_dir.join(".daedalus-migration-backup").join(format!(
        "course-learning-{}",
        migrated_at.replace([':', ' '], "-")
    ));
    workspace_fs::ensure_dir(&backup_dir)?;
    backup_if_exists(&project_dir, &backup_dir, ".daedalus")?;
    backup_if_exists(&project_dir, &backup_dir, "shared")?;
    if let Some(topic_dir) = &active_topic_dir {
        backup_if_exists(topic_dir, &backup_dir.join("active-topic"), ".daedalus")?;
    }

    let task_name = state_toml::task_name(&project_doc);
    let topic_slug = state_toml::active_topic(&project_doc).unwrap_or_default();
    let topic_title = state_toml::topics(&project_doc)
        .into_iter()
        .find(|topic| topic.slug == topic_slug)
        .map(|topic| topic.title)
        .unwrap_or_else(|| topic_slug.clone());
    let course_template = options
        .repo_root
        .join("system")
        .join("templates")
        .join("course");
    template_fs::copy_template_dir_missing(
        &course_template,
        &project_dir,
        &[
            ("{{TASK_NAME}}", &task_name),
            ("{{CREATED_AT}}", &migrated_at),
            ("{{TOPIC_SLUG}}", &topic_slug),
            ("{{TOPIC_TITLE}}", &topic_title),
            ("{{COURSE_URL}}", &options.course_url),
        ],
    )?;
    if let Some(topic_dir) = &active_topic_dir {
        let course_topic_template = options
            .repo_root
            .join("system")
            .join("templates")
            .join("course-topic");
        template_fs::copy_template_dir_missing(
            &course_topic_template,
            topic_dir,
            &[
                ("{{TASK_NAME}}", &task_name),
                ("{{CREATED_AT}}", &migrated_at),
                ("{{TOPIC_SLUG}}", &topic_slug),
                ("{{TOPIC_TITLE}}", &topic_title),
                ("{{COURSE_URL}}", &options.course_url),
            ],
        )?;
    }

    let mut project_doc = state_toml::load_state_doc(&project_state_path)?;
    project_doc["task"]["kind"] = value("course-learning");
    project_doc["project"]["source_kind"] = value("course");
    project_doc["project"]["course_url"] = value(options.course_url.as_str());
    if project_doc["project"]["source_name"]
        .as_str()
        .unwrap_or_default()
        .is_empty()
    {
        project_doc["project"]["source_name"] = value(task_name.as_str());
    }
    state_toml::set_next_action(
        &mut project_doc,
        &format!("继续 course-learning active topic `{topic_slug}`，按课程阶段推进下一份 guide。"),
    );
    state_toml::append_transition(
        &mut project_doc,
        crate::domain::transition::Transition {
            stage: "project".to_owned(),
            action: "migrate".to_owned(),
            timestamp: migrated_at.clone(),
            actor: "daedalus-cli".to_owned(),
            reason: "将项目从 repo-learning 语义迁移为 course-learning。".to_owned(),
            approval_source: None,
        },
    );
    state_toml::save_state_doc(&project_state_path, &project_doc)?;

    if let Some(topic_dir) = &active_topic_dir {
        migrate_topic_to_course_learning(topic_dir, &migrated_at)?;
    }

    rewrite_course_project_markdown(&project_dir, &options.course_url)?;
    let topic_state_md = if let Some(topic_dir) = &active_topic_dir {
        Some(render_state(topic_dir)?.path)
    } else {
        None
    };
    let state_md = render_state(&project_dir)?.path;
    sync_rust_analyzer_linked_projects(&options.repo_root)?;

    Ok(MigrateCourseLearningOutput {
        project_dir,
        topic_dir: active_topic_dir,
        backup_dir: Some(backup_dir),
        execute: true,
        state_md: Some(state_md),
        topic_state_md,
        planned_actions,
    })
}

fn migrate_topic_to_course_learning(topic_dir: &Path, migrated_at: &str) -> Result<()> {
    let topic_state_path = state_toml::state_path(topic_dir);
    let mut topic_doc = state_toml::load_state_doc(&topic_state_path)?;
    if state_toml::state_kind(&topic_doc) != "topic" {
        return Err(DaedalusError::InvalidStateDocumentKind {
            expected: "topic".to_owned(),
            actual: state_toml::state_kind(&topic_doc).to_owned(),
        });
    }
    let old_statuses = stage_statuses(&topic_doc);
    let old_current =
        state_toml::current_phase(&topic_doc).unwrap_or_else(|| "01-goal-aligner".to_owned());
    let new_current = repo_stage_to_course_stage(&old_current);

    topic_doc["topic"]["kind"] = value("course-learning-topic");
    topic_doc["topic"]["current_phase"] = value(new_current);
    topic_doc["topic"]["next_action"] = value(format!(
        "继续 course-learning 阶段 `{new_current}`，优先使用 guides/{new_current}/ 记录下一步学习指南。"
    ));
    topic_doc["stages"] = Item::ArrayOfTables(course_stage_tables(&old_statuses));
    ensure_course_todo_recovery(topic_dir, new_current)?;
    ensure_course_bridge_artifacts(topic_dir, &old_statuses)?;
    state_toml::append_transition(
        &mut topic_doc,
        crate::domain::transition::Transition {
            stage: new_current.to_owned(),
            action: "resume".to_owned(),
            timestamp: migrated_at.to_owned(),
            actor: "daedalus-cli".to_owned(),
            reason: "将 topic 从 repo-learning 阶段映射为 course-learning 阶段；历史 guide/notes 目录保留。"
                .to_owned(),
            approval_source: None,
        },
    );
    state_toml::save_state_doc(&topic_state_path, &topic_doc)
}

fn ensure_course_todo_recovery(topic_dir: &Path, current_phase: &str) -> Result<()> {
    let todo_path = topic_dir.join(".daedalus/todo.md");
    if !todo_path.exists() {
        return Ok(());
    }
    let mut content = fs::read_to_string(&todo_path).map_err(|source| DaedalusError::Io {
        path: todo_path.clone(),
        source,
    })?;
    if content.contains(current_phase) {
        return Ok(());
    }
    content.push_str(&format!(
        "\n## Course Learning Migration Cursor\n\n- Current course-learning stage：`{current_phase}`\n- 下一步：按 course-learning 阶段名继续推进，不再新增 repo-learning 命名的 guide。\n"
    ));
    fs::write(&todo_path, content).map_err(|source| DaedalusError::Io {
        path: todo_path,
        source,
    })
}

fn ensure_course_bridge_artifacts(
    topic_dir: &Path,
    old_statuses: &HashMap<String, String>,
) -> Result<()> {
    let bridges = [
        (
            "02-repo-scout",
            "guides/02-syllabus-mapper/README.md",
            "# Syllabus Mapper Migration Bridge\n\n旧 `02-repo-scout` 已完成；迁移后视为 `02-syllabus-mapper` 的历史证据。后续新课程指南使用 course-learning 阶段名。\n",
        ),
        (
            "03-socratic-coach",
            "guides/03-concept-roadmap/README.md",
            "# Concept Roadmap Migration Bridge\n\n旧 `03-socratic-coach` 已完成；迁移后视为 `03-concept-roadmap` 的历史输入。后续应补充课程概念问题和 mechanism checkpoint。\n",
        ),
        (
            "04-debugger-guide",
            "guides/04-lesson-lab/README.md",
            "# Lesson Lab Migration Bridge\n\n旧 `04-debugger-guide` 已完成；迁移后视为 `04-lesson-lab` 的历史实验现场。后续 lesson lab 应记录预测、最小代码、观察和用户解释。\n",
        ),
        (
            "04-debugger-guide",
            "notes/04-lesson-lab/README.md",
            "# Lesson Lab Notes Migration Bridge\n\n旧 `notes/04-debugger-guide` 已完成；迁移后此处用于继续记录用户运行、观察和解释证据。\n",
        ),
        (
            "05-arch-analyzer",
            "guides/05-mechanism-deep-dive/README.md",
            "# Mechanism Deep Dive Migration Bridge\n\n旧 `05-arch-analyzer` 已完成；迁移后视为 `05-mechanism-deep-dive` 的历史输入。后续只围绕课程机制问题做最小源码/文档深挖。\n",
        ),
        (
            "09-biz-solver",
            "guides/06-practice-transfer/README.md",
            "# Practice Transfer Migration Bridge\n\n旧 `09-biz-solver` 已完成；迁移后视为 `06-practice-transfer` 的历史输入。后续迁移练习必须指向已验证课程概念。\n",
        ),
    ];
    for (old_stage, relative, content) in bridges {
        if old_statuses.get(old_stage).map(String::as_str) == Some("done") {
            write_if_missing(&topic_dir.join(relative), content)?;
        }
    }
    Ok(())
}

fn stage_statuses(doc: &DocumentMut) -> HashMap<String, String> {
    state_toml::stages(doc)
        .into_iter()
        .map(|stage| (stage.id, stage.status))
        .collect()
}

fn repo_stage_to_course_stage(stage_id: &str) -> &'static str {
    match stage_id {
        "01-goal-aligner" => "01-need-aligner",
        "02-repo-scout" => "02-syllabus-mapper",
        "03-socratic-coach" => "03-concept-roadmap",
        "04-debugger-guide" => "04-lesson-lab",
        "05-arch-analyzer" | "06-code-reader" => "05-mechanism-deep-dive",
        "07-demo-architecture" | "08-demo-coder" => "07-capstone-lab",
        "09-biz-solver" => "06-practice-transfer",
        "10-reflection" => "09-closeout-archive",
        _ => "01-need-aligner",
    }
}

fn course_stage_status(old_statuses: &HashMap<String, String>, new_stage: &str) -> String {
    let old_stage = match new_stage {
        "01-need-aligner" => "01-goal-aligner",
        "02-syllabus-mapper" => "02-repo-scout",
        "03-concept-roadmap" => "03-socratic-coach",
        "04-lesson-lab" => "04-debugger-guide",
        "05-mechanism-deep-dive" => "05-arch-analyzer",
        "06-practice-transfer" => "09-biz-solver",
        "07-capstone-lab" => "07-demo-architecture",
        "09-closeout-archive" => "10-reflection",
        _ => "",
    };
    let status = old_statuses
        .get(old_stage)
        .cloned()
        .unwrap_or_else(|| "pending".to_owned());
    if new_stage == "08-review-loop" {
        "pending".to_owned()
    } else {
        status
    }
}

fn course_stage_tables(old_statuses: &HashMap<String, String>) -> ArrayOfTables {
    let specs = [
        (
            "01-need-aligner",
            "对齐课程学习诉求",
            vec![".daedalus/task-card.md", ".daedalus/outcome-map.md"],
        ),
        (
            "02-syllabus-mapper",
            "映射课程 syllabus",
            vec!["guides/02-syllabus-mapper/README.md"],
        ),
        (
            "03-concept-roadmap",
            "建立概念路线图",
            vec![
                "guides/03-concept-roadmap/README.md",
                "../../shared/concept-map.md",
            ],
        ),
        (
            "04-lesson-lab",
            "把课程 lesson 变成可观察实验",
            vec![
                "guides/04-lesson-lab/README.md",
                "notes/04-lesson-lab/README.md",
            ],
        ),
        (
            "05-mechanism-deep-dive",
            "拆解被 API 隐藏的机制",
            vec!["guides/05-mechanism-deep-dive/README.md"],
        ),
        (
            "06-practice-transfer",
            "迁移课程概念到真实任务",
            vec!["guides/06-practice-transfer/README.md"],
        ),
        (
            "07-capstone-lab",
            "完成课程驱动 mini project",
            vec!["demo/design.md"],
        ),
        (
            "08-review-loop",
            "复习与掌握度验证",
            vec!["review/mastery-map.md", "review/question-bank.md"],
        ),
        (
            "09-closeout-archive",
            "专题回顾与知识归档",
            vec![
                ".daedalus/artifact-index.md",
                ".daedalus/long-context.md",
                "reflection/candidate-map.md",
                "reflection/closeout.md",
            ],
        ),
    ];
    let mut stages = ArrayOfTables::new();
    for (id, title, required_artifacts) in specs {
        let mut table = Table::new();
        table["id"] = value(id);
        table["title"] = value(title);
        table["status"] = value(course_stage_status(old_statuses, id));
        let mut artifacts = Array::new();
        for artifact in required_artifacts {
            artifacts.push(artifact);
        }
        table["required_artifacts"] = Item::Value(artifacts.into());
        stages.push(table);
    }
    stages
}

fn rewrite_course_project_markdown(project_dir: &Path, course_url: &str) -> Result<()> {
    replace_if_exists(
        &project_dir.join("CLAUDE.md"),
        &[
            (
                "Repo Learning Project Context",
                "Course Learning Project Context",
            ),
            ("repo learning project", "course learning project"),
            ("system/prompts/repo", "system/prompts/course"),
            (
                "daedalus init repo-learning <project-name> --topic <topic-slug> --title <topic-title>",
                "daedalus init course-learning <project-name> --topic <topic-slug> --title <topic-title> --course-url <url>",
            ),
        ],
    )?;
    replace_if_exists(
        &project_dir.join(".daedalus/state.toml"),
        &[
            (
                "daedalus repo learning project state",
                "daedalus course learning project state",
            ),
            (
                "Project state 只描述长期学习项目本身，不承载 10-stage 专题进度。",
                "Project state 只描述长期课程学习项目本身，不承载专题阶段进度。",
            ),
        ],
    )?;
    replace_if_exists(
        &project_dir.join(".daedalus/project-map.md"),
        &[
            ("repo learning project", "course learning project"),
            ("repo/source", "course/syllabus"),
            ("源码模块索引", "课程章节索引"),
            ("共享源码位置", "课程材料入口"),
        ],
    )?;
    replace_if_exists(
        &project_dir.join(".daedalus/topic-board.md"),
        &[("repo learning project", "course learning project")],
    )?;
    replace_if_exists(
        &project_dir.join(".daedalus/artifact-index.md"),
        &[
            (
                "shared/source-index.md`](../shared/source-index.md) | shared | 被学习 repo 的源码索引和稳定入口",
                "shared/syllabus-map.md`](../shared/syllabus-map.md) | shared | 课程 syllabus、章节依赖和取舍边界",
            ),
            (
                "专题学习目录；每个 topic 自己维护 10-stage 产物",
                "专题学习目录；每个 topic 自己维护 course-learning 阶段产物",
            ),
        ],
    )?;
    replace_if_exists(
        &project_dir.join("shared/README.md"),
        &[
            (
                "跨 topic 可复用的已验证知识。探索草稿、用户假设和专题内源码阅读",
                "跨 topic 可复用的已验证课程学习上下文。探索草稿、用户假设和专题内 lesson 观察",
            ),
            (
                "- [`runbook.md`](runbook.md)：跨 topic 复用的运行与调试入口。",
                "- [`syllabus-map.md`](syllabus-map.md)：课程章节、概念依赖和取舍边界。",
            ),
            (
                "- [`architecture-map.md`](architecture-map.md)：跨 topic 稳定架构边界。",
                "- [`course-progress.md`](course-progress.md)：章节进度、当前 checkpoint 和复习状态。",
            ),
            (
                "- [`source-index.md`](source-index.md)：源码模块索引。",
                "- [`concept-map.md`](concept-map.md)：跨章节核心概念、机制问题和薄弱点。",
            ),
        ],
    )?;
    let project_state = state_toml::load_state_doc(&state_toml::state_path(project_dir))?;
    if let Some(active_topic_state) = state_toml::active_topic(&project_state)
        .and_then(|slug| state_toml::topic_path(&project_state, &slug))
        .map(|path| project_dir.join(path).join(".daedalus/state.toml"))
    {
        replace_if_exists(
            &active_topic_state,
            &[
                (
                    "daedalus repo learning topic state",
                    "daedalus course learning topic state",
                ),
                (
                    "Topic state 承载 10-stage 专题学习进度",
                    "Topic state 承载 course-learning 专题学习进度",
                ),
            ],
        )?;
    }
    append_if_missing(
        &project_dir.join(".daedalus/project-map.md"),
        "shared/syllabus-map.md",
        &format!(
            "\n## Course Learning Migration\n\n- 课程主页：{course_url}\n- 新 shared 入口：[`shared/syllabus-map.md`](../shared/syllabus-map.md)、[`shared/course-progress.md`](../shared/course-progress.md)、[`shared/concept-map.md`](../shared/concept-map.md)\n- 历史 `guides/04-debugger-guide` 保留为迁移前现场；后续新 guide 使用 course-learning 阶段名。\n"
        ),
    )?;
    Ok(())
}

fn replace_if_exists(path: &Path, replacements: &[(&str, &str)]) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let mut content = fs::read_to_string(path).map_err(|source| DaedalusError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    for (from, to) in replacements {
        content = content.replace(from, to);
    }
    fs::write(path, content).map_err(|source| DaedalusError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn append_if_missing(path: &Path, marker: &str, addition: &str) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let mut content = fs::read_to_string(path).map_err(|source| DaedalusError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if !content.contains(marker) {
        content.push_str(addition);
        fs::write(path, content).map_err(|source| DaedalusError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    }
    Ok(())
}

fn write_if_missing(path: &Path, content: &str) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        workspace_fs::ensure_dir(parent)?;
    }
    fs::write(path, content).map_err(|source| DaedalusError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn validate_old_workspace(project_dir: &Path) -> Result<()> {
    let doc = state_toml::load_state_doc(&state_toml::state_path(project_dir))?;
    if state_toml::state_kind(&doc) == "project" || project_dir.join("topics").exists() {
        return Err(DaedalusError::InvalidStateDocumentKind {
            expected: "legacy-task".to_owned(),
            actual: "project-or-topic-layout".to_owned(),
        });
    }
    Ok(())
}

fn build_topic_doc(mut old_doc: DocumentMut, slug: &str, title: &str) -> DocumentMut {
    let current_phase = state_toml::current_phase(&old_doc).unwrap_or_else(|| {
        state_toml::stages(&old_doc)
            .first()
            .map(|stage| stage.id.clone())
            .unwrap_or_else(|| "01-goal-aligner".to_owned())
    });
    let next_action = state_toml::next_action(&old_doc);
    old_doc.as_table_mut().remove("task");
    let mut topic = Table::new();
    topic["slug"] = value(slug);
    topic["title"] = value(title);
    topic["kind"] = value("repo-learning-topic");
    topic["parent_project"] = value("../..");
    topic["lifecycle"] = value("active");
    topic["current_phase"] = value(current_phase);
    topic["next_action"] = value(next_action);
    old_doc["topic"] = Item::Table(topic);
    old_doc
}

fn backup_if_exists(project_dir: &Path, backup_dir: &Path, relative: &str) -> Result<()> {
    let source = project_dir.join(relative);
    if source.exists() {
        copy_dir(&source, &backup_dir.join(relative))?;
    }
    Ok(())
}

fn preserve_existing_files(
    project_dir: &Path,
    relatives: &[&str],
) -> Result<Vec<(PathBuf, String)>> {
    let mut preserved = Vec::new();
    for relative in relatives {
        let path = project_dir.join(relative);
        if path.exists() {
            let content = fs::read_to_string(&path).map_err(|source| DaedalusError::Io {
                path: path.clone(),
                source,
            })?;
            preserved.push((PathBuf::from(relative), content));
        }
    }
    Ok(preserved)
}

fn restore_preserved_files(project_dir: &Path, preserved: Vec<(PathBuf, String)>) -> Result<()> {
    for (relative, content) in preserved {
        let path = project_dir.join(relative);
        if let Some(parent) = path.parent() {
            workspace_fs::ensure_dir(parent)?;
        }
        fs::write(&path, content).map_err(|source| DaedalusError::Io { path, source })?;
    }
    Ok(())
}

fn copy_dir(source: &Path, target: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(source) {
        let entry = entry.map_err(|source_err| DaedalusError::Io {
            path: source.to_path_buf(),
            source: std::io::Error::other(source_err),
        })?;
        let relative = entry.path().strip_prefix(source).unwrap_or(entry.path());
        let target_path = target.join(relative);
        if entry.file_type().is_dir() {
            workspace_fs::ensure_dir(&target_path)?;
        } else {
            if let Some(parent) = target_path.parent() {
                workspace_fs::ensure_dir(parent)?;
            }
            fs::copy(entry.path(), &target_path).map_err(|source_err| DaedalusError::Io {
                path: target_path,
                source: source_err,
            })?;
        }
    }
    Ok(())
}

fn move_dir_if_exists(project_dir: &Path, topic_dir: &Path, name: &str) -> Result<()> {
    let source = project_dir.join(name);
    if source.exists() {
        let target = topic_dir.join(name);
        if target.exists() {
            fs::remove_dir_all(&target).map_err(|source_err| DaedalusError::Io {
                path: target.clone(),
                source: source_err,
            })?;
        }
        fs::rename(&source, &target).map_err(|source_err| DaedalusError::Io {
            path: target,
            source: source_err,
        })?;
    }
    Ok(())
}

fn move_file_if_exists(source: &Path, target: &Path) -> Result<()> {
    if !source.exists() {
        return Ok(());
    }
    if let Some(parent) = target.parent() {
        workspace_fs::ensure_dir(parent)?;
    }
    if target.exists() {
        fs::remove_file(target).map_err(|source_err| DaedalusError::Io {
            path: target.to_path_buf(),
            source: source_err,
        })?;
    }
    fs::rename(source, target).map_err(|source_err| DaedalusError::Io {
        path: target.to_path_buf(),
        source: source_err,
    })
}

fn sanitize_slug(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let sanitized = sanitized.trim_matches('-');
    if sanitized.is_empty() {
        "topic".to_owned()
    } else {
        sanitized.to_owned()
    }
}
