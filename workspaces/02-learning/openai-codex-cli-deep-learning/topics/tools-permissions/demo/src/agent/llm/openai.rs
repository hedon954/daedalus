use std::str::FromStr;

use crate::agent::{
    llm::{LLmRequest, Llm},
    stream_event::{EventStream, StreamEvent, ToolCallFinished},
};
use async_stream::try_stream;
use async_trait::async_trait;
use derive_builder::Builder;
use futures_util::StreamExt;
use reqwest::Client;
use serde_json::json;

#[derive(Builder)]
#[builder(default)]
pub struct OpenAiCompatibleLlm {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub temperature: f32,
    pub tools: Vec<serde_json::Value>,
}

struct ToolCallCache {
    index: i64,
    id: String,
    name: String,
    arguments: String,
}

impl Default for OpenAiCompatibleLlm {
    fn default() -> Self {
        Self {
            base_url: "https://api.deepseek.com/chat/completions".to_string(),
            api_key: std::env::var("DEEPSEEK_API_KEY").expect("DEEPSEEK_API_KEY is not set"),
            model: "deepseek-v4-pro".to_string(),
            temperature: 0.7,
            tools: vec![],
        }
    }
}

#[async_trait]
impl Llm for OpenAiCompatibleLlm {
    async fn stream(&self, req: LLmRequest) -> anyhow::Result<EventStream> {
        let body = json!({
            "model": self.model,
            "messages": req.messages,
            "tools": self.tools,
            "stream": true,
        });

        let response = Client::new()
            .post(&self.base_url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let mut first = true;
        let mut bytes_stream = response.bytes_stream();
        let mut tool_calls = vec![];

        let event_stream = try_stream! {
            let mut buffer = String::new();

            while let Some(chunk) = bytes_stream.next().await {
                let chunk = chunk?;
                let text = std::str::from_utf8(&chunk)?;
                buffer.push_str(text);

                for raw_event in take_sse_events(&mut buffer) {
                    let value = sse_event_to_json(&raw_event)?;
                    for e in map_chunk(&value, &mut first, &mut tool_calls) {
                        yield e;
                    }
                }
            }
        };

        Ok(Box::pin(event_stream))
    }
}

fn map_chunk(
    value: &serde_json::Value,
    first: &mut bool,
    tool_calls: &mut Vec<ToolCallCache>,
) -> Vec<StreamEvent> {
    let mut result = vec![];

    let events = map_openai_event(value);

    if *first {
        result.push(StreamEvent::Started);
        *first = false;
    }

    for event in events {
        match event {
            StreamEvent::ToolCallArgumentsDelta {
                index,
                call_id,
                name,
                delta,
            } => {
                if tool_calls.len() <= index as usize {
                    tool_calls.push(ToolCallCache {
                        index,
                        id: call_id,
                        name: name.unwrap_or_default(),
                        arguments: delta,
                    });
                } else {
                    tool_calls[index as usize].arguments.push_str(&delta);
                }
            }
            StreamEvent::Completed => {
                for tc in tool_calls.iter() {
                    result.push(StreamEvent::ToolCallFinished(ToolCallFinished {
                        index: tc.index,
                        call_id: tc.id.clone(),
                        name: tc.name.clone(),
                        arguments: tc.arguments.clone(),
                    }));
                }
                tool_calls.clear();
                result.push(StreamEvent::Completed);
            }
            other => result.push(other),
        }
    }
    result
}

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

fn sse_event_to_json(event: &str) -> anyhow::Result<serde_json::Value> {
    let value = serde_json::Value::from_str(event)?;
    Ok(value)
}

fn map_openai_event(value: &serde_json::Value) -> Vec<StreamEvent> {
    /*
        - thinking
        {
            "id": "b68e26f7-6d18-4c97-8b0a-972e6da57610",
            "object": "chat.completion.chunk",
            "created": 1779288802,
            "model": "deepseek-v4-flash",
            "system_fingerprint": "fp_8b330d02d0_prod0820_fp8_kvcache_20260402",
            "choices": [
                {
                    "index": 0,
                    "delta": {
                        "content": null,
                        "reasoning_content": "问"
                    },
                    "logprobs": null,
                    "finish_reason": null
                }
            ]
        }

        - text
        {
            "id": "b68e26f7-6d18-4c97-8b0a-972e6da57610",
            "object": "chat.completion.chunk",
            "created": 1779288802,
            "model": "deepseek-v4-flash",
            "system_fingerprint": "fp_8b330d02d0_prod0820_fp8_kvcache_20260402",
            "choices": [
                {
                    "index": 0,
                    "delta": {
                        "content": "你好",
                        "reasoning_content": null
                    },
                    "logprobs": null,
                    "finish_reason": null
                }
            ]
        }

        - end
        {
            "id": "b68e26f7-6d18-4c97-8b0a-972e6da57610",
            "object": "chat.completion.chunk",
            "created": 1779288802,
            "model": "deepseek-v4-flash",
            "system_fingerprint": "fp_8b330d02d0_prod0820_fp8_kvcache_20260402",
            "choices": [
                {
                    "index": 0,
                    "delta": {
                        "content": "",
                        "reasoning_content": null
                    },
                    "logprobs": null,
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 11,
                "completion_tokens": 123,
                "total_tokens": 134,
                "prompt_tokens_details": {
                    "cached_tokens": 0
                },
                "completion_tokens_details": {
                    "reasoning_tokens": 114
                },
                "prompt_cache_hit_tokens": 0,
                "prompt_cache_miss_tokens": 11
            }
        }

        - tool call
        {
            "id": "2ce31dd0-6acb-4521-9cfe-def9575a1a0a",
            "object": "chat.completion.chunk",
            "created": 1779291486,
            "model": "deepseek-v4-flash",
            "system_fingerprint": "fp_8b330d02d0_prod0820_fp8_kvcache_20260402",
            "choices": [
                {
                    "index": 0,
                    "delta": {
                        "tool_calls": [
                            {
                                "index": 0,
                                "id": "call_00_PIw4wxbGmJNMWdK3uv3v4926",
                                "type": "function",
                                "function": {
                                    "name": "add",
                                    "arguments": ""
                                }
                            }
                        ]
                    },
                    "logprobs": null,
                    "finish_reason": null
                }
            ]
        }
    */

    let mut result = vec![];

    let choice = &value["choices"][0];
    let delta = &choice["delta"];

    // thinking
    if delta["content"].is_null() && !delta["reasoning_content"].is_null() {
        result.push(StreamEvent::ThinkingDelta(value_as_str(
            &delta["reasoning_content"],
        )));
    }

    // content
    if !delta["content"].is_null() {
        result.push(StreamEvent::TextDelta(value_as_str(&delta["content"])));
    }

    // tool_call
    for tool_call in delta["tool_calls"].as_array().unwrap_or(&vec![]) {
        let tool_call_index = tool_call["index"].as_i64().unwrap_or_default();
        let tool_call_id = value_as_str(&tool_call["id"]);
        let tool_call_name = value_as_str(&tool_call["function"]["name"]);
        let tool_cal_args = value_as_str(&tool_call["function"]["arguments"]);
        result.push(StreamEvent::ToolCallArgumentsDelta {
            index: tool_call_index,
            call_id: tool_call_id,
            name: if tool_call_name.is_empty() {
                None
            } else {
                Some(tool_call_name)
            },
            delta: tool_cal_args,
        });
    }

    // finish
    if !choice["finish_reason"].is_null() {
        result.push(StreamEvent::Completed);
    }

    result
}

fn value_as_str(v: &serde_json::Value) -> String {
    v.as_str().unwrap_or("").to_string()
}

#[cfg(test)]
mod tests {
    use crate::agent::tool::{add_spec, run_tool, sub_spec};

    use super::*;

    async fn stream(user_text: &str) -> anyhow::Result<()> {
        let body = json!({
            "model": "deepseek-v4-flash",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a local command agent, you should use tools to do calculate"
                },
                {
                    "role": "user",
                    "content": user_text
                }
            ],
            "tools": [add_spec(), sub_spec()],
            "stream": true,
        });

        let response = Client::new()
            .post("https://api.deepseek.com/chat/completions".to_string())
            .bearer_auth(std::env::var("DEEPSEEK_API_KEY").expect("DEEPSEEK_API_KEY is not set"))
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let mut bytes_stream = response.bytes_stream();

        let mut buffer = String::new();
        while let Some(chunk) = bytes_stream.next().await {
            let chunk = chunk?;
            let text = std::str::from_utf8(&chunk)?;
            buffer.push_str(text);

            let events = take_sse_events(&mut buffer);

            for event in events {
                let event = sse_event_to_json(&event)?;
                let es = map_openai_event(&event);
                for e in es {
                    println!("{:?}", e);
                }
            }
        }

        Ok(())
    }

    #[tokio::test]
    #[ignore = "do not run real llm invoke in test"]
    async fn run_stream() {
        let result = stream("999+666和999-666各等于多少").await;
        println!("result: {:?}", result)
    }

    #[tokio::test]
    #[ignore = "do not run real llm invoke in test"]
    async fn openai_compatible_llm_should_work() -> anyhow::Result<()> {
        let llm = OpenAiCompatibleLlmBuilder::default()
            .tools(vec![add_spec(), sub_spec()])
            .build()?;

        let mut stream = llm
            .stream(LLmRequest {
                messages: vec![
                    json!({
                        "role": "system",
                        "content": "You are a local command agent, you should use tools to do calculate",
                    }),
                    json!({
                        "role": "user",
                        "content": "999+666和999-666各等于多少"
                    }),
                ],
            })
            .await?;

        while let Some(event) = stream.next().await {
            match event? {
                StreamEvent::Started => {
                    println!("start ----->")
                }
                StreamEvent::ThinkingDelta(s) => print!("{s}"),
                StreamEvent::TextDelta(s) => print!("{s}"),
                StreamEvent::ToolCallFinished(ToolCallFinished {
                    index: _,
                    call_id: _,
                    name,
                    arguments,
                }) => {
                    let out = run_tool(&name, &arguments)?;
                    println!("\ntool {name}: {out}");
                }
                StreamEvent::Completed => break,
                other => println!("{other:?}"),
            }
        }

        Ok(())
    }
}
