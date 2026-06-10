use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::{DaedalusError, Result};
use crate::infrastructure::{state_toml, workspace_fs};

const WORKSPACE_BUCKETS: [&str; 1] = ["projects"];

/// TUI 当前页面。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiView {
    /// workspace 任务选择页。
    TaskSelector,
    /// 单个学习任务详情页。
    TaskOverview,
    /// 可滚动的只读详情页。
    Detail,
}

/// Overview 页当前聚焦的卡片。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverviewFocus {
    /// 下一步行动卡。
    NextAction,
    /// 证据和漂移风险。
    Evidence,
    /// 可打开阅读的文档入口。
    ReadingMap,
    /// 缺失产物。
    MissingArtifacts,
    /// Review 和 knowledge 状态。
    ReviewKnowledge,
    /// 状态流转记录。
    RecentTransitions,
}

impl OverviewFocus {
    fn all() -> &'static [OverviewFocus] {
        &[
            OverviewFocus::NextAction,
            OverviewFocus::Evidence,
            OverviewFocus::ReadingMap,
            OverviewFocus::MissingArtifacts,
            OverviewFocus::ReviewKnowledge,
            OverviewFocus::RecentTransitions,
        ]
    }
}

/// TUI 详情页的数据来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailSource {
    /// 下一步行动摘要。
    NextAction,
    /// 证据和漂移风险摘要。
    Evidence,
    /// 当前推荐 guide。
    Guide,
    /// `.daedalus/todo.md`。
    Todo,
    /// `.daedalus/outcome-map.md`。
    OutcomeMap,
    /// 缺失产物列表。
    MissingArtifacts,
    /// Review 与 knowledge 状态。
    ReviewKnowledge,
    /// 状态流转记录。
    RecentTransitions,
}

/// TUI 只读详情页内容。
#[derive(Debug, Clone)]
pub struct TuiDetail {
    /// 详情页标题。
    pub title: String,
    /// 文件路径或虚拟来源说明。
    pub source_label: String,
    /// 完整可滚动内容行。
    pub lines: Vec<String>,
}

impl TuiDetail {
    fn new(title: &str, source_label: &str, lines: Vec<String>) -> Self {
        Self {
            title: title.to_owned(),
            source_label: source_label.to_owned(),
            lines,
        }
    }

    /// 返回用于 markdown 渲染的完整内容。
    pub fn markdown(&self) -> String {
        self.lines.join("\n")
    }
}

/// TUI 只读应用状态。
#[derive(Debug, Clone)]
pub struct TuiApp {
    /// 当前页面。
    pub view: TuiView,
    /// 选择页任务列表。
    pub tasks: Vec<TuiTaskSummary>,
    /// 当前选择的任务索引。
    pub selected_task: usize,
    /// 当前展示的任务详情。
    pub overview: Option<TuiOverview>,
    /// 当前 overview 页聚焦的卡片。
    pub overview_focus: OverviewFocus,
    /// 当前打开的详情页。
    pub detail: Option<TuiDetail>,
    /// 详情页滚动位置。
    pub detail_scroll: usize,
    /// 是否允许从详情返回选择页。
    pub can_return_to_selector: bool,
    /// 从 workspace selector 启动时的 repo root，用于刷新任务列表。
    pub repo_root: Option<PathBuf>,
}

impl TuiApp {
    /// 根据当前目录和可选显式任务目录创建 TUI 状态。
    pub fn from_launch_context(
        repo_root: &Path,
        cwd: &Path,
        explicit: Option<PathBuf>,
    ) -> Result<Self> {
        if let Some(task_dir) = explicit {
            let overview = load_overview(&task_dir)?;
            return Ok(Self::single_task(overview));
        }

        if let Some(task_dir) = enclosing_task_dir(cwd, repo_root) {
            let overview = load_overview(&task_dir)?;
            return Ok(Self::single_task(overview));
        }

        let tasks = scan_task_summaries(repo_root)?;
        Ok(Self {
            view: TuiView::TaskSelector,
            tasks,
            selected_task: 0,
            overview: None,
            overview_focus: OverviewFocus::NextAction,
            detail: None,
            detail_scroll: 0,
            can_return_to_selector: true,
            repo_root: Some(repo_root.to_path_buf()),
        })
    }

    fn single_task(overview: TuiOverview) -> Self {
        Self {
            view: TuiView::TaskOverview,
            tasks: Vec::new(),
            selected_task: 0,
            overview: Some(overview),
            overview_focus: OverviewFocus::NextAction,
            detail: None,
            detail_scroll: 0,
            can_return_to_selector: false,
            repo_root: None,
        }
    }

    /// 将选择游标向上移动一项。
    pub fn select_previous(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        self.selected_task = self.selected_task.saturating_sub(1);
    }

    /// 将选择游标向下移动一项。
    pub fn select_next(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        self.selected_task = (self.selected_task + 1).min(self.tasks.len() - 1);
    }

    /// 进入当前选择的任务详情页。
    pub fn open_selected_task(&mut self) -> Result<()> {
        if let Some(task) = self.tasks.get(self.selected_task) {
            self.overview = Some(load_overview(&task.path)?);
            self.view = TuiView::TaskOverview;
            self.overview_focus = OverviewFocus::NextAction;
            self.detail = None;
            self.detail_scroll = 0;
        }
        Ok(())
    }

    /// 从任务详情页返回选择页。
    pub fn return_to_selector(&mut self) {
        if self.can_return_to_selector {
            self.view = TuiView::TaskSelector;
            self.overview = None;
            self.detail = None;
            self.detail_scroll = 0;
        }
    }

    /// 在 overview 页面向上移动焦点。
    pub fn focus_previous(&mut self) {
        let all = OverviewFocus::all();
        let current = all
            .iter()
            .position(|focus| *focus == self.overview_focus)
            .unwrap_or(0);
        self.overview_focus = all[current.saturating_sub(1)];
    }

    /// 在 overview 页面向下移动焦点。
    pub fn focus_next(&mut self) {
        let all = OverviewFocus::all();
        let current = all
            .iter()
            .position(|focus| *focus == self.overview_focus)
            .unwrap_or(0);
        self.overview_focus = all[(current + 1).min(all.len() - 1)];
    }

    /// 打开当前聚焦卡片对应的详情页。
    pub fn open_focused_detail(&mut self) -> Result<()> {
        let source = match self.overview_focus {
            OverviewFocus::NextAction => DetailSource::NextAction,
            OverviewFocus::Evidence => DetailSource::Evidence,
            OverviewFocus::ReadingMap => DetailSource::Guide,
            OverviewFocus::MissingArtifacts => DetailSource::MissingArtifacts,
            OverviewFocus::ReviewKnowledge => DetailSource::ReviewKnowledge,
            OverviewFocus::RecentTransitions => DetailSource::RecentTransitions,
        };
        self.open_detail(source)
    }

    /// 打开指定只读详情。
    pub fn open_detail(&mut self, source: DetailSource) -> Result<()> {
        let Some(overview) = &self.overview else {
            return Ok(());
        };
        self.detail = Some(overview.detail(source)?);
        self.detail_scroll = 0;
        self.view = TuiView::Detail;
        Ok(())
    }

    /// 从详情页返回 overview。
    pub fn close_detail(&mut self) {
        self.view = TuiView::TaskOverview;
        self.detail = None;
        self.detail_scroll = 0;
    }

    /// 重新读取当前展示对象。
    pub fn refresh(&mut self) -> Result<()> {
        match self.view {
            TuiView::TaskSelector => {
                if let Some(repo_root) = &self.repo_root {
                    self.tasks = scan_task_summaries(repo_root)?;
                    if !self.tasks.is_empty() {
                        self.selected_task = self.selected_task.min(self.tasks.len() - 1);
                    }
                }
            }
            TuiView::TaskOverview | TuiView::Detail => {
                if let Some(overview) = &self.overview {
                    self.overview = Some(load_overview(&overview.task_dir)?);
                }
                if self.view == TuiView::Detail {
                    self.close_detail();
                }
            }
        }
        Ok(())
    }

    /// 向上滚动详情页。
    pub fn scroll_detail_up(&mut self, amount: usize) {
        self.detail_scroll = self.detail_scroll.saturating_sub(amount);
    }

    /// 向下滚动详情页。
    pub fn scroll_detail_down(
        &mut self,
        amount: usize,
        viewport_height: usize,
        viewport_width: usize,
    ) {
        let max_scroll = self.max_detail_scroll(viewport_height, viewport_width);
        self.detail_scroll = (self.detail_scroll + amount).min(max_scroll);
    }

    /// 滚动到详情页顶部。
    pub fn scroll_detail_top(&mut self) {
        self.detail_scroll = 0;
    }

    /// 滚动到详情页底部。
    pub fn scroll_detail_bottom(&mut self, viewport_height: usize, viewport_width: usize) {
        self.detail_scroll = self.max_detail_scroll(viewport_height, viewport_width);
    }

    fn max_detail_scroll(&self, viewport_height: usize, viewport_width: usize) -> usize {
        let line_count = self
            .detail
            .as_ref()
            .map(|detail| detail.estimated_markdown_line_count(viewport_width))
            .unwrap_or(0);
        line_count.saturating_sub(viewport_height.max(1))
    }
}

impl TuiDetail {
    fn estimated_markdown_line_count(&self, viewport_width: usize) -> usize {
        let width = viewport_width.max(20);
        self.lines
            .iter()
            .map(|line| {
                let visual_width = line.chars().count().max(1);
                visual_width.div_ceil(width).max(1)
            })
            .sum::<usize>()
            .max(1)
    }
}

/// 选择页展示的学习任务摘要。
#[derive(Debug, Clone)]
pub struct TuiTaskSummary {
    /// 学习项目名称。
    pub task_name: String,
    /// 学习项目目录。
    pub path: PathBuf,
    /// 实际所在 workspace bucket。
    pub bucket: String,
    /// 生命周期。
    pub lifecycle: String,
    /// Active topic。
    pub active_topic: String,
    /// 当前 topic 阶段。
    pub current_phase: String,
    /// 当前阶段状态。
    pub current_status: String,
    /// 已完成阶段数量。
    pub done_stage_count: usize,
    /// 阶段总数。
    pub total_stage_count: usize,
}

/// TUI 只读总览所需的数据。
#[derive(Debug, Clone)]
pub struct TuiOverview {
    /// 学习项目名称。
    pub task_name: String,
    /// 学习项目目录。
    pub task_dir: PathBuf,
    /// Active topic slug。
    pub active_topic: String,
    /// Active topic title。
    pub active_topic_title: String,
    /// Active topic 目录。
    pub topic_dir: Option<PathBuf>,
    /// 任务生命周期状态。
    pub lifecycle: String,
    /// workspace bucket。
    pub workspace_bucket: String,
    /// 当前阶段 ID。
    pub current_phase: String,
    /// 当前阶段状态。
    pub current_status: String,
    /// 面向 Agent 的下一步动作。
    pub next_action: String,
    /// 当前可直接打开的 action guide。
    pub current_guide_path: Option<PathBuf>,
    /// topic/project 的 todo 文件。
    pub todo_path: PathBuf,
    /// topic/project 的 outcome map 文件。
    pub outcome_map_path: PathBuf,
    /// TUI 当前读取 artifact 的根目录。
    pub artifact_root: PathBuf,
    /// 已完成阶段数量。
    pub done_stage_count: usize,
    /// 阶段总数。
    pub total_stage_count: usize,
    /// 当前缺失的关键产物。
    pub missing_artifacts: Vec<String>,
    /// `todo.md` 中的待办摘要。
    pub todo_summary: Vec<String>,
    /// 最近几条状态流转摘要。
    pub recent_transitions: Vec<String>,
    /// Review 摘要。
    pub review_summary: Vec<String>,
    /// Knowledge 摘要。
    pub knowledge_summary: Vec<String>,
    /// 任务关闭信息摘要。
    pub closure_summary: Vec<String>,
}

impl TuiOverview {
    /// 构建详情页内容。
    pub fn detail(&self, source: DetailSource) -> Result<TuiDetail> {
        match source {
            DetailSource::NextAction => Ok(self.virtual_detail(
                "Next Action",
                "generated from current guide or todo",
                self.next_action.lines().map(ToOwned::to_owned).collect(),
            )),
            DetailSource::Evidence => Ok(self.virtual_detail(
                "Evidence / Drift",
                "generated from state, artifacts, review, and knowledge summaries",
                self.evidence_lines(),
            )),
            DetailSource::Guide => self.file_detail(
                "Current Guide",
                self.current_guide_path.as_deref(),
                "No current actionable guide was found. Use todo/outcome-map for the next step.",
            ),
            DetailSource::Todo => self.file_detail(
                "Todo Path Board",
                Some(&self.todo_path),
                "todo.md does not exist yet.",
            ),
            DetailSource::OutcomeMap => self.file_detail(
                "Outcome Map",
                Some(&self.outcome_map_path),
                "outcome-map.md does not exist yet.",
            ),
            DetailSource::MissingArtifacts => Ok(self.virtual_detail(
                "Missing Artifacts",
                "generated from required_artifacts in state.toml",
                non_empty_lines(
                    &self.missing_artifacts,
                    "All required artifacts are present.",
                ),
            )),
            DetailSource::ReviewKnowledge => {
                let mut lines = Vec::new();
                lines.push("Review Focus".to_owned());
                lines.extend(indented_or_empty(
                    &self.review_summary,
                    "No review plans yet.",
                ));
                lines.push(String::new());
                lines.push("Knowledge Focus".to_owned());
                lines.extend(indented_or_empty(
                    &self.knowledge_summary,
                    "No knowledge-system focus yet.",
                ));
                Ok(self.virtual_detail(
                    "Review / Knowledge Focus",
                    "generated from .daedalus/reviews and knowledge-system files",
                    lines,
                ))
            }
            DetailSource::RecentTransitions => Ok(self.virtual_detail(
                "Recent Transitions",
                "generated from state.toml transition history",
                non_empty_lines(&self.recent_transitions, "No transition history yet."),
            )),
        }
    }

    /// Overview 第一屏的证据和漂移摘要。
    pub fn evidence_lines(&self) -> Vec<String> {
        vec![
            format!(
                "Stage: {} / {} ({}/{})",
                self.current_phase,
                self.current_status,
                self.done_stage_count,
                self.total_stage_count
            ),
            format!("Missing artifacts: {}", self.missing_artifacts.len()),
            format!("Review focus items: {}", self.review_summary.len()),
            format!("Knowledge focus items: {}", self.knowledge_summary.len()),
            format!("Transitions recorded: {}", self.recent_transitions.len()),
            format!(
                "Current guide: {}",
                self.current_guide_path
                    .as_ref()
                    .map(|path| relative_display(&self.artifact_root, path))
                    .unwrap_or_else(|| "none".to_owned())
            ),
            "Verify: run `daedalus validate` and the topic-specific test command.".to_owned(),
        ]
    }

    /// Overview 第一屏的可读文档入口。
    pub fn reading_map_lines(&self) -> Vec<String> {
        vec![
            format!(
                "g  Guide: {}",
                self.current_guide_path
                    .as_ref()
                    .map(|path| relative_display(&self.artifact_root, path))
                    .unwrap_or_else(|| "not available".to_owned())
            ),
            format!(
                "t  Todo: {}",
                relative_display(&self.artifact_root, &self.todo_path)
            ),
            format!(
                "o  Outcome: {}",
                relative_display(&self.artifact_root, &self.outcome_map_path)
            ),
            "enter  Open focused card for full reading".to_owned(),
        ]
    }

    fn file_detail(
        &self,
        title: &str,
        path: Option<&Path>,
        missing_message: &str,
    ) -> Result<TuiDetail> {
        let Some(path) = path else {
            return Ok(self.virtual_detail(
                title,
                "not available",
                vec![missing_message.to_owned()],
            ));
        };
        if !path.exists() {
            let source_label = relative_display(&self.artifact_root, path);
            return Ok(self.virtual_detail(title, &source_label, vec![missing_message.to_owned()]));
        }
        let content = fs::read_to_string(path).map_err(|source| DaedalusError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let lines = if content.is_empty() {
            vec!["<empty file>".to_owned()]
        } else {
            content.lines().map(ToOwned::to_owned).collect()
        };
        Ok(TuiDetail::new(
            title,
            &relative_display(&self.artifact_root, path),
            lines,
        ))
    }

    fn virtual_detail(&self, title: &str, source_label: &str, lines: Vec<String>) -> TuiDetail {
        TuiDetail::new(title, source_label, lines)
    }
}

/// 从学习任务目录加载 TUI 只读总览数据。
pub fn load_overview(task_dir: &Path) -> Result<TuiOverview> {
    let doc = state_toml::load_state_doc(&state_toml::state_path(task_dir))?;
    let (topic_dir, topic_doc) = active_topic_doc(task_dir, &doc)?;
    let active_topic = topic_doc
        .as_ref()
        .map(state_toml::topic_slug)
        .or_else(|| state_toml::active_topic(&doc))
        .unwrap_or_else(|| "none".to_owned());
    let active_topic_title = topic_doc
        .as_ref()
        .map(state_toml::topic_title)
        .unwrap_or_else(|| "none".to_owned());
    let progress_doc = topic_doc.as_ref().unwrap_or(&doc);
    let current_phase =
        state_toml::current_phase(progress_doc).unwrap_or_else(|| "unknown".to_owned());
    let stages = state_toml::stages(progress_doc);
    let current_status = stages
        .iter()
        .find(|stage| stage.id == current_phase)
        .map(|stage| stage.status.clone())
        .unwrap_or_else(|| "unknown".to_owned());
    let done_stage_count = stages.iter().filter(|stage| stage.status == "done").count();
    let total_stage_count = stages.len();
    let lifecycle = task_field(&doc, "lifecycle").unwrap_or_else(|| "unknown".to_owned());
    let workspace_bucket = actual_bucket(task_dir);

    let mut missing_artifacts = Vec::new();
    let artifact_root = topic_dir.clone().unwrap_or_else(|| task_dir.to_path_buf());
    for stage in stages {
        for artifact in stage.required_artifacts {
            if !artifact_root.join(&artifact).exists() {
                missing_artifacts.push(format!("{artifact} ({})", stage.id));
            }
        }
    }

    let todo_summary = fs::read_to_string(artifact_root.join(".daedalus").join("todo.md"))
        .ok()
        .map(|content| {
            content
                .lines()
                .filter(|line| line.trim_start().starts_with("- [ ]"))
                .take(6)
                .map(|line| line.trim().trim_start_matches("- [ ]").trim().to_owned())
                .collect()
        })
        .unwrap_or_default();

    let recent_transitions = state_toml::transitions(progress_doc)
        .iter()
        .map(|transition| {
            format!(
                "{}  {}  {}",
                transition.timestamp, transition.action, transition.stage
            )
        })
        .collect();

    let mut closure_summary = Vec::new();
    if let Some(closed_at) = state_toml::closed_at(&doc) {
        closure_summary.push(format!("closed_at: {closed_at}"));
    }
    if let Some(close_reason) = state_toml::close_reason(&doc) {
        closure_summary.push(format!("reason: {close_reason}"));
    }

    let next_action = build_next_action(
        &artifact_root,
        &current_phase,
        &current_status,
        state_toml::next_action(progress_doc),
    );
    let current_guide_path = find_action_guide(&artifact_root.join("guides").join(&current_phase));

    Ok(TuiOverview {
        task_name: state_toml::task_name(&doc),
        task_dir: task_dir.to_path_buf(),
        active_topic,
        active_topic_title,
        topic_dir,
        lifecycle,
        workspace_bucket,
        current_phase,
        current_status,
        next_action,
        current_guide_path,
        todo_path: artifact_root.join(".daedalus").join("todo.md"),
        outcome_map_path: artifact_root.join(".daedalus").join("outcome-map.md"),
        artifact_root,
        done_stage_count,
        total_stage_count,
        missing_artifacts,
        todo_summary,
        recent_transitions,
        review_summary: review_summary(task_dir),
        knowledge_summary: knowledge_summary(task_dir),
        closure_summary,
    })
}

fn build_next_action(
    artifact_root: &Path,
    current_phase: &str,
    current_status: &str,
    fallback: String,
) -> String {
    let guide_dir = artifact_root.join("guides").join(current_phase);
    if let Some(guide) = find_action_guide(&guide_dir)
        && let Ok(content) = fs::read_to_string(&guide)
    {
        let mut lines = Vec::new();
        let relative_guide = relative_display(artifact_root, &guide);
        lines.push(format!("Current: {current_phase} / {current_status}"));
        if let Some(slice) = markdown_value(&content, "Current slice") {
            lines.push(format!("Slice: {slice}"));
        }
        if let Some(gap) = markdown_value(&content, "Current gap") {
            lines.push(format!("Gap: {gap}"));
        }
        let actions = action_card_items(&content);
        if !actions.is_empty() {
            lines.push(format!("Next: {}", actions.join(" -> ")));
        }
        if let Some(after) = markdown_value(&content, "After this") {
            lines.push(format!("Unlocks: {after}"));
        }
        lines.push(format!("Guide: {relative_guide}"));
        return lines.join("\n");
    }

    if let Some(todo_summary) = todo_now_summary(&artifact_root.join(".daedalus").join("todo.md")) {
        return todo_summary;
    }

    fallback
}

fn find_action_guide(guide_dir: &Path) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    let entries = fs::read_dir(guide_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if path.is_file()
            && name.ends_with(".md")
            && name != "README.md"
            && name.contains("slice")
            && is_actionable_guide(&path)
        {
            candidates.push(path);
        }
    }
    candidates.sort();
    candidates.pop()
}

fn is_actionable_guide(path: &Path) -> bool {
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    if markdown_value(&content, "Current slice")
        .map(|value| value.to_lowercase().contains("completed"))
        .unwrap_or(false)
    {
        return false;
    }
    content.contains("## Action Card")
}

fn markdown_value(content: &str, key: &str) -> Option<String> {
    let prefix = format!("- {key}:");
    content.lines().find_map(|line| {
        line.trim()
            .strip_prefix(&prefix)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    })
}

fn action_card_items(content: &str) -> Vec<String> {
    let mut in_action_card = false;
    let mut values = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "## Action Card" {
            in_action_card = true;
            continue;
        }
        if in_action_card && trimmed.starts_with("## ") {
            break;
        }
        if in_action_card
            && trimmed.starts_with("### ")
            && let Some((_, title)) = trimmed.trim_start_matches("### ").split_once(". ")
        {
            values.push(title.to_owned());
        }
        if values.len() >= 3 {
            break;
        }
    }
    values
}

fn todo_now_summary(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let mut in_now = false;
    let mut lines = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "## Now" {
            in_now = true;
            continue;
        }
        if in_now && trimmed.starts_with("## ") {
            break;
        }
        if in_now
            && trimmed.starts_with("- ")
            && (trimmed.contains("当前问题")
                || trimmed.contains("当前待解决")
                || trimmed.contains("完成后解锁"))
        {
            lines.push(trimmed.trim_start_matches("- ").to_owned());
        }
        if lines.len() >= 4 {
            break;
        }
    }
    (!lines.is_empty()).then(|| lines.join("\n"))
}

fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

fn non_empty_lines(values: &[String], empty_message: &str) -> Vec<String> {
    if values.is_empty() {
        vec![empty_message.to_owned()]
    } else {
        values.to_vec()
    }
}

fn indented_or_empty(values: &[String], empty_message: &str) -> Vec<String> {
    if values.is_empty() {
        vec![format!("  {empty_message}")]
    } else {
        values.iter().map(|value| format!("  - {value}")).collect()
    }
}

/// 查找当前目录是否处于某个学习任务目录或其子目录下。
pub fn enclosing_task_dir(cwd: &Path, repo_root: &Path) -> Option<PathBuf> {
    if let Some(topic_dir) = workspace_fs::enclosing_topic_dir(cwd) {
        return workspace_fs::project_dir_from_topic_dir(&topic_dir).ok();
    }
    workspace_fs::enclosing_project_dir(cwd).filter(|path| path.starts_with(repo_root))
}

/// 扫描当前已具备明确 lifecycle 语义的 workspace bucket。
pub fn scan_task_summaries(repo_root: &Path) -> Result<Vec<TuiTaskSummary>> {
    let mut tasks = Vec::new();
    for bucket in WORKSPACE_BUCKETS {
        let root = repo_root.join("workspaces").join(bucket);
        if !root.exists() {
            continue;
        }
        let entries = fs::read_dir(&root).map_err(|source| crate::domain::DaedalusError::Io {
            path: root.clone(),
            source,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| crate::domain::DaedalusError::Io {
                path: root.clone(),
                source,
            })?;
            let path = entry.path();
            if path.is_dir() && path.join(".daedalus").join("state.toml").exists() {
                tasks.push(summary_from_task_dir(&path, bucket)?);
            }
        }
    }
    tasks.sort_by(|left, right| {
        bucket_order(&left.bucket)
            .cmp(&bucket_order(&right.bucket))
            .then_with(|| left.task_name.cmp(&right.task_name))
    });
    Ok(tasks)
}

fn summary_from_task_dir(task_dir: &Path, bucket: &str) -> Result<TuiTaskSummary> {
    let doc = state_toml::load_state_doc(&state_toml::state_path(task_dir))?;
    let (_, topic_doc) = active_topic_doc(task_dir, &doc)?;
    let progress_doc = topic_doc.as_ref().unwrap_or(&doc);
    let active_topic = topic_doc
        .as_ref()
        .map(state_toml::topic_slug)
        .or_else(|| state_toml::active_topic(&doc))
        .unwrap_or_else(|| "none".to_owned());
    let current_phase =
        state_toml::current_phase(progress_doc).unwrap_or_else(|| "unknown".to_owned());
    let stages = state_toml::stages(progress_doc);
    let current_status = stages
        .iter()
        .find(|stage| stage.id == current_phase)
        .map(|stage| stage.status.clone())
        .unwrap_or_else(|| "unknown".to_owned());
    Ok(TuiTaskSummary {
        task_name: state_toml::task_name(&doc),
        path: task_dir.to_path_buf(),
        bucket: bucket.to_owned(),
        lifecycle: task_field(&doc, "lifecycle").unwrap_or_else(|| "unknown".to_owned()),
        active_topic,
        current_phase,
        current_status,
        done_stage_count: stages.iter().filter(|stage| stage.status == "done").count(),
        total_stage_count: stages.len(),
    })
}

fn active_topic_doc(
    project_dir: &Path,
    project_doc: &toml_edit::DocumentMut,
) -> Result<(Option<PathBuf>, Option<toml_edit::DocumentMut>)> {
    let Some(slug) = state_toml::active_topic(project_doc).filter(|slug| !slug.is_empty()) else {
        return Ok((None, None));
    };
    let Some(path) = state_toml::topic_path(project_doc, &slug) else {
        return Ok((None, None));
    };
    let topic_dir = project_dir.join(path);
    let topic_doc = state_toml::load_state_doc(&state_toml::state_path(&topic_dir))?;
    Ok((Some(topic_dir), Some(topic_doc)))
}

fn task_field(doc: &toml_edit::DocumentMut, field: &str) -> Option<String> {
    doc["task"][field].as_str().map(ToOwned::to_owned)
}

fn actual_bucket(task_dir: &Path) -> String {
    workspace_fs::bucket_from_task_dir(task_dir)
        .map(|bucket| bucket.as_str().to_owned())
        .unwrap_or_else(|_| {
            task_dir
                .parent()
                .and_then(Path::file_name)
                .and_then(|value| value.to_str())
                .unwrap_or("unknown")
                .to_owned()
        })
}

fn review_summary(task_dir: &Path) -> Vec<String> {
    let mut values = Vec::new();
    collect_review_labels(
        &task_dir.join(".daedalus").join("reviews"),
        "project",
        &mut values,
    );
    if let Ok(doc) = state_toml::load_state_doc(&state_toml::state_path(task_dir)) {
        for topic in state_toml::topics(&doc) {
            collect_review_labels(
                &task_dir.join(&topic.path).join(".daedalus").join("reviews"),
                &topic.slug,
                &mut values,
            );
        }
    }
    values
}

fn collect_review_labels(root: &Path, owner: &str, values: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("state.toml").exists() {
            let id = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("unknown");
            values.push(format!("{owner}: {id}"));
        }
    }
}

fn knowledge_summary(task_dir: &Path) -> Vec<String> {
    let mut values = Vec::new();
    let shared = task_dir.join("shared").join("knowledge-system");
    if shared.exists() {
        values.push("shared knowledge-system present".to_owned());
    }
    if let Ok(doc) = state_toml::load_state_doc(&state_toml::state_path(task_dir)) {
        for topic in state_toml::topics(&doc) {
            let extraction = task_dir
                .join(&topic.path)
                .join("notes")
                .join("knowledge-system")
                .join("extraction.md");
            if extraction.exists() {
                values.push(format!("{}: extraction candidates", topic.slug));
            }
        }
    }
    values
}

fn bucket_order(bucket: &str) -> usize {
    WORKSPACE_BUCKETS
        .iter()
        .position(|candidate| *candidate == bucket)
        .unwrap_or(WORKSPACE_BUCKETS.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn overview_for_test(root: &Path) -> TuiOverview {
        TuiOverview {
            task_name: "demo-task".to_owned(),
            task_dir: root.to_path_buf(),
            active_topic: "tools-permissions".to_owned(),
            active_topic_title: "Tools Permissions".to_owned(),
            topic_dir: None,
            lifecycle: "active".to_owned(),
            workspace_bucket: "projects".to_owned(),
            current_phase: "08-demo-coder".to_owned(),
            current_status: "active".to_owned(),
            next_action: "Goal: keep learning navigable\nVerify: cargo test".to_owned(),
            current_guide_path: None,
            todo_path: root.join(".daedalus").join("todo.md"),
            outcome_map_path: root.join(".daedalus").join("outcome-map.md"),
            artifact_root: root.to_path_buf(),
            done_stage_count: 7,
            total_stage_count: 10,
            missing_artifacts: vec!["demo/README.md (08-demo-coder)".to_owned()],
            todo_summary: vec!["Slice 8 Event Protocol Hardening".to_owned()],
            recent_transitions: vec![
                "2026-05-16 21:49:16  enter  08-demo-coder".to_owned(),
                "2026-06-02 22:00:00  checkpoint  08-demo-coder".to_owned(),
            ],
            review_summary: Vec::new(),
            knowledge_summary: vec!["shared knowledge-system present".to_owned()],
            closure_summary: Vec::new(),
        }
    }

    #[test]
    fn next_action_prefers_slice_action_guide() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let root = temp.path();
        let guide_dir = root.join("guides/08-demo-coder");
        fs::create_dir_all(&guide_dir).expect("guide dir");
        fs::write(
            guide_dir.join("slice-6-hardening.md"),
            r#"# Slice 6 Hardening Guide

## Learning Navigation

- Current slice: Slice 6 Agent Orchestrator
- Current gap: live path 已跑通，但 deterministic tests 和安全阀不足
- After this: 可以把 approval / sandbox / retry 接入 ReAct loop

## Action Card

### 1. Add `max_turns`

### 2. Emit `ToolCallFinished`

### 3. Add Fake LLM Tests

## Completion Criteria
"#,
        )
        .expect("guide");

        let next_action =
            build_next_action(root, "08-demo-coder", "active", "short fallback".to_owned());

        assert!(next_action.contains("Current: 08-demo-coder / active"));
        assert!(next_action.contains("Slice: Slice 6 Agent Orchestrator"));
        assert!(next_action.contains("Gap: live path 已跑通"));
        assert!(next_action.contains("Next: Add `max_turns` -> Emit `ToolCallFinished`"));
        assert!(next_action.contains("Guide: guides/08-demo-coder/slice-6-hardening.md"));
        assert!(!next_action.contains("short fallback"));
    }

    #[test]
    fn next_action_accepts_numbered_slice_guide_and_ignores_completed_guide() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let root = temp.path();
        let guide_dir = root.join("guides/08-demo-coder");
        fs::create_dir_all(&guide_dir).expect("guide dir");
        fs::write(
            guide_dir.join("07-slice-7-policy-composition.md"),
            r#"# Slice 7

## Learning Navigation

- Current slice: Slice 7 completed
- Current gap: old completed work

## Action Card

### 1. Old completed action
"#,
        )
        .expect("completed guide");
        fs::write(
            guide_dir.join("08-slice-8-event-protocol.md"),
            r#"# Slice 8

## Learning Navigation

- Current slice: Slice 8 Event Protocol Hardening
- Current gap: event protocol is not observable enough
- After this: README can explain the command safety trace

## Action Card

### 1. Define event protocol

### 2. Emit shell runtime events
"#,
        )
        .expect("active guide");

        let next_action =
            build_next_action(root, "08-demo-coder", "active", "short fallback".to_owned());

        assert!(next_action.contains("Slice: Slice 8 Event Protocol Hardening"));
        assert!(next_action.contains("Gap: event protocol is not observable enough"));
        assert!(next_action.contains("Next: Define event protocol -> Emit shell runtime events"));
        assert!(next_action.contains("Guide: guides/08-demo-coder/08-slice-8-event-protocol.md"));
        assert!(!next_action.contains("Old completed action"));
    }

    #[test]
    fn detail_source_reads_full_todo_without_truncation() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let root = temp.path();
        let todo_path = root.join(".daedalus").join("todo.md");
        fs::create_dir_all(todo_path.parent().expect("todo parent")).expect("todo dir");
        let content = (0..24)
            .map(|index| format!("line-{index}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&todo_path, content).expect("todo");
        let overview = overview_for_test(root);

        let detail = overview.detail(DetailSource::Todo).expect("detail");

        assert_eq!(detail.lines.len(), 24);
        assert_eq!(detail.lines.first().map(String::as_str), Some("line-0"));
        assert_eq!(detail.lines.last().map(String::as_str), Some("line-23"));
    }

    #[test]
    fn detail_estimates_wrapped_markdown_line_count_for_scrolling() {
        let detail = TuiDetail::new(
            "Markdown Detail",
            "test.md",
            vec![
                "# Heading".to_owned(),
                String::new(),
                "- **strong** item".to_owned(),
                "- a very long markdown item that should wrap when the viewport is narrow"
                    .to_owned(),
            ],
        );

        assert!(detail.estimated_markdown_line_count(20) > detail.lines.len());
    }

    #[test]
    fn detail_scroll_clamps_to_available_content() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let root = temp.path();
        let overview = overview_for_test(root);
        let mut app = TuiApp {
            view: TuiView::Detail,
            tasks: Vec::new(),
            selected_task: 0,
            overview: Some(overview),
            overview_focus: OverviewFocus::NextAction,
            detail: Some(TuiDetail::new(
                "Long Detail",
                "test",
                (0..10).map(|index| format!("line-{index}")).collect(),
            )),
            detail_scroll: 0,
            can_return_to_selector: false,
            repo_root: None,
        };

        app.scroll_detail_down(100, 4, 80);
        assert_eq!(app.detail_scroll, 6);

        app.scroll_detail_up(2);
        assert_eq!(app.detail_scroll, 4);

        app.scroll_detail_bottom(8, 80);
        assert_eq!(app.detail_scroll, 2);

        app.scroll_detail_top();
        assert_eq!(app.detail_scroll, 0);
    }

    #[test]
    fn overview_exposes_reading_map_and_evidence_for_first_screen() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let root = temp.path();
        let overview = overview_for_test(root);

        let reading_map = overview.reading_map_lines();
        let evidence = overview.evidence_lines();

        assert!(
            reading_map
                .iter()
                .any(|line| line.contains("Todo: .daedalus/todo.md"))
        );
        assert!(
            reading_map
                .iter()
                .any(|line| line.contains("Outcome: .daedalus/outcome-map.md"))
        );
        assert!(
            evidence
                .iter()
                .any(|line| line.contains("Missing artifacts: 1"))
        );
        assert!(
            evidence
                .iter()
                .any(|line| line.contains("Transitions recorded: 2"))
        );
    }
}
