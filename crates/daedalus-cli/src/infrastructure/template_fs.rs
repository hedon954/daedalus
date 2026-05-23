use std::fs;
use std::path::Path;

use walkdir::WalkDir;

use crate::domain::{DaedalusError, Result};

/// 复制模板目录并执行简单占位符替换。
///
/// v1 使用纯文本替换而不是模板引擎，避免在需求明确前引入额外复杂度。
pub fn copy_template_dir(
    template_dir: &Path,
    target_dir: &Path,
    replacements: &[(&str, &str)],
) -> Result<()> {
    copy_template_dir_inner(template_dir, target_dir, replacements, true)
}

/// 复制模板目录，只创建缺失文件，不覆盖已有用户内容。
pub fn copy_template_dir_missing(
    template_dir: &Path,
    target_dir: &Path,
    replacements: &[(&str, &str)],
) -> Result<()> {
    copy_template_dir_inner(template_dir, target_dir, replacements, false)
}

fn copy_template_dir_inner(
    template_dir: &Path,
    target_dir: &Path,
    replacements: &[(&str, &str)],
    overwrite: bool,
) -> Result<()> {
    for entry in WalkDir::new(template_dir) {
        let entry = entry.map_err(|source| DaedalusError::Io {
            path: template_dir.to_path_buf(),
            source: std::io::Error::other(source),
        })?;
        let source_path = entry.path();
        let relative = source_path
            .strip_prefix(template_dir)
            .unwrap_or(source_path);
        let target_path = target_dir.join(relative);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target_path).map_err(|source| DaedalusError::Io {
                path: target_path,
                source,
            })?;
            continue;
        }
        if !overwrite && target_path.exists() {
            continue;
        }

        let mut content = fs::read_to_string(source_path).map_err(|source| DaedalusError::Io {
            path: source_path.to_path_buf(),
            source,
        })?;
        for (from, to) in replacements {
            content = content.replace(from, to);
        }
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|source| DaedalusError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::write(&target_path, content).map_err(|source| DaedalusError::Io {
            path: target_path,
            source,
        })?;
    }

    Ok(())
}
