/// 判断 argv 是否以 prefix 开头。
///
/// 这是当前 capability registry 的最小匹配规则；它只适用于已经拆好的单段命令 argv。
pub fn args_has_prefix(argv: &[String], prefix: &[String]) -> bool {
    argv.len() >= prefix.len() && argv[..prefix.len()] == prefix[..]
}
