//! 时间来源适配器。

/// 返回当前本地时区的简洁时间字符串。
pub fn now_local_timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
