use std::path::{Path, PathBuf};

use crate::application::render::render_state;
use crate::domain::{DaedalusError, Result};
use crate::infrastructure::{clock, template_fs, workspace_fs};

/// 初始化 repo learning 任务所需的参数。
#[derive(Debug, Clone)]
pub struct InitTaskOptions {
    /// daedalus 项目根目录。
    pub repo_root: PathBuf,
    /// 用户提供的学习任务名称。
    pub name: String,
    /// 初始专题 slug。
    pub topic_slug: String,
    /// 初始专题标题。
    pub topic_title: String,
    /// 是否允许在已有 active task 时继续创建任务。
    pub allow_existing_active: bool,
    /// 使用绕过选项时必须提供的具体原因。
    pub reason: Option<String>,
}

/// 初始化学习任务后的输出。
#[derive(Debug, Clone)]
pub struct InitTaskOutput {
    /// 新建学习任务目录。
    pub task_dir: PathBuf,
    /// 新建初始专题目录。
    pub topic_dir: PathBuf,
    /// 生成后的 `.daedalus/state.md` 路径。
    pub state_md: PathBuf,
    /// 生成后的 topic `.daedalus/state.md` 路径。
    pub topic_state_md: PathBuf,
}

/// 初始化一个 repo learning project 和初始 topic。
///
/// 该 use case 会复制 project 模板、创建初始 topic，并立即分别渲染 project/topic
/// `state.md`。
pub fn init_repo_learning(options: InitTaskOptions) -> Result<InitTaskOutput> {
    let learning_root = workspace_fs::learning_root(&options.repo_root);
    workspace_fs::ensure_dir(&learning_root)?;

    let active = workspace_fs::active_tasks(&learning_root)?;
    if !active.is_empty() && !options.allow_existing_active {
        return Err(DaedalusError::TaskAlreadyActive(active[0].clone()));
    }
    if options.allow_existing_active && !specific_reason(options.reason.as_deref()) {
        return Err(DaedalusError::ForceRequiresApproval);
    }

    let task_dir = unique_task_dir(&learning_root, &options.name);
    workspace_fs::ensure_dir(&task_dir)?;
    workspace_fs::ensure_dir(&task_dir.join("source"))?;
    workspace_fs::ensure_dir(&task_dir.join("shared"))?;
    workspace_fs::ensure_dir(&task_dir.join("topics"))?;

    let created_at = clock::now_local_timestamp();
    let topic_slug = sanitize_name(&options.topic_slug);
    let topic_title = if options.topic_title.trim().is_empty() {
        topic_slug.clone()
    } else {
        options.topic_title.trim().to_owned()
    };
    let template_dir = options
        .repo_root
        .join("system")
        .join("templates")
        .join("repo");
    template_fs::copy_template_dir(
        &template_dir,
        &task_dir,
        &[
            ("{{TASK_NAME}}", &options.name),
            ("{{CREATED_AT}}", &created_at),
            ("{{TOPIC_SLUG}}", &topic_slug),
            ("{{TOPIC_TITLE}}", &topic_title),
        ],
    )?;
    let topic_dir = task_dir.join("topics").join(&topic_slug);
    workspace_fs::ensure_dir(&topic_dir)?;
    let topic_template_dir = options
        .repo_root
        .join("system")
        .join("templates")
        .join("repo-topic");
    template_fs::copy_template_dir(
        &topic_template_dir,
        &topic_dir,
        &[
            ("{{TASK_NAME}}", &options.name),
            ("{{CREATED_AT}}", &created_at),
            ("{{TOPIC_SLUG}}", &topic_slug),
            ("{{TOPIC_TITLE}}", &topic_title),
        ],
    )?;
    let state_md = render_state(&task_dir)?.path;
    let topic_state_md = render_state(&topic_dir)?.path;

    Ok(InitTaskOutput {
        task_dir,
        topic_dir,
        state_md,
        topic_state_md,
    })
}

fn unique_task_dir(root: &Path, name: &str) -> PathBuf {
    let sanitized = sanitize_name(name);
    let candidate = root.join(&sanitized);
    if !candidate.exists() {
        return candidate;
    }

    for idx in 2.. {
        let candidate = root.join(format!("{sanitized}-{idx}"));
        if !candidate.exists() {
            return candidate;
        }
    }

    unreachable!("infinite iterator should always return a candidate")
}

fn sanitize_name(name: &str) -> String {
    let sanitized: String = name
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
        "learning-task".to_owned()
    } else {
        sanitized.to_owned()
    }
}

fn specific_reason(reason: Option<&str>) -> bool {
    reason
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter(|value| !matches!(*value, "continue" | "skip" | "not needed"))
        .is_some()
}
