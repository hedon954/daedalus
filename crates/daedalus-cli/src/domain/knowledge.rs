//! knowledge system 领域类型。

/// 知识晋升目标。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgePromotionTarget {
    /// 晋升到 project shared context。
    Shared,
    /// 导出为 knowledge-base candidate。
    KnowledgeBase,
}

impl KnowledgePromotionTarget {
    /// 返回稳定字符串。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Shared => "shared",
            Self::KnowledgeBase => "knowledge-base",
        }
    }

    /// 从字符串解析。
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "shared" => Some(Self::Shared),
            "knowledge-base" => Some(Self::KnowledgeBase),
            _ => None,
        }
    }
}

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
