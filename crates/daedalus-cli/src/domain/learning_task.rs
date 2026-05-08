use std::path::PathBuf;

/// 一个 daedalus 学习任务的最小领域表示。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearningTask {
    /// 学习任务名称。
    pub name: String,
    /// 学习任务在文件系统中的根目录。
    pub path: PathBuf,
}
