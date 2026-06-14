# Rust 接入 OpenAI-compatible Chat Completions 指南

这份 guide 服务 `08-demo-coder` 的 Slice 6：把当前 Codex mini demo 从局部函数推进到真实模型驱动的 Agent Orchestrator。

当前实现目标不是 OpenAI Responses API，而是 **DeepSeek / OpenAI-compatible Chat Completions streaming**：

```text
user messages
  -> OpenAI-compatible chat/completions stream
  -> tool call: run_command
  -> approval / sandbox / retry
  -> tool result observation
  -> next model request
  -> final answer
```

Responses API 可以作为后续 adapter 方案，但不要把它和当前 `openai.rs` 的请求/事件协议混在一起。

## 1. 先贴近真实协议，不急着造 Message 抽象

当前阶段建议直接使用 provider 兼容的 Chat Completions 形态：

```text
messages / tools / stream chunk 是模型层协议。
CommandRequest / ApprovalRequirement / RetryDecision / StreamEvent 是 demo 领域协议。
```

不要一开始就自定义一套完整跨 provider message 类型。原因是：

```text
1. 当前目标是验证本地命令安全执行链路，不是先做多模型 SDK。
2. DeepSeek 的兼容接口已经能给出 messages、tools、tool_calls、stream chunk。
3. 过早封装容易遮住真实 raw stream 的边界，反而不利于学习。
```

后面真的要支持 Responses API、Anthropic 或本地模型，再在 `Llm` trait 后面新增 adapter。

## 2. Rust 里怎么接

当前 demo 推荐直接 HTTP：

```text
reqwest 发请求
bytes_stream 读取 streaming body
buffer 按 SSE event 切块
serde_json 解析 data JSON
map_openai_event 映射为 StreamEvent
```

建议依赖：

```toml
[dependencies]
anyhow = "1"
async-stream = "0.3"
async-trait = "0.1"
futures-core = "0.3"
futures-util = "0.3"
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

API key 从环境变量读取，例如：

```text
DEEPSEEK_API_KEY
```

不要把 key 写进源码、notes 或 demo README。

## 3. 请求应该长什么样

当前最小请求是：

```http
POST https://api.deepseek.com/chat/completions
Authorization: Bearer $DEEPSEEK_API_KEY
Content-Type: application/json
```

请求体第一版可以这样：

```json
{
  "model": "deepseek-v4-flash",
  "messages": [
    {
      "role": "system",
      "content": "You are a local command agent. Use run_command only when a local command is necessary."
    },
    {
      "role": "user",
      "content": "帮我运行 npm install vite"
    }
  ],
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "run_command",
        "description": "Request execution of one local command. The host application will approve, sandbox, and execute it.",
        "parameters": {
          "type": "object",
          "properties": {
            "command": {
              "type": "string",
              "description": "The shell command to run."
            },
            "justification": {
              "type": "string",
              "description": "Why this command is needed."
            }
          },
          "required": ["command", "justification"],
          "additionalProperties": false
        }
      }
    }
  ],
  "stream": true
}
```

这里 `run_command` 不是让模型真的运行命令。模型只提出 tool call；本地程序再走：

```text
parse command
-> capability registry match
-> decide_approval
-> sandbox runner
-> decide_retry
```

## 4. Rust 请求骨架

先用 `serde_json::json!` 直接构造 body，等结构稳定后再改成强类型。

```rust
use anyhow::{Context, Result};
use futures_util::StreamExt;
use reqwest::Client;
use serde_json::json;

pub async fn print_chat_stream(user_text: &str) -> Result<()> {
    let api_key = std::env::var("DEEPSEEK_API_KEY")
        .context("DEEPSEEK_API_KEY is required")?;

    let body = json!({
        "model": "deepseek-v4-flash",
        "messages": [
            {
                "role": "system",
                "content": "You are a local command agent."
            },
            {
                "role": "user",
                "content": user_text
            }
        ],
        "stream": true
    });

    let response = Client::new()
        .post("https://api.deepseek.com/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?
        .error_for_status()?;

    let mut bytes = response.bytes_stream();
    while let Some(chunk) = bytes.next().await {
        let chunk = chunk?;
        print!("{}", String::from_utf8_lossy(&chunk));
    }

    Ok(())
}
```

第一步先打印原始 SSE chunk，确认真实响应长什么样。不要凭想象写 parser。

## 5. Streaming chunk 怎么映射

当前 DeepSeek / OpenAI-compatible Chat Completions stream 的核心形态是：

```text
data: {"object":"chat.completion.chunk","choices":[{"delta":{...},"finish_reason":null}]}

data: [DONE]
```

当前 demo 只关心这几类字段：

```text
choices[0].delta.reasoning_content -> StreamEvent::ThinkingDelta
choices[0].delta.content           -> StreamEvent::TextDelta
choices[0].delta.tool_calls        -> ToolCallArgumentsDelta / ToolCallStarted / ToolCallFinished
choices[0].finish_reason != null   -> StreamEvent::Completed
```

注意：

```text
Text delta 可以边到边显示。
Tool call arguments 不能边到边执行。
必须等参数分片聚合完成后，再 parse JSON、构造 CommandRequest。
```

原因是 function arguments 是分片到达的 JSON 字符串，任意中间片段都可能不是合法 JSON。

## 6. SSE 解析边界

HTTP chunk 不等于完整 SSE event。正确思路是：

```text
bytes_stream chunk
  -> 追加到 buffer
  -> 按 "\n\n" 切出完整 SSE block
  -> 读取 data 行
  -> 忽略 [DONE]
  -> serde_json::Value
  -> StreamEvent
```

对当前 DeepSeek raw stream 来说，按行读取 `data: ...` 是合理的第一版。当前实现可以先限定在这个范围：

```rust
fn take_sse_events(buffer: &mut String) -> Vec<String> {
    const EVENT_PREFIX: &str = "data: ";
    const DONE_FLAG: &str = "[DONE]";

    let mut events = vec![];
    while let Some(index) = buffer.find("\n\n") {
        let mut event = buffer[..index].to_string();
        buffer.drain(..index + 2);

        if event.starts_with(EVENT_PREFIX) {
            let event = event.split_off(EVENT_PREFIX.len());
            if event == DONE_FLAG {
                break;
            }
            events.push(event);
        }
    }
    events
}
```

这里的设计含义是：

```text
当前 parser 服务 DeepSeek / OpenAI-compatible Chat Completions 的常见 data-line stream。
它不是完整 SSE 规范 parser。
如果未来要兼容 event: xxx + 多行 data: xxx，需要单独升级 parser，并补 fixture tests。
```

## 7. Tool call 聚合规则

Chat Completions 的 tool call 可能分多片到达：

```json
{
  "choices": [
    {
      "delta": {
        "tool_calls": [
          {
            "index": 0,
            "id": "call_123",
            "type": "function",
            "function": {
              "name": "run_command",
              "arguments": "{\"command\":\"cargo"
            }
          }
        ]
      },
      "finish_reason": null
    }
  ]
}
```

下一片可能只给 arguments：

```json
{
  "choices": [
    {
      "delta": {
        "tool_calls": [
          {
            "index": 0,
            "function": {
              "arguments": " test\"}"
            }
          }
        ]
      },
      "finish_reason": "tool_calls"
    }
  ]
}
```

因此映射层要做三件事：

```text
第一次看到某个 index -> 发 ToolCallStarted，并创建 cache。
同一个 index 后续片段 -> append arguments。
遇到 finish_reason 或切到新 index -> 发 ToolCallFinished。
```

如果切到新 tool call，事件顺序应该是：

```text
ToolCallFinished(old)
ToolCallStarted(new)
cache = new
```

这样不会把旧 tool 的参数串到新 tool 里。

## 8. SSE 解析层不要污染 Orchestrator

建议保持三层：

```text
openai.rs
  - 发 HTTP 请求
  - 读取 SSE bytes
  - 映射为 StreamEvent

orchestrator.rs
  - 消费 StreamEvent
  - 遇到 ToolCallFinished 后 parse arguments
  - 进入 approval / runner / retry

domain modules
  - approval.rs / sandbox / retry.rs
  - 不知道 OpenAI JSON 长什么样
```

orchestrator 不应该直接写：

```rust
if value["choices"][0]["delta"]["tool_calls"].is_array() { ... }
```

它应该只看：

```rust
StreamEvent::ToolCallFinished { name, arguments, .. }
```

## 9. Tool result 怎么回给模型

模型提出 tool call 后，应用程序执行本地工具，然后把 tool result 作为下一轮 message 交回 Chat Completions。

第一版可以把结果放进 `role = "tool"` 的消息里：

```json
{
  "role": "tool",
  "tool_call_id": "call_123",
  "content": "{\"status\":\"sandbox_denied_then_retry_success\",\"stdout\":\"simulated install success without sandbox\"}"
}
```

对 demo 来说，tool output 可以包含：

```text
approval requirement
approval decision
execution attempt
execution result
retry decision
final tool result
```

当前 Slice 6 的重点是跑通闭环；结果 JSON 的字段名可以等 orchestrator 写到一半再收敛。

## 10. 当前 demo 的最小实现顺序

不要一口气写完整 agent：

```text
1. 给 take_sse_events / map_openai_event / map_chunk 补 recorded fixture 单测。
2. 把 OpenAiCompatibleLlm::default() 改成显式 from_env()，避免缺 key 时 panic。
3. 定义 orchestrator 最小 loop：model stream -> ToolCallFinished -> execute tool -> append tool message。
4. 接入已有 approval / runner / retry。
5. main.rs 接收用户输入，调用 run_agent。
```

每一步都要能 `cargo test` 或 `cargo check`。

## 11. 测试策略

真实模型调用不要放进默认 `cargo test`：

```text
默认单测：测试 event parser、tool call aggregation、orchestrator 状态机。
手动验收：需要 DEEPSEEK_API_KEY 的 cargo run / ignored test。
```

可以用 recorded SSE fixture 测 parser：

```text
data: {"choices":[{"delta":{"reasoning_content":"想"},"finish_reason":null}]}

data: {"choices":[{"delta":{"content":"你好"},"finish_reason":null}]}

data: {"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_1","function":{"name":"run_command","arguments":"{\"command\":\"cargo"}}]},"finish_reason":null}]}

data: {"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":" test\"}"}}]},"finish_reason":"tool_calls"}]}

data: [DONE]
```

这样测试不会联网、不会花钱，也不会受模型随机输出影响。

## 12. 当前设计结论

本阶段采用：

```text
OpenAI-compatible Chat Completions request format
async streaming HTTP
thin StreamEvent adapter
domain-specific command execution pipeline
```

暂不采用：

```text
OpenAI Responses API
通用 Message 抽象
多 provider adapter
真实 OS sandbox
完整 shell parser
```
