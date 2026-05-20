# Rust 接入 OpenAI 指南

这份 guide 服务 `08-demo-coder` 的 Slice 6：把当前 Codex mini demo 从局部函数推进到真实模型驱动的 Agent Orchestrator。

目标不是先封装一个通用 LLM SDK，而是在 Rust 中用 OpenAI Responses API 跑通：

```text
user input
  -> OpenAI streaming response
  -> function call: run_command
  -> approval / sandbox / retry
  -> function call output
  -> next OpenAI response
  -> final answer
```

## 1. 先选 OpenAI-native，而不是先造 Message 抽象

当前阶段建议直接贴 OpenAI Responses API：

```text
OpenAI input / tools / streaming events 是模型层协议。
CommandRequest / ApprovalRequirement / RetryDecision / AgentEvent 是 demo 领域协议。
```

不要一开始就自定义：

```rust
enum Message {
    System,
    User,
    Assistant,
    Tool,
}
```

原因：

```text
1. OpenAI Responses API 已经有 input、instructions、tools、function call output、previous_response_id 等概念。
2. 我们当前要验证的是本地命令安全执行链路，不是多模型兼容层。
3. 过早封装 Message 很容易和真实 API 的 event / tool call 结构不完全一致。
```

后面真的要支持 Anthropic、本地模型或多 provider，再抽自己的 adapter。

## 2. Rust 里怎么接 OpenAI

OpenAI 官方 SDK 页面列出 Rust 为 community library，并提醒这些项目不是 OpenAI 官方验证的正确性或安全性保证；当前列出的 Rust community library 是 `async-openai`。因此 demo 有两条路：

```text
路线 A：直接 HTTP + reqwest + 自己解析 SSE。
路线 B：使用 async-openai 这类 community crate。
```

本 demo 推荐路线 A：

```text
更少魔法。
更容易看清 Responses API 的真实输入输出。
更符合我们学习 agent loop 和 stream event 的目标。
```

建议依赖：

```toml
[dependencies]
anyhow = "1"
futures-util = "0.3"
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

API key 从环境变量读取：

```text
OPENAI_API_KEY
```

不要把 key 写进源码、notes 或 demo README。

## 3. 请求应该长什么样

最小请求是：

```http
POST https://api.openai.com/v1/responses
Authorization: Bearer $OPENAI_API_KEY
Content-Type: application/json
```

请求体第一版可以这样：

```json
{
  "model": "gpt-5.5",
  "instructions": "You are a local command agent. Use run_command only when a local command is necessary.",
  "input": [
    {
      "role": "user",
      "content": "帮我运行 npm install vite"
    }
  ],
  "tools": [
    {
      "type": "function",
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
      },
      "strict": true
    }
  ],
  "stream": true
}
```

这里 `run_command` 不是让模型真的运行命令。模型只提出 tool call；本地程序再走：

```text
parse command
-> registry match
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

pub async fn stream_openai_response(user_text: &str) -> Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .context("OPENAI_API_KEY is required")?;

    let body = json!({
        "model": "gpt-5.5",
        "instructions": "You are a local command agent. Use run_command only when a local command is necessary.",
        "input": [
            {
                "role": "user",
                "content": user_text
            }
        ],
        "tools": [
            {
                "type": "function",
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
                },
                "strict": true
            }
        ],
        "stream": true
    });

    let response = Client::new()
        .post("https://api.openai.com/v1/responses")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?
        .error_for_status()?;

    let mut bytes = response.bytes_stream();
    while let Some(chunk) = bytes.next().await {
        let chunk = chunk?;
        // 下一步：把 bytes 解成 SSE event，再映射为 demo 的 ModelStreamEvent。
        print!("{}", String::from_utf8_lossy(&chunk));
    }

    Ok(())
}
```

第一步先打印原始 SSE chunk，确认 API 连通。不要急着把所有 event 都强类型化。

## 5. Streaming 事件怎么映射

OpenAI Responses streaming 使用 typed semantic events。当前 demo 只需要关心这几类：

```text
response.created
response.output_text.delta
response.output_item.added
response.function_call_arguments.delta
response.function_call_arguments.done
response.completed
error
```

我们可以先定义一个 demo 自己的薄事件：

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelStreamEvent {
    Started,
    TextDelta(String),
    ToolCallStarted {
        output_index: usize,
        call_id: String,
        name: String,
    },
    ToolCallArgumentsDelta {
        output_index: usize,
        delta: String,
    },
    ToolCallFinished {
        output_index: usize,
        call_id: String,
        name: String,
        arguments: String,
    },
    Completed,
    Error(String),
}
```

注意：

```text
Text delta 可以边到边显示。
Function call arguments 不能边到边执行。
必须等 arguments done 后再 parse JSON、构造 CommandRequest。
```

原因是 streaming function call 的参数是分片到达的 JSON 字符串，只有 `done` 事件才代表参数完整。

## 6. 流式 SSE 解析示例

OpenAI streaming 返回的是 Server-Sent Events。一个事件通常长这样：

```text
event: response.output_text.delta
data: {"type":"response.output_text.delta","delta":"hello"}

```

HTTP chunk 不一定刚好等于一个完整事件。正确思路是：

```text
bytes_stream chunk
  -> 追加到 buffer
  -> 按 "\n\n" 切出完整 SSE event
  -> 从 event 里取 data 行
  -> serde_json::Value
  -> ModelStreamEvent
```

先定义错误类型：

```rust
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelError {
    pub message: String,
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ModelError {}
```

再定义一个很窄的 SSE parser。它不试图支持全部 SSE 规范，只服务 OpenAI streaming：

```rust
use serde_json::Value;

fn take_sse_events(buffer: &mut String) -> Vec<String> {
    let mut events = Vec::new();

    while let Some(index) = buffer.find("\n\n") {
        let event = buffer[..index].to_string();
        buffer.drain(..index + 2);

        if !event.trim().is_empty() {
            events.push(event);
        }
    }

    events
}

fn sse_event_to_json(event: &str) -> Result<Option<Value>, ModelError> {
    let mut data_lines = Vec::new();

    for line in event.lines() {
        if let Some(data) = line.strip_prefix("data: ") {
            data_lines.push(data);
        }
    }

    if data_lines.is_empty() {
        return Ok(None);
    }

    let data = data_lines.join("\n");
    if data == "[DONE]" {
        return Ok(None);
    }

    serde_json::from_str(&data).map(Some).map_err(|error| ModelError {
        message: format!("failed to parse OpenAI SSE data as JSON: {error}; data: {data}"),
    })
}
```

然后把 OpenAI JSON event 映射成 demo 关心的 `ModelStreamEvent`：

```rust
fn map_openai_event(value: &Value) -> Option<ModelStreamEvent> {
    let event_type = value.get("type")?.as_str()?;

    match event_type {
        "response.created" => Some(ModelStreamEvent::Started),

        "response.output_text.delta" => {
            let delta = value.get("delta")?.as_str()?.to_string();
            Some(ModelStreamEvent::TextDelta(delta))
        }

        "response.output_item.added" => {
            let item = value.get("item")?;
            if item.get("type")?.as_str()? != "function_call" {
                return None;
            }

            Some(ModelStreamEvent::ToolCallStarted {
                output_index: value.get("output_index")?.as_u64()? as usize,
                call_id: item.get("call_id")?.as_str()?.to_string(),
                name: item.get("name")?.as_str()?.to_string(),
            })
        }

        "response.function_call_arguments.delta" => {
            Some(ModelStreamEvent::ToolCallArgumentsDelta {
                output_index: value.get("output_index")?.as_u64()? as usize,
                delta: value.get("delta")?.as_str()?.to_string(),
            })
        }

        "response.function_call_arguments.done" => {
            Some(ModelStreamEvent::ToolCallFinished {
                output_index: value.get("output_index")?.as_u64()? as usize,
                call_id: value.get("call_id")?.as_str()?.to_string(),
                name: value.get("name")?.as_str()?.to_string(),
                arguments: value.get("arguments")?.as_str()?.to_string(),
            })
        }

        "response.completed" => Some(ModelStreamEvent::Completed),

        "error" => Some(ModelStreamEvent::Error(
            value
                .get("message")
                .and_then(|message| message.as_str())
                .unwrap_or("unknown OpenAI streaming error")
                .to_string(),
        )),

        _ => None,
    }
}
```

最后把它接到 `reqwest` 的 `bytes_stream()`：

```rust
use futures_util::StreamExt;
use reqwest::Client;

pub async fn stream_model_events(
    client: &Client,
    api_key: &str,
    body: serde_json::Value,
) -> Result<Vec<ModelStreamEvent>, ModelError> {
    let response = client
        .post("https://api.openai.com/v1/responses")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|error| ModelError {
            message: format!("failed to send OpenAI request: {error}"),
        })?
        .error_for_status()
        .map_err(|error| ModelError {
            message: format!("OpenAI request failed: {error}"),
        })?;

    let mut buffer = String::new();
    let mut stream = response.bytes_stream();
    let mut model_events = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| ModelError {
            message: format!("failed to read OpenAI stream chunk: {error}"),
        })?;

        let text = std::str::from_utf8(&chunk).map_err(|error| ModelError {
            message: format!("OpenAI stream chunk was not valid UTF-8: {error}"),
        })?;

        buffer.push_str(text);

        for raw_event in take_sse_events(&mut buffer) {
            if let Some(value) = sse_event_to_json(&raw_event)? {
                if let Some(model_event) = map_openai_event(&value) {
                    model_events.push(model_event);
                }
            }
        }
    }

    Ok(model_events)
}
```

注意这个示例为了好理解，返回 `Vec<ModelStreamEvent>`。真正接 orchestrator 时，可以改成 channel 或 `impl Stream<Item = Result<ModelStreamEvent, ModelError>>`，边收到边处理。

当前实现顺序建议：

```text
1. 先写 take_sse_events / sse_event_to_json 的单测。
2. 再写 map_openai_event 的 fixture 单测。
3. 最后接 reqwest bytes_stream。
```

默认 `cargo test` 不应该请求 OpenAI；真实联网测试放到手动 runbook。

## 7. SSE 解析层不要污染 Orchestrator

建议分三层：

```text
openai_http.rs
  - 发 HTTP 请求
  - 读取 SSE bytes
  - 输出 serde_json::Value 或 OpenAiRawEvent

openai_model_client.rs
  - 把 OpenAI raw event 映射为 ModelStreamEvent
  - 聚合 function call arguments

agent.rs / orchestrator.rs
  - 消费 ModelStreamEvent
  - 遇到 ToolCallFinished 后进入 approval / runner / retry
```

不要让 orchestrator 直接写：

```rust
if event["type"] == "response.function_call_arguments.delta" { ... }
```

orchestrator 应该只看：

```rust
ModelStreamEvent::ToolCallFinished { name, arguments, .. }
```

## 8. Tool call output 怎么回给模型

模型提出 tool call 后，应用程序执行本地工具，然后把 tool result 作为下一轮 input 交回 Responses API。

对 demo 来说，tool output 可以包含：

```text
approval requirement
approval decision
execution attempt
execution result
retry decision
final tool result
```

结构上建议先用 JSON 字符串：

```json
{
  "status": "sandbox_denied_then_retry_success",
  "stdout": "simulated install success without sandbox",
  "events": [
    "approval: NeedsApproval",
    "execution: SandboxFirst -> SandboxDenied",
    "retry: RetryWithApproval",
    "execution: NoSandboxRetry -> Success"
  ]
}
```

后续再决定是否用 OpenAI 的 function call output item 结构精确表达。当前 Slice 6 的重点是跑通闭环。

## 9. 当前 demo 的最小实现顺序

不要一口气写完整 OpenAI client。建议按这个顺序：

```text
1. 新增 model_client.rs：定义 ModelStreamEvent、ModelError。
2. 新增 openai_http.rs：用 reqwest 发 stream 请求，先打印原始 event。
3. 新增 openai_model_client.rs：把 raw SSE event 映射成 ModelStreamEvent。
4. 新增 agent.rs：消费 ModelStreamEvent，遇到 ToolCallFinished 后进入已有 approval / runner / retry。
5. main.rs 接收用户输入，调用 run_agent。
```

每一步都要能 `cargo test` 或 `cargo check`。

## 10. 测试策略

真实 OpenAI 调用不要放进默认 `cargo test`：

```text
默认单测：测试 event parser、tool call aggregation、orchestrator 状态机。
手动验收：需要 OPENAI_API_KEY 的 cargo run / make run。
```

可以用 recorded SSE fixture 测 parser：

```text
response.output_item.added
response.function_call_arguments.delta
response.function_call_arguments.done
response.completed
```

这样测试不会联网、不会花钱，也不会受模型随机输出影响。

## 11. 当前设计结论

本阶段采用：

```text
OpenAI-native request format
async streaming HTTP
thin ModelStreamEvent adapter
domain-specific command execution pipeline
```

暂不采用：

```text
通用 Message 抽象
多 provider adapter
真实 OS sandbox
完整 shell parser
```

## Sources

- OpenAI Responses API reference: <https://platform.openai.com/docs/api-reference/responses?api-mode=responses>
- OpenAI streaming guide: <https://platform.openai.com/docs/api-reference/streaming>
- OpenAI function calling guide: <https://developers.openai.com/api/docs/guides/function-calling>
- OpenAI SDKs and CLI page: <https://developers.openai.com/api/docs/libraries>
