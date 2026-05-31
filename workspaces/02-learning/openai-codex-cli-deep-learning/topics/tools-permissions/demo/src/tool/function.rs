use std::str::FromStr;

use openai_tool_project::openai_tool;

/// 计算两数之和，在需要做加法计算时，请调用它
#[openai_tool]
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

/// 计算两数之差，在需要做减法计算时，请调用它
#[openai_tool]
pub fn sub(a: i64, b: i64) -> i64 {
    a - b
}

/// 执行纯编译函数
pub fn run_pure_function(name: &str, arguments: &str) -> anyhow::Result<String> {
    let arguments = serde_json::Value::from_str(arguments)?;
    match name {
        "add" => {
            let a = arguments["a"]
                .as_i64()
                .ok_or(anyhow::anyhow!("a should be i64: {:?}", arguments["a"]))?;

            let b = arguments["b"]
                .as_i64()
                .ok_or(anyhow::anyhow!("b should be i64: {:?}", arguments["b"]))?;

            Ok(add(a, b).to_string())
        }
        "sub" => {
            let a = arguments["a"]
                .as_i64()
                .ok_or(anyhow::anyhow!("a should be i64: {:?}", arguments["a"]))?;

            let b = arguments["b"]
                .as_i64()
                .ok_or(anyhow::anyhow!("b should be i64: {:?}", arguments["b"]))?;

            Ok(sub(a, b).to_string())
        }
        _ => anyhow::bail!("unknown tool: {name}"),
    }
}
