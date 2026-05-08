//! 时间来源适配器。

/// 返回当前 UTC 时间的 RFC3339 字符串。
pub fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}
