use std::fs;
use std::path::{Path, PathBuf};

use toml_edit::{DocumentMut, Item, Table, value};

use crate::domain::{
    DaedalusError, Result, ReviewLifecycle, ReviewMode, ReviewSnapshot, ReviewTarget,
};
use crate::infrastructure::{clock, state_toml, template_fs, workspace_fs};

/// 创建 review plan 的参数。
#[derive(Debug, Clone)]
pub struct StartReviewOptions {
    pub repo_root: PathBuf,
    pub project_dir: PathBuf,
    pub target: ReviewTarget,
    pub mode: ReviewMode,
    pub goal: String,
    pub actor: String,
}

/// review 操作输出。
#[derive(Debug, Clone)]
pub struct ReviewOutput {
    pub project_dir: PathBuf,
    pub review_dir: PathBuf,
    pub review_id: String,
    pub action: String,
    pub state_md: Option<PathBuf>,
}

/// review 列表输出。
#[derive(Debug, Clone)]
pub struct ReviewListOutput {
    pub project_dir: PathBuf,
    pub reviews: Vec<ReviewSnapshot>,
}

/// review session 操作输出。
#[derive(Debug, Clone)]
pub struct ReviewSessionOutput {
    pub project_dir: PathBuf,
    pub review_dir: PathBuf,
    pub review_id: String,
    pub session_id: String,
    pub session_path: PathBuf,
    pub action: String,
}

/// review 校验输出。
#[derive(Debug, Clone)]
pub struct ReviewValidationOutput {
    pub project_dir: PathBuf,
    pub review_dir: PathBuf,
    pub review_id: String,
    pub issues: Vec<String>,
}

impl ReviewValidationOutput {
    pub fn is_ok(&self) -> bool {
        self.issues.is_empty()
    }
}

/// review session start 参数。
#[derive(Debug, Clone)]
pub struct StartReviewSessionOptions {
    pub project_dir: PathBuf,
    pub review_id: String,
    pub session_id: Option<String>,
    pub actor: String,
}

/// review session complete 参数。
#[derive(Debug, Clone)]
pub struct CompleteReviewSessionOptions {
    pub project_dir: PathBuf,
    pub review_id: String,
    pub session_id: Option<String>,
    pub reason: String,
    pub actor: String,
}

/// review close 参数。
#[derive(Debug, Clone)]
pub struct CloseReviewOptions {
    pub project_dir: PathBuf,
    pub review_id: String,
    pub lifecycle: ReviewLifecycle,
    pub reason: String,
    pub actor: String,
}

/// 创建 review plan。
pub fn start_review(options: StartReviewOptions) -> Result<ReviewOutput> {
    let project_doc = state_toml::load_state_doc(&state_toml::state_path(&options.project_dir))?;
    let target_label = review_target_label(&project_doc, &options.target);
    let target_dir = review_target_dir(&options.project_dir, &project_doc, &options.target)?;
    let reviews_root = target_dir.join(".daedalus").join("reviews");
    ensure_reviews_root(&reviews_root, &target_label)?;

    let base_id = format!(
        "{}-{}-{}",
        clock::now_local_timestamp()
            .chars()
            .take(10)
            .collect::<String>(),
        sanitize_slug(&target_label),
        options.mode.as_str()
    );
    let review_dir = unique_review_dir(&reviews_root, &base_id);
    let review_id = review_dir
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(&base_id)
        .to_owned();

    let template_dir = options
        .repo_root
        .join("system")
        .join("templates")
        .join("review");
    let created_at = clock::now_local_timestamp();
    let (target_type, target_name) = target_parts(&options.target, &target_label);
    template_fs::copy_template_dir(
        &template_dir,
        &review_dir,
        &[
            ("{{review_id}}", &review_id),
            ("{{target_type}}", target_type),
            ("{{target}}", &target_name),
            ("{{review_mode}}", options.mode.as_str()),
            ("{{review_goal}}", options.goal.trim()),
            ("{{created_at}}", &created_at),
        ],
    )?;

    let mut doc = load_review_doc(&review_dir)?;
    append_review_transition(&mut doc, "start", &options.actor, "初始化 review plan。");
    save_review_doc(&review_dir, &doc)?;
    let state_md = render_review_state(&review_dir)?.state_md;

    Ok(ReviewOutput {
        project_dir: options.project_dir,
        review_dir,
        review_id,
        action: "review-start".to_owned(),
        state_md: Some(state_md),
    })
}

/// 列出 project 下所有 review。
pub fn list_reviews(project_dir: PathBuf) -> Result<ReviewListOutput> {
    let reviews = collect_review_snapshots(&project_dir)?;
    Ok(ReviewListOutput {
        project_dir,
        reviews,
    })
}

/// 展示 review。
pub fn show_review(project_dir: PathBuf, review_id: String) -> Result<ReviewOutput> {
    let review_dir = find_review_dir(&project_dir, &review_id)?;
    Ok(ReviewOutput {
        project_dir,
        review_dir,
        review_id,
        action: "review-show".to_owned(),
        state_md: None,
    })
}

/// 开始一次 review session。
pub fn start_review_session(options: StartReviewSessionOptions) -> Result<ReviewSessionOutput> {
    let review_dir = find_review_dir(&options.project_dir, &options.review_id)?;
    let mut doc = load_review_doc(&review_dir)?;
    let session_id = options.session_id.unwrap_or_else(|| {
        format!(
            "{}-session-{}",
            clock::now_local_timestamp()
                .chars()
                .take(10)
                .collect::<String>(),
            next_session_number(&doc)
        )
    });
    let session_path = review_dir
        .join("sessions")
        .join(format!("{}.md", sanitize_slug(&session_id)));
    if session_path.exists() {
        return Err(DaedalusError::InvalidReviewOperation(format!(
            "review session already exists: {session_id}"
        )));
    }
    let template = review_dir.join("sessions").join("session-template.md");
    let content = fs::read_to_string(&template).map_err(|source| DaedalusError::Io {
        path: template.clone(),
        source,
    })?;
    fs::write(&session_path, content).map_err(|source| DaedalusError::Io {
        path: session_path.clone(),
        source,
    })?;

    append_session(&mut doc, &session_id, &session_path, "active");
    set_review_next_action(
        &mut doc,
        &format!("完成 review session `{session_id}`：补齐 User Answers 与 Calibration。"),
    );
    append_review_transition(
        &mut doc,
        "session-start",
        &options.actor,
        &format!("Start review session `{session_id}`."),
    );
    save_review_doc(&review_dir, &doc)?;
    let _ = render_review_state(&review_dir)?;

    Ok(ReviewSessionOutput {
        project_dir: options.project_dir,
        review_dir,
        review_id: options.review_id,
        session_id,
        session_path,
        action: "review-session-start".to_owned(),
    })
}

/// 完成一次 review session。
pub fn complete_review_session(
    options: CompleteReviewSessionOptions,
) -> Result<ReviewSessionOutput> {
    let reason = options.reason.trim();
    if reason.is_empty() {
        return Err(DaedalusError::TaskLifecycleReasonRequired);
    }
    let review_dir = find_review_dir(&options.project_dir, &options.review_id)?;
    let mut doc = load_review_doc(&review_dir)?;
    let session_id = options
        .session_id
        .or_else(|| active_session_id(&doc))
        .ok_or_else(|| {
            DaedalusError::InvalidReviewOperation("no active review session found".to_owned())
        })?;
    let session_path = update_session_status(&mut doc, &session_id, "completed")?;
    set_review_next_action(
        &mut doc,
        "更新 mastery-map.md，并根据薄弱点决定下一次 review session。",
    );
    append_review_transition(
        &mut doc,
        "session-complete",
        &options.actor,
        &format!("Complete review session `{session_id}`: {reason}"),
    );
    save_review_doc(&review_dir, &doc)?;
    let _ = render_review_state(&review_dir)?;

    let absolute_session_path = review_dir.join(session_path);
    Ok(ReviewSessionOutput {
        project_dir: options.project_dir,
        review_dir: review_dir.clone(),
        review_id: options.review_id,
        session_id,
        session_path: absolute_session_path,
        action: "review-session-complete".to_owned(),
    })
}

/// 完成或放弃 review。
pub fn close_review(options: CloseReviewOptions) -> Result<ReviewOutput> {
    let reason = options.reason.trim();
    if reason.is_empty() {
        return Err(DaedalusError::TaskLifecycleReasonRequired);
    }
    let review_dir = find_review_dir(&options.project_dir, &options.review_id)?;
    if options.lifecycle == ReviewLifecycle::Completed {
        let issues = validate_review_dir_with_lifecycle(
            &options.project_dir,
            &review_dir,
            Some("completed"),
        )?;
        if !issues.is_empty() {
            return Err(DaedalusError::WorkspaceValidationFailed(issues));
        }
    }
    let mut doc = load_review_doc(&review_dir)?;
    set_review_lifecycle(&mut doc, options.lifecycle);
    set_review_next_action(
        &mut doc,
        match options.lifecycle {
            ReviewLifecycle::Completed => "Review completed. Promote verified deltas if needed.",
            ReviewLifecycle::Abandoned => {
                "Review abandoned. Preserve existing sessions as context."
            }
            _ => "Review lifecycle changed.",
        },
    );
    append_review_transition(
        &mut doc,
        match options.lifecycle {
            ReviewLifecycle::Completed => "complete",
            ReviewLifecycle::Abandoned => "abandon",
            _ => "close",
        },
        &options.actor,
        reason,
    );
    save_review_doc(&review_dir, &doc)?;
    let state_md = render_review_state(&review_dir)?.state_md;
    Ok(ReviewOutput {
        project_dir: options.project_dir,
        review_dir,
        review_id: options.review_id,
        action: "review-close".to_owned(),
        state_md: Some(state_md),
    })
}

/// 渲染 review state.md。
pub fn render_review(project_dir: PathBuf, review_id: String) -> Result<ReviewOutput> {
    let review_dir = find_review_dir(&project_dir, &review_id)?;
    let state_md = render_review_state(&review_dir)?.state_md;
    Ok(ReviewOutput {
        project_dir,
        review_dir,
        review_id,
        action: "review-render".to_owned(),
        state_md: Some(state_md),
    })
}

/// 校验单个 review。
pub fn validate_review(project_dir: PathBuf, review_id: String) -> Result<ReviewValidationOutput> {
    let review_dir = find_review_dir(&project_dir, &review_id)?;
    let issues = validate_review_dir(&project_dir, &review_dir)?;
    Ok(ReviewValidationOutput {
        project_dir,
        review_dir,
        review_id,
        issues,
    })
}

/// 校验 project 下所有 review。
pub fn validate_all_reviews(project_dir: &Path) -> Result<Vec<String>> {
    let mut issues = Vec::new();
    for review in collect_review_snapshots(project_dir)? {
        let review_dir = project_dir.join(&review.path);
        issues.extend(validate_review_dir(project_dir, &review_dir)?);
    }
    Ok(issues)
}

#[derive(Debug, Clone)]
pub struct RenderedReviewState {
    pub state_md: PathBuf,
}

fn render_review_state(review_dir: &Path) -> Result<RenderedReviewState> {
    let doc = load_review_doc(review_dir)?;
    let output_path = review_dir.join("state.md");
    let content = render_review_markdown(&doc);
    fs::write(&output_path, content).map_err(|source| DaedalusError::Io {
        path: output_path.clone(),
        source,
    })?;
    Ok(RenderedReviewState {
        state_md: output_path,
    })
}

fn render_review_markdown(doc: &DocumentMut) -> String {
    let mut output = String::new();
    output.push_str("# Review State\n\n");
    output.push_str("> 从 [`state.toml`](state.toml) 生成。不要手动编辑。\n\n");
    output.push_str("## Current Review\n\n");
    output.push_str(&format!("- Review：`{}`\n", review_field(doc, "id")));
    output.push_str(&format!(
        "- Target：`{}` `{}`\n",
        review_field(doc, "target_type"),
        review_field(doc, "target")
    ));
    output.push_str(&format!("- Mode：`{}`\n", review_field(doc, "mode")));
    output.push_str(&format!(
        "- Lifecycle：`{}`\n",
        review_field(doc, "lifecycle")
    ));
    output.push_str(&format!("- Goal：{}\n", review_field(doc, "goal")));
    output.push_str(&format!("- Next：{}\n\n", review_field(doc, "next_action")));
    output.push_str("## First-Principles Review Chain\n\n");
    output.push_str("```text\n");
    output.push_str("业务目标 / 现实任务\n");
    output.push_str("  -> 现实制约\n");
    output.push_str("  -> naive solution 为什么失败\n");
    output.push_str("  -> 核心抽象 / 不变量\n");
    output.push_str("  -> 实现机制\n");
    output.push_str("  -> trade-off\n");
    output.push_str("  -> 对比最佳实践\n");
    output.push_str("  -> 可迁移模式\n");
    output.push_str("  -> 复习题 / 应用题\n");
    output.push_str("```\n\n");
    output.push_str("## Sessions\n\n");
    let sessions = session_rows(doc);
    if sessions.is_empty() {
        output.push_str("- 无\n");
    } else {
        for (id, status, path) in sessions {
            output.push_str(&format!("- `{id}` ({status}) -> [`{path}`]({path})\n"));
        }
    }
    output
}

fn validate_review_dir(project_dir: &Path, review_dir: &Path) -> Result<Vec<String>> {
    validate_review_dir_with_lifecycle(project_dir, review_dir, None)
}

fn validate_review_dir_with_lifecycle(
    project_dir: &Path,
    review_dir: &Path,
    lifecycle_override: Option<&str>,
) -> Result<Vec<String>> {
    let mut issues = Vec::new();
    for path in [
        "state.toml",
        "review-plan.md",
        "mastery-map.md",
        "question-bank.md",
        "sessions",
    ] {
        if !review_dir.join(path).exists() {
            issues.push(format!(
                "review `{}` missing required path: {path}",
                review_dir_label(review_dir)
            ));
        }
    }
    let doc = match load_review_doc(review_dir) {
        Ok(doc) => doc,
        Err(error) => {
            issues.push(format!(
                "review `{}` state.toml cannot be parsed: {error}",
                review_dir_label(review_dir)
            ));
            return Ok(issues);
        }
    };
    let target_type = review_field(&doc, "target_type");
    let target = review_field(&doc, "target");
    match target_type.as_str() {
        "project" => {
            if !state_toml::state_path(project_dir).exists() {
                issues.push("review target project state is missing".to_owned());
            }
        }
        "topic" => {
            let project_doc = state_toml::load_state_doc(&state_toml::state_path(project_dir))?;
            if state_toml::topic_path(&project_doc, &target).is_none() {
                issues.push(format!("review target topic not found: {target}"));
            }
        }
        _ => issues.push(format!("invalid review target_type: {target_type}")),
    }
    let sessions = session_rows(&doc);
    for (id, _, path) in &sessions {
        if !review_dir.join(path).exists() {
            issues.push(format!("review session `{id}` missing file: {path}"));
        }
    }
    let lifecycle = lifecycle_override
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| review_field(&doc, "lifecycle"));
    if lifecycle == "completed" {
        let completed: Vec<_> = sessions
            .iter()
            .filter(|(_, status, _)| status == "completed")
            .collect();
        if completed.is_empty() {
            issues.push("completed review must have at least one completed session".to_owned());
        }
        for (id, _, path) in completed {
            let content =
                fs::read_to_string(review_dir.join(path)).map_err(|source| DaedalusError::Io {
                    path: review_dir.join(path),
                    source,
                })?;
            if !section_has_body(&content, "## User Answers") {
                issues.push(format!(
                    "completed review session `{id}` lacks user answers"
                ));
            }
            if !section_has_body(&content, "## Calibration") {
                issues.push(format!("completed review session `{id}` lacks calibration"));
            }
        }
    }
    Ok(issues)
}

fn collect_review_snapshots(project_dir: &Path) -> Result<Vec<ReviewSnapshot>> {
    let mut reviews = Vec::new();
    for review_dir in review_dirs(project_dir)? {
        if !review_dir.join("state.toml").exists() {
            continue;
        }
        let doc = load_review_doc(&review_dir)?;
        let path = review_dir
            .strip_prefix(project_dir)
            .unwrap_or(&review_dir)
            .to_string_lossy()
            .to_string();
        reviews.push(ReviewSnapshot {
            id: review_field(&doc, "id"),
            target_type: review_field(&doc, "target_type"),
            target: review_field(&doc, "target"),
            mode: review_field(&doc, "mode"),
            lifecycle: review_field(&doc, "lifecycle"),
            path,
            next_action: review_field(&doc, "next_action"),
        });
    }
    reviews.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(reviews)
}

fn review_dirs(project_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut dirs = child_review_dirs(&project_dir.join(".daedalus").join("reviews"))?;
    let project_doc = state_toml::load_state_doc(&state_toml::state_path(project_dir))?;
    for topic in state_toml::topics(&project_doc) {
        let root = project_dir
            .join(topic.path)
            .join(".daedalus")
            .join("reviews");
        dirs.extend(child_review_dirs(&root)?);
    }
    Ok(dirs)
}

fn child_review_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut dirs = Vec::new();
    let entries = fs::read_dir(root).map_err(|source| DaedalusError::Io {
        path: root.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| DaedalusError::Io {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }
    Ok(dirs)
}

fn find_review_dir(project_dir: &Path, review_id: &str) -> Result<PathBuf> {
    review_dirs(project_dir)?
        .into_iter()
        .find(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value == review_id)
        })
        .ok_or_else(|| DaedalusError::ReviewNotFound(review_id.to_owned()))
}

fn review_target_dir(
    project_dir: &Path,
    project_doc: &DocumentMut,
    target: &ReviewTarget,
) -> Result<PathBuf> {
    match target {
        ReviewTarget::Project => Ok(project_dir.to_path_buf()),
        ReviewTarget::Topic(slug) => {
            workspace_fs::topic_dir_by_slug(project_dir, slug).or_else(|_| {
                state_toml::topic_path(project_doc, slug)
                    .map(|path| project_dir.join(path))
                    .ok_or_else(|| DaedalusError::TopicNotFound(slug.to_owned()))
            })
        }
    }
}

fn target_parts<'a>(target: &'a ReviewTarget, fallback: &'a str) -> (&'static str, String) {
    match target {
        ReviewTarget::Project => ("project", fallback.to_owned()),
        ReviewTarget::Topic(slug) => ("topic", slug.to_owned()),
    }
}

fn review_target_label(project_doc: &DocumentMut, target: &ReviewTarget) -> String {
    match target {
        ReviewTarget::Project => state_toml::task_name(project_doc),
        ReviewTarget::Topic(slug) => slug.clone(),
    }
}

fn unique_review_dir(root: &Path, base_id: &str) -> PathBuf {
    let candidate = root.join(base_id);
    if !candidate.exists() {
        return candidate;
    }
    for idx in 2.. {
        let candidate = root.join(format!("{base_id}-{idx}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    unreachable!("infinite iterator should always return a candidate")
}

fn ensure_reviews_root(root: &Path, target_label: &str) -> Result<()> {
    fs::create_dir_all(root).map_err(|source| DaedalusError::Io {
        path: root.to_path_buf(),
        source,
    })?;
    let readme = root.join("README.md");
    if !readme.exists() {
        fs::write(
            &readme,
            format!(
                "# Reviews\n\nReviews for `{target_label}`. Review lifecycle is independent from learning lifecycle.\n"
            ),
        )
        .map_err(|source| DaedalusError::Io {
            path: readme,
            source,
        })?;
    }
    Ok(())
}

fn load_review_doc(review_dir: &Path) -> Result<DocumentMut> {
    let path = review_dir.join("state.toml");
    let content = fs::read_to_string(&path).map_err(|source| DaedalusError::Io {
        path: path.clone(),
        source,
    })?;
    content
        .parse::<DocumentMut>()
        .map_err(|source| DaedalusError::Toml { path, source })
}

fn save_review_doc(review_dir: &Path, doc: &DocumentMut) -> Result<()> {
    let path = review_dir.join("state.toml");
    fs::write(&path, doc.to_string()).map_err(|source| DaedalusError::Io { path, source })
}

fn review_field(doc: &DocumentMut, field: &str) -> String {
    doc.get("review")
        .and_then(|review| review.get(field))
        .and_then(Item::as_str)
        .unwrap_or("")
        .to_owned()
}

fn set_review_lifecycle(doc: &mut DocumentMut, lifecycle: ReviewLifecycle) {
    doc["review"]["lifecycle"] = value(lifecycle.as_str());
}

fn set_review_next_action(doc: &mut DocumentMut, next_action: &str) {
    doc["review"]["next_action"] = value(next_action);
}

fn append_review_transition(doc: &mut DocumentMut, action: &str, actor: &str, reason: &str) {
    let mut table = Table::new();
    table["action"] = value(action);
    table["timestamp"] = value(clock::now_local_timestamp());
    table["actor"] = value(actor);
    table["reason"] = value(reason);
    if !doc.as_table().contains_key("transitions") {
        doc["transitions"] = Item::ArrayOfTables(Default::default());
    }
    if let Some(array) = doc["transitions"].as_array_of_tables_mut() {
        array.push(table);
    }
}

fn append_session(doc: &mut DocumentMut, session_id: &str, session_path: &Path, status: &str) {
    let mut table = Table::new();
    table["id"] = value(session_id);
    table["status"] = value(status);
    table["path"] = value(
        session_path
            .file_name()
            .and_then(|value| value.to_str())
            .map(|name| format!("sessions/{name}"))
            .unwrap_or_else(|| "sessions/unknown.md".to_owned()),
    );
    table["started_at"] = value(clock::now_local_timestamp());
    if !doc.as_table().contains_key("sessions") {
        doc["sessions"] = Item::ArrayOfTables(Default::default());
    }
    if let Some(array) = doc["sessions"].as_array_of_tables_mut() {
        array.push(table);
    }
}

fn update_session_status(doc: &mut DocumentMut, session_id: &str, status: &str) -> Result<String> {
    let Some(array) = doc
        .get_mut("sessions")
        .and_then(Item::as_array_of_tables_mut)
    else {
        return Err(DaedalusError::InvalidReviewOperation(
            "review has no sessions".to_owned(),
        ));
    };
    for session in array.iter_mut() {
        if session["id"].as_str() == Some(session_id) {
            session["status"] = value(status);
            if status == "completed" {
                session["completed_at"] = value(clock::now_local_timestamp());
            }
            return session["path"]
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| {
                    DaedalusError::InvalidReviewOperation(format!(
                        "review session `{session_id}` missing path"
                    ))
                });
        }
    }
    Err(DaedalusError::InvalidReviewOperation(format!(
        "review session not found: {session_id}"
    )))
}

fn active_session_id(doc: &DocumentMut) -> Option<String> {
    session_rows(doc)
        .into_iter()
        .rev()
        .find(|(_, status, _)| status == "active")
        .map(|(id, _, _)| id)
}

fn next_session_number(doc: &DocumentMut) -> usize {
    session_rows(doc).len() + 1
}

fn session_rows(doc: &DocumentMut) -> Vec<(String, String, String)> {
    doc.get("sessions")
        .and_then(Item::as_array_of_tables)
        .map(|array| {
            array
                .iter()
                .map(|session| {
                    (
                        session["id"].as_str().unwrap_or_default().to_owned(),
                        session["status"].as_str().unwrap_or_default().to_owned(),
                        session["path"].as_str().unwrap_or_default().to_owned(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn section_has_body(content: &str, heading: &str) -> bool {
    let mut in_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == heading {
            in_section = true;
            continue;
        }
        if in_section && trimmed.starts_with("## ") {
            return false;
        }
        if in_section && !trimmed.is_empty() && !trimmed.starts_with('-') {
            return true;
        }
    }
    false
}

fn review_dir_label(review_dir: &Path) -> String {
    review_dir
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("unknown")
        .to_owned()
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
        "review".to_owned()
    } else {
        sanitized.to_owned()
    }
}
