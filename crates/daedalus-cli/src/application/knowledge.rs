use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::{DaedalusError, KnowledgeSnapshot, Result};
use crate::infrastructure::clock;
use toml_edit::{DocumentMut, Item, Table, value};

const REQUIRED_KNOWLEDGE_DIRS: [&str; 10] = [
    "concepts",
    "skills",
    "patterns",
    "problems",
    "cases",
    "source-maps",
    "trees",
    "drills",
    "index",
    "site",
];

const ENTRY_DIRS: [&str; 8] = [
    "concepts",
    "skills",
    "patterns",
    "problems",
    "cases",
    "source-maps",
    "trees",
    "drills",
];

const REQUIRED_ENTRY_HEADINGS: [&str; 10] = [
    "## 回忆钩子",
    "## 现实问题",
    "## 第一性原理",
    "## 底层原理",
    "## 关键不变量",
    "## 取舍",
    "## 不要照搬",
    "## 迁移方式",
    "## 证据来源",
    "## 复习练习",
];

/// knowledge 操作输出。
#[derive(Debug, Clone)]
pub struct KnowledgeOutput {
    pub action: String,
    pub path: PathBuf,
    pub next: String,
}

/// knowledge list 输出。
#[derive(Debug, Clone)]
pub struct KnowledgeListOutput {
    pub root: PathBuf,
    pub items: Vec<KnowledgeSnapshot>,
}

/// knowledge validate / link-check 输出。
#[derive(Debug, Clone)]
pub struct KnowledgeValidationOutput {
    pub root: PathBuf,
    pub action: String,
    pub issues: Vec<String>,
}

impl KnowledgeValidationOutput {
    pub fn is_ok(&self) -> bool {
        self.issues.is_empty()
    }
}

/// 确保 knowledge-base 基础目录存在。
pub fn ensure_knowledge_base_layout(repo_root: &Path) -> Result<PathBuf> {
    let root = knowledge_root(repo_root);
    ensure_dir(&root)?;
    for dir in REQUIRED_KNOWLEDGE_DIRS {
        ensure_dir(&root.join(dir))?;
    }
    ensure_index_readme(&root)?;
    Ok(root)
}

/// 生成某一类知识条目模板。
pub fn create_knowledge_template(
    repo_root: &Path,
    kind: &str,
    slug: &str,
    title: Option<&str>,
) -> Result<KnowledgeOutput> {
    let root = ensure_knowledge_base_layout(repo_root)?;
    let plural = plural_dir(kind)?;
    let slug = sanitize_slug(slug);
    let title = title
        .filter(|value| !value.trim().is_empty())
        .map(str::trim)
        .unwrap_or(&slug);
    let template_path = repo_root
        .join("system")
        .join("templates")
        .join("knowledge")
        .join(format!("{kind}.md"));
    let template = fs::read_to_string(&template_path).map_err(|source| DaedalusError::Io {
        path: template_path.clone(),
        source,
    })?;
    let content = template
        .replace("{{TITLE}}", title)
        .replace("{{SLUG}}", &slug)
        .replace("{{KIND}}", kind)
        .replace("{{CREATED_AT}}", &clock::now_local_timestamp());
    let path = root.join(plural).join(format!("{slug}.md"));
    if path.exists() {
        return Err(DaedalusError::InvalidKnowledgeOperation(format!(
            "knowledge entry already exists: {}",
            path.display()
        )));
    }
    fs::write(&path, content).map_err(|source| DaedalusError::Io {
        path: path.clone(),
        source,
    })?;
    Ok(KnowledgeOutput {
        action: "knowledge-template".to_owned(),
        path,
        next: "校准条目的问题入口、第一性原理、底层原理、迁移边界和复习练习，然后运行 daedalus knowledge index。"
            .to_owned(),
    })
}

/// 重建 knowledge-base/index.toml。
pub fn rebuild_knowledge_index(repo_root: &Path) -> Result<KnowledgeOutput> {
    let root = ensure_knowledge_base_layout(repo_root)?;
    let entries = collect_knowledge_snapshots_from_root(&root)?;
    let path = root.join("index.toml");
    let mut doc = DocumentMut::new();
    doc["schema_version"] = value(1);
    doc["generated_at"] = value(clock::now_local_timestamp());
    doc["entries"] = Item::ArrayOfTables(Default::default());
    if let Some(array) = doc["entries"].as_array_of_tables_mut() {
        for entry in entries {
            let mut table = Table::new();
            table["kind"] = value(entry.level);
            table["title"] = value(entry.name);
            table["path"] = value(entry.path);
            table["status"] = value(entry.status);
            array.push(table);
        }
    }
    fs::write(&path, doc.to_string()).map_err(|source| DaedalusError::Io {
        path: path.clone(),
        source,
    })?;
    Ok(KnowledgeOutput {
        action: "knowledge-index".to_owned(),
        path,
        next: "运行 daedalus knowledge validate 和 daedalus knowledge link-check。".to_owned(),
    })
}

/// 列出 knowledge-base 条目。
pub fn list_knowledge(repo_root: PathBuf) -> Result<KnowledgeListOutput> {
    let root = ensure_knowledge_base_layout(&repo_root)?;
    Ok(KnowledgeListOutput {
        items: collect_knowledge_snapshots_from_root(&root)?,
        root,
    })
}

/// 校验 knowledge-base 结构与条目最低质量门槛。
pub fn validate_knowledge(repo_root: PathBuf) -> Result<KnowledgeValidationOutput> {
    let root = knowledge_root(&repo_root);
    Ok(KnowledgeValidationOutput {
        issues: validate_knowledge_base(&root)?,
        root,
        action: "knowledge-validate".to_owned(),
    })
}

/// 检查 knowledge-base 内部本地链接。
pub fn link_check_knowledge(repo_root: PathBuf) -> Result<KnowledgeValidationOutput> {
    let root = knowledge_root(&repo_root);
    Ok(KnowledgeValidationOutput {
        issues: link_check_knowledge_base(&root)?,
        root,
        action: "knowledge-link-check".to_owned(),
    })
}

/// 供 workspace validate 调用的 knowledge 校验。
pub fn validate_project_knowledge(_project_dir: &Path, repo_root: &Path) -> Result<Vec<String>> {
    validate_knowledge_base(&knowledge_root(repo_root))
}

fn knowledge_root(repo_root: &Path) -> PathBuf {
    repo_root.join("knowledge-base")
}

fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| DaedalusError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn ensure_index_readme(root: &Path) -> Result<()> {
    let path = root.join("index").join("README.md");
    if !path.exists() {
        fs::write(
            &path,
            "# 知识库导航\n\n这里是人可读的知识库入口。机器索引见 `../index.toml`。\n",
        )
        .map_err(|source| DaedalusError::Io { path, source })?;
    }
    Ok(())
}

fn collect_knowledge_snapshots_from_root(root: &Path) -> Result<Vec<KnowledgeSnapshot>> {
    let mut items = Vec::new();
    for dir in ENTRY_DIRS {
        let dir_path = root.join(dir);
        if !dir_path.exists() {
            continue;
        }
        for entry in fs::read_dir(&dir_path).map_err(|source| DaedalusError::Io {
            path: dir_path.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| DaedalusError::Io {
                path: dir_path.clone(),
                source,
            })?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("md") {
                continue;
            }
            if path.file_name().and_then(|value| value.to_str()) == Some("README.md") {
                continue;
            }
            let content = fs::read_to_string(&path).map_err(|source| DaedalusError::Io {
                path: path.clone(),
                source,
            })?;
            items.push(KnowledgeSnapshot {
                level: dir.to_owned(),
                name: first_heading(&content).unwrap_or_else(|| {
                    path.file_stem()
                        .and_then(|value| value.to_str())
                        .unwrap_or("untitled")
                        .to_owned()
                }),
                path: path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string(),
                status: frontmatter_value(&content, "status").unwrap_or_else(|| "draft".to_owned()),
            });
        }
    }
    items.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(items)
}

fn validate_knowledge_base(root: &Path) -> Result<Vec<String>> {
    let mut issues = Vec::new();
    if !root.exists() {
        issues.push("knowledge-base missing".to_owned());
        return Ok(issues);
    }
    for dir in REQUIRED_KNOWLEDGE_DIRS {
        if !root.join(dir).is_dir() {
            issues.push(format!("knowledge-base missing directory: {dir}"));
        }
    }
    if !root.join("index.toml").is_file() {
        issues.push("knowledge-base missing index.toml; run daedalus knowledge index".to_owned());
    }
    for item in collect_knowledge_snapshots_from_root(root)? {
        let path = root.join(&item.path);
        let content = fs::read_to_string(&path).map_err(|source| DaedalusError::Io {
            path: path.clone(),
            source,
        })?;
        for heading in REQUIRED_ENTRY_HEADINGS {
            if !content.contains(heading) {
                issues.push(format!(
                    "knowledge entry `{}` missing heading: {heading}",
                    item.path
                ));
            }
        }
        if !content.contains("source = ") {
            issues.push(format!(
                "knowledge entry `{}` missing frontmatter source",
                item.path
            ));
        }
    }
    Ok(issues)
}

fn link_check_knowledge_base(root: &Path) -> Result<Vec<String>> {
    let mut issues = Vec::new();
    if !root.exists() {
        issues.push("knowledge-base missing".to_owned());
        return Ok(issues);
    }
    for file in markdown_files(root)? {
        let content = fs::read_to_string(&file).map_err(|source| DaedalusError::Io {
            path: file.clone(),
            source,
        })?;
        for link in markdown_links(&content) {
            if is_external_or_anchor(&link) {
                continue;
            }
            let without_fragment = link.split('#').next().unwrap_or("");
            if without_fragment.is_empty() {
                continue;
            }
            let target = file.parent().unwrap_or(root).join(without_fragment);
            if !target.exists() {
                issues.push(format!(
                    "broken local link in `{}`: {}",
                    file.strip_prefix(root).unwrap_or(&file).display(),
                    link
                ));
            }
        }
    }
    Ok(issues)
}

fn markdown_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(root) {
        let entry = entry.map_err(|source| DaedalusError::Io {
            path: root.to_path_buf(),
            source: std::io::Error::other(source),
        })?;
        if entry.file_type().is_file()
            && entry.path().extension().and_then(|value| value.to_str()) == Some("md")
        {
            files.push(entry.path().to_path_buf());
        }
    }
    Ok(files)
}

fn markdown_links(content: &str) -> Vec<String> {
    let mut links = Vec::new();
    for line in content.lines() {
        let mut rest = line;
        while let Some(start) = rest.find("](") {
            rest = &rest[start + 2..];
            let Some(end) = rest.find(')') else {
                break;
            };
            links.push(rest[..end].to_owned());
            rest = &rest[end + 1..];
        }
    }
    links
}

fn is_external_or_anchor(link: &str) -> bool {
    link.starts_with('#')
        || link.starts_with("http://")
        || link.starts_with("https://")
        || link.starts_with("mailto:")
}

fn plural_dir(kind: &str) -> Result<&'static str> {
    match kind {
        "concept" => Ok("concepts"),
        "skill" => Ok("skills"),
        "pattern" => Ok("patterns"),
        "problem" => Ok("problems"),
        "case" => Ok("cases"),
        "source-map" => Ok("source-maps"),
        "tree" => Ok("trees"),
        "drill" => Ok("drills"),
        _ => Err(DaedalusError::InvalidKnowledgeOperation(format!(
            "unknown knowledge kind: {kind}"
        ))),
    }
}

fn first_heading(content: &str) -> Option<String> {
    content
        .lines()
        .find_map(|line| line.strip_prefix("# ").map(|value| value.trim().to_owned()))
}

fn frontmatter_value(content: &str, key: &str) -> Option<String> {
    let mut lines = content.lines();
    if lines.next()? != "---" {
        return None;
    }
    for line in lines {
        if line == "---" {
            return None;
        }
        let Some((left, right)) = line.split_once('=') else {
            continue;
        };
        if left.trim() == key {
            return Some(right.trim().trim_matches('"').to_owned());
        }
    }
    None
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
