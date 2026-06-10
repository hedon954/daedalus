//! knowledge system 领域类型。

/// Knowledge system 列表展示快照。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeSnapshot {
    /// 所属层级：topic / shared / knowledge-base。
    pub level: String,
    /// 展示名称。
    pub name: String,
    /// 路径。
    pub path: String,
    /// 状态说明。
    pub status: String,
}
