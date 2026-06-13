use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::{DaedalusError, KnowledgeSnapshot, Result};
use crate::infrastructure::clock;
use toml_edit::{DocumentMut, Item, Table, value};

const RESERVED_MARKDOWN_FILES: [&str; 2] = ["README.md", "index.md"];

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
    ensure_human_index(&root)?;
    Ok(root)
}

/// 生成一篇空白知识笔记骨架。
pub fn create_knowledge_template(
    repo_root: &Path,
    entry_path: &str,
    title: Option<&str>,
) -> Result<KnowledgeOutput> {
    let root = ensure_knowledge_base_layout(repo_root)?;
    let path = normalize_entry_path(&root, entry_path)?;
    let title = title
        .filter(|value| !value.trim().is_empty())
        .map(str::trim)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| title_from_path(&path));
    if path.exists() {
        return Err(DaedalusError::InvalidKnowledgeOperation(format!(
            "knowledge entry already exists: {}",
            path.display()
        )));
    }
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    let content = format!(
        "---\nstatus = \"draft\"\ncreated_at = \"{}\"\n---\n\n# {title}\n",
        clock::now_local_timestamp()
    );
    fs::write(&path, content).map_err(|source| DaedalusError::Io {
        path: path.clone(),
        source,
    })?;
    Ok(KnowledgeOutput {
        action: "knowledge-template".to_owned(),
        path,
        next: "根据这篇笔记的主题按需组织内容，补齐外部资料、第一性原理、底层原理和学习来源链接，然后运行 daedalus knowledge index。".to_owned(),
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
            table["level"] = value(entry.level);
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

fn ensure_human_index(root: &Path) -> Result<()> {
    let path = root.join("index.md");
    if !path.exists() {
        fs::write(
            &path,
            "# 知识库导航\n\n这里是人可读的知识树入口。机器索引见 `index.toml`。\n",
        )
        .map_err(|source| DaedalusError::Io { path, source })?;
    }
    Ok(())
}

fn collect_knowledge_snapshots_from_root(root: &Path) -> Result<Vec<KnowledgeSnapshot>> {
    let mut items = Vec::new();
    for path in markdown_files(root)? {
        if !is_knowledge_entry_file(root, &path) {
            continue;
        }
        let content = fs::read_to_string(&path).map_err(|source| DaedalusError::Io {
            path: path.clone(),
            source,
        })?;
        let relative = path.strip_prefix(root).unwrap_or(&path);
        items.push(KnowledgeSnapshot {
            level: knowledge_level(relative),
            name: first_heading(&content).unwrap_or_else(|| title_from_path(&path)),
            path: relative.to_string_lossy().to_string(),
            status: frontmatter_value(&content, "status").unwrap_or_else(|| "draft".to_owned()),
        });
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
    let index_path = root.join("index.toml");
    let entries = collect_knowledge_snapshots_from_root(root)?;
    if !index_path.is_file() {
        issues.push("knowledge-base missing index.toml; run daedalus knowledge index".to_owned());
    } else {
        validate_knowledge_index(root, &index_path, &entries, &mut issues)?;
    }
    for item in entries {
        let path = root.join(&item.path);
        let content = fs::read_to_string(&path).map_err(|source| DaedalusError::Io {
            path: path.clone(),
            source,
        })?;
        if first_heading(&content).is_none() {
            issues.push(format!(
                "knowledge entry `{}` missing title heading",
                item.path
            ));
        }
    }
    Ok(issues)
}

fn validate_knowledge_index(
    root: &Path,
    index_path: &Path,
    entries: &[KnowledgeSnapshot],
    issues: &mut Vec<String>,
) -> Result<()> {
    let content = fs::read_to_string(index_path).map_err(|source| DaedalusError::Io {
        path: index_path.to_path_buf(),
        source,
    })?;
    let Ok(doc) = content.parse::<DocumentMut>() else {
        issues
            .push("knowledge-base index.toml is invalid; run daedalus knowledge index".to_owned());
        return Ok(());
    };
    let Some(indexed_entries) = doc["entries"].as_array_of_tables() else {
        issues.push(
            "knowledge-base index.toml missing entries array; run daedalus knowledge index"
                .to_owned(),
        );
        return Ok(());
    };

    let current_paths = entries
        .iter()
        .map(|entry| entry.path.clone())
        .collect::<BTreeSet<_>>();
    let mut indexed_paths = BTreeSet::new();
    for entry in indexed_entries {
        let Some(path) = entry.get("path").and_then(|item| item.as_str()) else {
            issues.push(
                "knowledge-base index.toml contains entry without path; run daedalus knowledge index"
                    .to_owned(),
            );
            continue;
        };
        indexed_paths.insert(path.to_owned());
        if !root.join(path).is_file() {
            issues.push(format!(
                "knowledge-base index.toml points to missing entry `{path}`; run daedalus knowledge index"
            ));
        }
    }
    for path in current_paths.difference(&indexed_paths) {
        issues.push(format!(
            "knowledge entry `{path}` missing from index.toml; run daedalus knowledge index"
        ));
    }
    Ok(())
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

fn normalize_entry_path(root: &Path, value: &str) -> Result<PathBuf> {
    let relative = Path::new(value);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(DaedalusError::InvalidKnowledgeOperation(format!(
            "knowledge entry path must stay inside knowledge-base: {value}"
        )));
    }
    let mut path = root.join(relative);
    if path.extension().and_then(|value| value.to_str()) != Some("md") {
        path.set_extension("md");
    }
    Ok(path)
}

fn title_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("untitled")
        .replace(['-', '_'], " ")
}

fn is_knowledge_entry_file(root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    if relative.components().count() == 1
        && relative
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| RESERVED_MARKDOWN_FILES.contains(&name))
    {
        return false;
    }
    path.file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| name != "README.md")
}

fn knowledge_level(relative: &Path) -> String {
    relative
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(|parent| parent.to_string_lossy().to_string())
        .unwrap_or_else(|| "root".to_owned())
}
