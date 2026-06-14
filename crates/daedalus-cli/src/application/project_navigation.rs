use std::fs;
use std::path::Path;

use toml_edit::DocumentMut;

use crate::domain::{DaedalusError, Result, TopicSnapshot};
use crate::infrastructure::{state_toml, workspace_fs};

/// Synchronize project-level navigation Markdown from project `state.toml`.
///
/// `state.toml` remains the lifecycle source of truth. These Markdown files are
/// learner-facing maps, so the CLI keeps their machine-checkable status sections
/// aligned with the source of truth after lifecycle transitions.
pub fn sync_project_navigation(project_dir: &Path) -> Result<()> {
    let state_path = state_toml::state_path(project_dir);
    let doc = state_toml::load_state_doc(&state_path)?;
    sync_project_map(project_dir, &doc)?;
    sync_topic_board(project_dir, &doc)?;
    Ok(())
}

/// Validate that project navigation Markdown still matches project `state.toml`.
pub fn validate_project_navigation(project_dir: &Path, doc: &DocumentMut) -> Result<Vec<String>> {
    let mut issues = Vec::new();

    let project_map_path = project_dir.join(".daedalus/project-map.md");
    if project_map_path.exists() {
        let content = read_to_string(&project_map_path)?;
        let expected_line = current_active_topic_line(doc);
        if !content.contains(&expected_line) {
            issues.push(format!(
                "project-map.md active topic line is stale; expected `{expected_line}`"
            ));
        }
        let expected_section = project_map_active_topic_section(doc);
        if !content.contains(expected_section.trim()) {
            issues.push("project-map.md Active Topic section is stale".to_owned());
        }
    }

    let topic_board_path = project_dir.join(".daedalus/topic-board.md");
    if topic_board_path.exists() {
        let content = read_to_string(&topic_board_path)?;
        let expected_section = topic_board_active_topic_section(doc);
        if !content.contains(expected_section.trim()) {
            issues.push("topic-board.md Active Topic section is stale".to_owned());
        }
        for row in topic_rows(doc) {
            if !content.contains(&row) {
                issues.push(format!(
                    "topic-board.md topic row is stale or missing: {row}"
                ));
            }
        }
    }

    Ok(issues)
}

fn sync_project_map(project_dir: &Path, doc: &DocumentMut) -> Result<()> {
    let path = project_dir.join(".daedalus/project-map.md");
    let mut content = read_to_string(&path)?;
    content = replace_line_with_prefix(
        content,
        "- 当前 active topic：",
        &current_active_topic_line(doc),
    );
    content = replace_markdown_section(
        &content,
        "## Active Topic",
        &project_map_active_topic_section(doc),
    );
    write_string(&path, &content)
}

fn sync_topic_board(project_dir: &Path, doc: &DocumentMut) -> Result<()> {
    let path = project_dir.join(".daedalus/topic-board.md");
    let mut content = read_to_string(&path)?;
    content = replace_markdown_section(
        &content,
        "## Active Topic",
        &topic_board_active_topic_section(doc),
    );
    content = replace_markdown_section(&content, "## Topics", &topic_board_topics_section(doc));
    write_string(&path, &content)
}

fn current_active_topic_line(doc: &DocumentMut) -> String {
    format!(
        "- 当前 active topic：`{}`",
        state_toml::active_topic(doc).unwrap_or_else(|| "none".to_owned())
    )
}

fn project_map_active_topic_section(doc: &DocumentMut) -> String {
    let body = if let Some(topic) = active_topic_snapshot(doc) {
        format!(
            "- slug：`{}`\n- title：{}\n- topic workspace：[`{}`](../{})\n- topic outcome map：[`{}/.daedalus/outcome-map.md`](../{}/.daedalus/outcome-map.md)\n",
            topic.slug, topic.title, topic.path, topic.path, topic.path, topic.path
        )
    } else {
        "- 无 active topic。\n- 下一步：复盘 project，或用 `daedalus topic new` / `daedalus topic activate` 启动新专题。\n".to_owned()
    };
    format!("## Active Topic\n\n{body}")
}

fn topic_board_active_topic_section(doc: &DocumentMut) -> String {
    let body = if let Some(topic) = active_topic_snapshot(doc) {
        format!("- `{}`：{}\n", topic.slug, topic.title)
    } else {
        "- 无 active topic。\n".to_owned()
    };
    format!("## Active Topic\n\n{body}")
}

fn topic_board_topics_section(doc: &DocumentMut) -> String {
    let mut output = String::from(
        "## Topics\n\n| Topic | Title | Lifecycle | Path | Inherits |\n| --- | --- | --- | --- | --- |\n",
    );
    for row in topic_rows(doc) {
        output.push_str(&row);
        output.push('\n');
    }
    output
}

fn topic_rows(doc: &DocumentMut) -> Vec<String> {
    state_toml::topics(doc)
        .into_iter()
        .map(|topic| {
            let inherits = if topic.inherits.is_empty() {
                "-".to_owned()
            } else {
                topic.inherits.join(", ")
            };
            format!(
                "| `{}` | {} | {} | [`{}`](../{}) | {} |",
                topic.slug, topic.title, topic.lifecycle, topic.path, topic.path, inherits
            )
        })
        .collect()
}

fn active_topic_snapshot(doc: &DocumentMut) -> Option<TopicSnapshot> {
    let active = state_toml::active_topic(doc)?;
    state_toml::topics(doc)
        .into_iter()
        .find(|topic| topic.slug == active)
}

fn replace_line_with_prefix(content: String, prefix: &str, replacement: &str) -> String {
    let mut replaced = false;
    let mut lines = Vec::new();
    for line in content.lines() {
        if line.starts_with(prefix) {
            lines.push(replacement.to_owned());
            replaced = true;
        } else {
            lines.push(line.to_owned());
        }
    }
    let mut output = lines.join("\n");
    if content.ends_with('\n') {
        output.push('\n');
    }
    if replaced {
        output
    } else {
        format!("{output}\n{replacement}\n")
    }
}

fn replace_markdown_section(content: &str, header: &str, replacement: &str) -> String {
    let Some(start) = content.find(header) else {
        let mut output = content.trim_end().to_owned();
        output.push_str("\n\n");
        output.push_str(replacement.trim_end());
        output.push('\n');
        return output;
    };
    let after_header = start + header.len();
    let end = content[after_header..]
        .find("\n## ")
        .map(|index| after_header + index + 1)
        .unwrap_or(content.len());

    let mut output = String::new();
    output.push_str(content[..start].trim_end());
    output.push_str("\n\n");
    output.push_str(replacement.trim_end());
    output.push_str("\n\n");
    output.push_str(content[end..].trim_start());
    output
}

fn read_to_string(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|source| DaedalusError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn write_string(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        workspace_fs::ensure_dir(parent)?;
    }
    fs::write(path, content).map_err(|source| DaedalusError::Io {
        path: path.to_path_buf(),
        source,
    })
}
