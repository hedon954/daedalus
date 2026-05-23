use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::{DaedalusError, KnowledgeSnapshot, Result};
use crate::infrastructure::{clock, state_toml, template_fs, workspace_fs};

/// topic knowledge extraction 参数。
#[derive(Debug, Clone)]
pub struct ExtractKnowledgeOptions {
    pub repo_root: PathBuf,
    pub project_dir: PathBuf,
    pub topic_slug: String,
}

/// knowledge promote 参数。
#[derive(Debug, Clone)]
pub struct PromoteKnowledgeOptions {
    pub repo_root: PathBuf,
    pub project_dir: PathBuf,
    pub topic_slug: String,
}

/// knowledge export 参数。
#[derive(Debug, Clone)]
pub struct ExportKnowledgeOptions {
    pub repo_root: PathBuf,
    pub project_dir: PathBuf,
}

/// knowledge 操作输出。
#[derive(Debug, Clone)]
pub struct KnowledgeOutput {
    pub project_dir: PathBuf,
    pub action: String,
    pub path: PathBuf,
    pub next: String,
}

/// knowledge list 输出。
#[derive(Debug, Clone)]
pub struct KnowledgeListOutput {
    pub project_dir: PathBuf,
    pub items: Vec<KnowledgeSnapshot>,
}

/// knowledge validate 输出。
#[derive(Debug, Clone)]
pub struct KnowledgeValidationOutput {
    pub project_dir: PathBuf,
    pub issues: Vec<String>,
}

impl KnowledgeValidationOutput {
    pub fn is_ok(&self) -> bool {
        self.issues.is_empty()
    }
}

/// 为 topic 创建知识体系萃取候选目录。
pub fn extract_topic_knowledge(options: ExtractKnowledgeOptions) -> Result<KnowledgeOutput> {
    let topic_dir = workspace_fs::topic_dir_by_slug(&options.project_dir, &options.topic_slug)?;
    let target_dir = topic_dir.join("notes").join("knowledge-system");
    copy_knowledge_template_missing(&options.repo_root, &target_dir)?;
    Ok(KnowledgeOutput {
        project_dir: options.project_dir,
        action: "knowledge-extract".to_owned(),
        path: target_dir,
        next: "让用户校准 extraction.md 中的候选知识，再考虑 promote 到 shared。".to_owned(),
    })
}

/// 将 topic knowledge candidates 晋升到 shared knowledge-system。
pub fn promote_topic_knowledge(options: PromoteKnowledgeOptions) -> Result<KnowledgeOutput> {
    let topic_dir = workspace_fs::topic_dir_by_slug(&options.project_dir, &options.topic_slug)?;
    let extraction = topic_dir
        .join("notes")
        .join("knowledge-system")
        .join("extraction.md");
    if !extraction.exists() {
        return Err(DaedalusError::InvalidKnowledgeOperation(format!(
            "topic `{}` has no knowledge extraction candidates",
            options.topic_slug
        )));
    }
    let shared_dir = options.project_dir.join("shared").join("knowledge-system");
    copy_knowledge_template_missing(&options.repo_root, &shared_dir)?;
    append_promotion_log(
        &shared_dir.join("promotion-log.md"),
        &options.topic_slug,
        &format!(
            "topics/{}/notes/knowledge-system/extraction.md",
            options.topic_slug
        ),
        "shared/knowledge-system/",
        "shared verified candidate",
    )?;
    Ok(KnowledgeOutput {
        project_dir: options.project_dir,
        action: "knowledge-promote".to_owned(),
        path: shared_dir,
        next: "校准 shared knowledge-system 中的 concept/invariant/pattern/relation maps。"
            .to_owned(),
    })
}

/// 导出 project knowledge-base candidate。
pub fn export_project_knowledge(options: ExportKnowledgeOptions) -> Result<KnowledgeOutput> {
    let project_doc = state_toml::load_state_doc(&state_toml::state_path(&options.project_dir))?;
    let project_name = state_toml::task_name(&project_doc);
    let candidates_dir = options
        .repo_root
        .join("knowledge-base")
        .join("00-candidates");
    fs::create_dir_all(&candidates_dir).map_err(|source| DaedalusError::Io {
        path: candidates_dir.clone(),
        source,
    })?;
    let readme = candidates_dir.join("README.md");
    if !readme.exists() {
        fs::write(
            &readme,
            "# Knowledge Base Candidates\n\n这里保存等待用户确认的知识库候选条目。\n",
        )
        .map_err(|source| DaedalusError::Io {
            path: readme,
            source,
        })?;
    }
    let candidate_path = unique_candidate_path(&candidates_dir, &project_name);
    fs::write(
        &candidate_path,
        knowledge_base_candidate_content(&project_name, &options.project_dir),
    )
    .map_err(|source| DaedalusError::Io {
        path: candidate_path.clone(),
        source,
    })?;
    Ok(KnowledgeOutput {
        project_dir: options.project_dir,
        action: "knowledge-export".to_owned(),
        path: candidate_path,
        next: "用户确认候选条目后，再移动到合适的 knowledge-base 分类。".to_owned(),
    })
}

/// 列出 project 下已有 knowledge-system 产物。
pub fn list_knowledge(project_dir: PathBuf, repo_root: PathBuf) -> Result<KnowledgeListOutput> {
    Ok(KnowledgeListOutput {
        items: collect_knowledge_snapshots(&project_dir, &repo_root)?,
        project_dir,
    })
}

/// 校验 project knowledge-system 产物。
pub fn validate_knowledge(
    project_dir: PathBuf,
    repo_root: PathBuf,
) -> Result<KnowledgeValidationOutput> {
    Ok(KnowledgeValidationOutput {
        issues: validate_knowledge_paths(&project_dir, &repo_root)?,
        project_dir,
    })
}

/// 供 workspace validate 调用的 knowledge 校验。
pub fn validate_project_knowledge(project_dir: &Path, repo_root: &Path) -> Result<Vec<String>> {
    validate_knowledge_paths(project_dir, repo_root)
}

fn copy_knowledge_template_missing(repo_root: &Path, target_dir: &Path) -> Result<()> {
    let template_dir = repo_root
        .join("system")
        .join("templates")
        .join("knowledge-system");
    template_fs::copy_template_dir_missing(&template_dir, target_dir, &[])
}

fn collect_knowledge_snapshots(
    project_dir: &Path,
    repo_root: &Path,
) -> Result<Vec<KnowledgeSnapshot>> {
    let mut items = Vec::new();
    let project_doc = state_toml::load_state_doc(&state_toml::state_path(project_dir))?;
    for topic in state_toml::topics(&project_doc) {
        let path = project_dir
            .join(&topic.path)
            .join("notes")
            .join("knowledge-system")
            .join("extraction.md");
        if path.exists() {
            items.push(KnowledgeSnapshot {
                level: "topic".to_owned(),
                name: topic.slug,
                path: path
                    .strip_prefix(project_dir)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string(),
                status: "candidate".to_owned(),
            });
        }
    }
    let shared = project_dir.join("shared").join("knowledge-system");
    if shared.exists() {
        items.push(KnowledgeSnapshot {
            level: "shared".to_owned(),
            name: "shared knowledge-system".to_owned(),
            path: shared
                .strip_prefix(project_dir)
                .unwrap_or(&shared)
                .to_string_lossy()
                .to_string(),
            status: "verified candidates".to_owned(),
        });
    }
    let candidates_dir = repo_root.join("knowledge-base").join("00-candidates");
    if candidates_dir.exists() {
        for entry in fs::read_dir(&candidates_dir).map_err(|source| DaedalusError::Io {
            path: candidates_dir.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| DaedalusError::Io {
                path: candidates_dir.clone(),
                source,
            })?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) == Some("md")
                && path.file_name().and_then(|value| value.to_str()) != Some("README.md")
            {
                items.push(KnowledgeSnapshot {
                    level: "knowledge-base".to_owned(),
                    name: path
                        .file_stem()
                        .and_then(|value| value.to_str())
                        .unwrap_or("candidate")
                        .to_owned(),
                    path: path
                        .strip_prefix(repo_root)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string(),
                    status: "candidate".to_owned(),
                });
            }
        }
    }
    Ok(items)
}

fn validate_knowledge_paths(project_dir: &Path, repo_root: &Path) -> Result<Vec<String>> {
    let mut issues = Vec::new();
    let project_doc = state_toml::load_state_doc(&state_toml::state_path(project_dir))?;
    for topic in state_toml::topics(&project_doc) {
        let root = project_dir
            .join(&topic.path)
            .join("notes")
            .join("knowledge-system");
        if root.exists() {
            for file in [
                "extraction.md",
                "concept-map.md",
                "invariant-map.md",
                "failure-mode-map.md",
                "pattern-catalog.md",
                "relation-map.md",
                "promotion-log.md",
            ] {
                if !root.join(file).exists() {
                    issues.push(format!(
                        "topic `{}` knowledge-system missing file: {file}",
                        topic.slug
                    ));
                }
            }
        }
    }
    let shared = project_dir.join("shared").join("knowledge-system");
    if shared.exists() {
        for file in [
            "README.md",
            "concept-map.md",
            "invariant-map.md",
            "failure-mode-map.md",
            "pattern-catalog.md",
            "relation-map.md",
            "promotion-log.md",
        ] {
            if !shared.join(file).exists() {
                issues.push(format!("shared knowledge-system missing file: {file}"));
            }
        }
    }
    let candidates = repo_root.join("knowledge-base").join("00-candidates");
    if candidates.exists() {
        for entry in fs::read_dir(&candidates).map_err(|source| DaedalusError::Io {
            path: candidates.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| DaedalusError::Io {
                path: candidates.clone(),
                source,
            })?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("md")
                || path.file_name().and_then(|value| value.to_str()) == Some("README.md")
            {
                continue;
            }
            let content = fs::read_to_string(&path).map_err(|source| DaedalusError::Io {
                path: path.clone(),
                source,
            })?;
            for heading in [
                "## 业务目标 / 现实任务",
                "## 现实制约",
                "## Trade-off",
                "## 可迁移模式",
                "## 来源",
            ] {
                if !content.contains(heading) {
                    issues.push(format!(
                        "knowledge-base candidate `{}` missing heading: {heading}",
                        path.display()
                    ));
                }
            }
        }
    }
    Ok(issues)
}

fn append_promotion_log(path: &Path, item: &str, from: &str, to: &str, status: &str) -> Result<()> {
    let mut content = fs::read_to_string(path).unwrap_or_else(|_| {
        "# Promotion Log\n\n| Item | From | To | Status | Evidence | Decision |\n| --- | --- | --- | --- | --- | --- |\n".to_owned()
    });
    let line = format!(
        "| `{item}` | `{from}` | `{to}` | {status} | `{from}` | pending user calibration ({}) |\n",
        clock::now_local_timestamp()
    );
    if !content.contains(&line) {
        content.push_str(&line);
    }
    fs::write(path, content).map_err(|source| DaedalusError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn unique_candidate_path(candidates_dir: &Path, project_name: &str) -> PathBuf {
    let base = sanitize_slug(project_name);
    let candidate = candidates_dir.join(format!("{base}-knowledge-candidate.md"));
    if !candidate.exists() {
        return candidate;
    }
    for idx in 2.. {
        let candidate = candidates_dir.join(format!("{base}-knowledge-candidate-{idx}.md"));
        if !candidate.exists() {
            return candidate;
        }
    }
    unreachable!("infinite iterator should always return a candidate")
}

fn knowledge_base_candidate_content(project_name: &str, project_dir: &Path) -> String {
    format!(
        r#"# {project_name} 知识库候选

## 归档决策

- 建议位置：
- 是否需要调整知识库结构：
- 理由：
- Project source：`{}`
- Created at：{}

## 业务目标 / 现实任务

## 现实制约

## Naive Solution 失败点

## 核心抽象 / 不变量

## 实现机制

## Trade-off

## 对比最佳实践

## 可迁移模式

## 适用边界

## 复习题 / 应用题

## 来源

"#,
        project_dir.display(),
        clock::now_local_timestamp()
    )
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
        "knowledge".to_owned()
    } else {
        sanitized.to_owned()
    }
}
