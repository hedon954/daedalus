use std::fs;
use std::path::{Path, PathBuf};

use toml_edit::DocumentMut;

use crate::domain::{DaedalusError, Result};
use crate::infrastructure::state_toml;

const RECENT_TRANSITION_LIMIT: usize = 10;

/// `state.md` 渲染结果。
#[derive(Debug, Clone)]
pub struct RenderedState {
    /// 写入的 `state.md` 文件路径。
    pub path: PathBuf,
    /// 渲染后的 Markdown 内容。
    pub content: String,
}

/// 从学习任务目录中的 `.daedalus/state.toml` 生成 `.daedalus/state.md`。
pub fn render_state(task_dir: &Path) -> Result<RenderedState> {
    let path = state_toml::state_path(task_dir);
    let doc = state_toml::load_state_doc(&path)?;
    let content = render_state_markdown(&doc, task_dir)?;
    let output_path = state_toml::state_md_path(task_dir);
    fs::write(&output_path, &content).map_err(|source| DaedalusError::Io {
        path: output_path.clone(),
        source,
    })?;
    Ok(RenderedState {
        path: output_path,
        content,
    })
}

/// 将已解析的状态文档渲染成面向 Agent 阅读的中文 Markdown。
///
/// 该函数保持确定性：相同的 `state.toml` 和相同的文件系统产物状态，
/// 应生成相同的 Markdown 内容。
pub fn render_state_markdown(doc: &DocumentMut, task_dir: &Path) -> Result<String> {
    match state_toml::state_kind(doc) {
        "project" => render_project_state_markdown(doc, task_dir),
        "topic" => render_topic_state_markdown(doc, task_dir),
        _ => render_topic_state_markdown(doc, task_dir),
    }
}

fn render_project_state_markdown(doc: &DocumentMut, task_dir: &Path) -> Result<String> {
    let mut output = String::new();
    output.push_str("# Project 状态\n\n");
    output.push_str("> 从 [`.daedalus/state.toml`](state.toml) 生成。不要手动编辑。\n\n");
    output.push_str("## 当前状态\n\n");
    output.push_str(&format!("- Project：`{}`\n", state_toml::task_name(doc)));
    output.push_str(&format!("- 类型：`{}`\n", state_toml::task_kind(doc)));
    output.push_str(&format!(
        "- 生命周期：`{}`\n",
        state_toml::task_lifecycle(doc)
            .map(|lifecycle| lifecycle.as_str().to_owned())
            .unwrap_or_else(|_| "unknown".to_owned())
    ));
    output.push_str(&format!(
        "- Workspace Bucket：`{}`\n",
        state_toml::workspace_bucket(doc)
            .map(|bucket| bucket.as_str().to_owned())
            .unwrap_or_else(|_| "unknown".to_owned())
    ));
    output.push_str(&format!(
        "- Active Topic：`{}`\n",
        state_toml::active_topic(doc).unwrap_or_else(|| "none".to_owned())
    ));
    output.push_str(&format!(
        "- Pending Closeout：`{}`\n",
        state_toml::awaiting_reflection_topic(doc).unwrap_or_else(|| "none".to_owned())
    ));
    if let Some(course_url) = state_toml::course_url(doc) {
        output.push_str(&format!("- Course URL：{course_url}\n"));
    }
    output.push_str(&format!("- 下一步：{}\n\n", state_toml::next_action(doc)));

    output.push_str("## Topics\n\n");
    let topics = state_toml::topics(doc);
    if topics.is_empty() {
        output.push_str("- 无\n");
    } else {
        for topic in topics {
            output.push_str(&format!(
                "- `{}`: {} ({}) -> [`{}`](../{})\n",
                topic.slug, topic.title, topic.lifecycle, topic.path, topic.path
            ));
        }
    }

    output.push_str("\n## Project Files\n\n");
    let project_files: Vec<&str> = match state_toml::task_kind(doc).as_str() {
        "course-learning" => vec![
            ".daedalus/project-map.md",
            ".daedalus/topic-board.md",
            "shared/syllabus-map.md",
            "shared/course-progress.md",
            "shared/concept-map.md",
            "shared/evidence-registry.md",
        ],
        _ => vec![
            ".daedalus/project-map.md",
            ".daedalus/topic-board.md",
            "shared/evidence-registry.md",
            "shared/source-index.md",
        ],
    };
    for path in project_files {
        let status = if task_dir.join(path).exists() {
            "present"
        } else {
            "missing"
        };
        output.push_str(&format!("- `{path}`: {status}\n"));
    }

    output.push_str("\n## 最近状态流转\n\n");
    render_transitions(doc, &mut output);
    output.push_str("\n## 下一步 CLI 建议\n\n");
    output.push_str("- `daedalus state render --project`\n");
    output.push_str("- `daedalus validate`\n");
    Ok(output)
}

fn render_topic_state_markdown(doc: &DocumentMut, task_dir: &Path) -> Result<String> {
    let current_phase = state_toml::current_phase(doc).unwrap_or_else(|| "unknown".to_owned());
    let stages = state_toml::stages(doc);
    let current_stage = stages.iter().find(|stage| stage.id == current_phase);

    let mut output = String::new();
    output.push_str("# Topic 学习状态\n\n");
    output.push_str("> 从 [`.daedalus/state.toml`](state.toml) 生成。不要手动编辑。\n\n");
    output.push_str("## 当前状态\n\n");
    output.push_str(&format!(
        "- Topic：`{}` - {}\n",
        state_toml::topic_slug(doc),
        state_toml::topic_title(doc)
    ));
    output.push_str(&format!(
        "- 生命周期：`{}`\n",
        state_toml::topic_lifecycle(doc)
            .map(|lifecycle| lifecycle.as_str().to_owned())
            .unwrap_or_else(|_| "unknown".to_owned())
    ));
    output.push_str(&format!("- 当前阶段：`{}`\n", current_phase));
    output.push_str(&format!(
        "- 状态：`{}`\n",
        current_stage
            .map(|stage| stage.status.as_str())
            .unwrap_or("unknown")
    ));
    output.push_str(&format!("- 下一步：{}\n\n", state_toml::next_action(doc)));

    output.push_str("## 枚举约束\n\n");
    output.push_str("- `topic.lifecycle` 只能是：`planned`、`active`、`blocked`、`awaiting-reflection`、`completed`、`abandoned`、`skipped`。\n");
    output
        .push_str("- `stage.status` 只能是：`pending`、`active`、`blocked`、`paused`、`done`。\n");
    output.push_str("- `transition.action` 只能是：`init`、`enter`、`complete`、`block`、`resume`、`rollback`、`topic-await-reflection`、`topic-complete`、`topic-abandon`。\n");
    output.push_str("- `transition.approval_source` 只能是：`user-confirmed`、`artifact-equivalent`、`stage-not-applicable`。\n");
    output.push_str("- Agent 不要发明新的枚举值；如需新增，先修改 Rust 领域模型、模板和测试。\n\n");

    output.push_str("## 阶段进度\n\n");
    for stage in &stages {
        output.push_str(&format!(
            "- `{}`: {} ({})\n",
            stage.id, stage.title, stage.status
        ));
    }

    output.push_str("\n## 缺失产物\n\n");
    let mut missing_any = false;
    for stage in &stages {
        for artifact in &stage.required_artifacts {
            if !task_dir.join(artifact).exists() {
                missing_any = true;
                output.push_str(&format!(
                    "- {} 属于 `{}`\n",
                    artifact_markdown_link(artifact),
                    stage.id
                ));
            }
        }
    }
    if !missing_any {
        output.push_str("- 无\n");
    }

    output.push_str("\n## 阻塞项\n\n");
    let blockers: Vec<_> = stages
        .iter()
        .filter(|stage| stage.status == "blocked")
        .collect();
    if blockers.is_empty() {
        output.push_str("- 无\n");
    } else {
        for stage in blockers {
            output.push_str(&format!("- `{}`: {}\n", stage.id, stage.title));
        }
    }

    output.push_str("\n## 最近状态流转\n\n");
    render_transitions(doc, &mut output);

    output.push_str("\n## 下一步 CLI 建议\n\n");
    output.push_str(&format!(
        "- `daedalus state render --topic-dir {}`\n",
        task_dir.display()
    ));
    output.push_str("- 完成阶段前先运行 `daedalus validate`。\n");

    Ok(output)
}

fn render_transitions(doc: &DocumentMut, output: &mut String) {
    let transitions = state_toml::transitions(doc);
    if transitions.is_empty() {
        output.push_str("- 无\n");
        return;
    }
    let start = transitions.len().saturating_sub(RECENT_TRANSITION_LIMIT);
    let shown = transitions.len() - start;
    output.push_str(&format!(
        "> 共 {} 条状态流转；下面显示最近 {} 条，完整历史见 [`.daedalus/state.toml`](state.toml) 的 `[[transitions]]`。\n\n",
        transitions.len(),
        shown
    ));
    for transition in &transitions[start..] {
        let approval = transition
            .approval_source
            .as_ref()
            .map(|source| format!("，批准来源：`{source}`"))
            .unwrap_or_default();
        output.push_str(&format!(
            "- `{}` 由 `{}` 对 `{}` 执行 `{}`：{}{}\n",
            transition.timestamp,
            transition.actor,
            transition.stage,
            transition.action,
            transition.reason,
            approval
        ));
    }
}

fn artifact_markdown_link(path: &str) -> String {
    let href = path
        .strip_prefix(".daedalus/")
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("../{path}"));
    format!("[`{path}`]({href})")
}
