/// 判断 argv 是否以 prefix 开头
pub fn args_has_prefix(argv: &[String], prefix: &[String]) -> bool {
    argv.len() >= prefix.len() && argv[..prefix.len()] == prefix[..]
}
