//! 学习产物领域类型。

/// 阶段完成前必须存在的产物要求。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactRequirement {
    /// 相对学习任务根目录的产物路径。
    pub path: String,
}
