use std::sync::Arc;

use futures_util::{StreamExt, stream};
use serde_json::{Value, json};
use tokio::sync::mpsc;

use crate::agent::{
    llm::{LLmRequest, Llm},
    stream_event::{EventStream, StreamEvent, ToolCallFinished},
    tool::run_tool,
};

type EventSender = mpsc::Sender<anyhow::Result<StreamEvent>>;

pub struct ReActAgent {
    llm: Arc<dyn Llm + Send + Sync>,
}

impl ReActAgent {
    pub fn new(llm: Arc<dyn Llm + Send + Sync>) -> Self {
        Self { llm }
    }

    pub fn run(&self, prompt: String) -> anyhow::Result<EventStream> {
        if prompt.is_empty() {
            anyhow::bail!("prompt should not be empty");
        }

        let llm = self.llm.clone();
        let (tx, rx) = mpsc::channel(64);

        tokio::spawn(async move {
            if let Err(err) = run_agent_loop(llm, prompt, tx.clone()).await {
                let _ = tx.send(Err(err)).await;
            }
        });

        let s = stream::unfold(rx, |mut rx| async {
            rx.recv().await.map(|event| (event, rx))
        });

        Ok(Box::pin(s))
    }
}

async fn run_agent_loop(
    llm: Arc<dyn Llm + Send + Sync>,
    prompt: String,
    tx: EventSender,
) -> anyhow::Result<()> {
    let mut messages = vec![
        json!({"role": "system", "content": ""}),
        json!({"role": "user", "content": prompt}),
    ];

    loop {
        let mut stream = llm
            .stream(LLmRequest {
                messages: messages.clone(),
            })
            .await?;

        let mut pending_tool_calls = vec![];
        let mut ai_content = String::new();
        let mut reasoning_content = String::new();

        while let Some(event) = stream.next().await {
            let event = event?;
            match event {
                StreamEvent::TextDelta(text) => {
                    ai_content.push_str(&text);
                    emit(&tx, StreamEvent::TextDelta(text)).await?;
                }
                StreamEvent::ThinkingDelta(text) => {
                    reasoning_content.push_str(&text);
                    emit(&tx, StreamEvent::ThinkingDelta(text)).await?;
                }
                StreamEvent::ToolCallFinished(tc) => pending_tool_calls.push(tc),
                StreamEvent::Completed => break,
                StreamEvent::Error(err) => anyhow::bail!(err),
                other => emit(&tx, other).await?,
            }
        }

        if pending_tool_calls.is_empty() {
            break;
        }

        pending_tool_calls.sort_by_key(|call| call.index);

        // 填充 messages
        messages.push(json!({
            "role": "assistant",
            "content": ai_content,
            "reasoning_content": reasoning_content,
            "tool_calls": pending_tool_calls.iter().map(|call| {
                json!({
                    "id": call.call_id,
                    "type": "function",
                    "function": {
                        "name": call.name,
                        "arguments": call.arguments,
                    }
                })
            }).collect::<Vec<_>>()
        }));

        // 逐个工具执行
        run_tools(&tx, pending_tool_calls, &mut messages).await?;
    }

    emit(&tx, StreamEvent::Completed).await
}

async fn run_tools(
    tx: &EventSender,
    tool_calls: Vec<ToolCallFinished>,
    messages: &mut Vec<Value>,
) -> anyhow::Result<()> {
    for call in tool_calls {
        emit(
            tx,
            StreamEvent::ToolRunStarted {
                index: call.index,
                call_id: call.call_id.clone(),
                name: call.name.clone(),
                arguments: call.arguments.clone(),
            },
        )
        .await?;

        // TODO: 权限检查？
        // 1 个工具没有权限，是后面的工具都不执行，还是跳过它先执行后面的工具？

        match run_tool(&call.name, &call.arguments) {
            Ok(output) => {
                emit(
                    tx,
                    StreamEvent::ToolRunFinished {
                        index: call.index,
                        call_id: call.call_id.clone(),
                        name: call.name.clone(),
                        output: output.clone(),
                    },
                )
                .await?;

                messages.push(json!({
                    "role": "tool",
                    "tool_call_id": call.call_id,
                    "content": output,
                }));
            }
            Err(e) => {
                let error = e.to_string();
                emit(
                    tx,
                    StreamEvent::ToolRunFailed {
                        index: call.index,
                        call_id: call.call_id.clone(),
                        name: call.name.clone(),
                        error: error.clone(),
                    },
                )
                .await?;

                messages.push(json!({
                    "role": "tool",
                    "tool_call_id": call.call_id,
                    "content": format!("tool failed: {error}")
                }));
            }
        }
    }

    Ok(())
}

async fn emit(tx: &EventSender, event: StreamEvent) -> anyhow::Result<()> {
    tx.send(Ok(event))
        .await
        .map_err(|e| anyhow::anyhow!("event receiver dropped {:?}", e))
}

#[cfg(test)]
mod tests {
    use crate::agent::{
        llm::openai::OpenAiCompatibleLlmBuilder,
        tool::{add_spec, sub_spec},
    };

    use super::*;

    #[tokio::test]
    #[ignore = "requires DEEPSEEK_API_KEY and network"]
    async fn react_agent_should_work() -> anyhow::Result<()> {
        let llm = OpenAiCompatibleLlmBuilder::default()
            .tools(vec![add_spec(), sub_spec()])
            .build()?;

        let agent = ReActAgent::new(Arc::new(llm));

        let mut stream = agent.run("计算一下999+666和321-123,将把它们俩的结果相加".to_string())?;

        while let Some(event) = stream.next().await {
            match event? {
                StreamEvent::Started => {
                    println!("start ----->")
                }
                StreamEvent::ThinkingDelta(s) => print!("{s}"),
                StreamEvent::TextDelta(s) => print!("{s}"),
                StreamEvent::ToolRunStarted {
                    index: _,
                    call_id,
                    name,
                    arguments,
                } => {
                    println!("\nstart to call tool [{call_id}: {name}]: {arguments}")
                }
                StreamEvent::ToolRunFinished {
                    index: _,
                    call_id,
                    name,
                    output,
                } => {
                    println!("\ntool call success [{call_id}: {name}], result: {output}")
                }
                StreamEvent::ToolRunFailed {
                    index: _,
                    call_id,
                    name,
                    error,
                } => {
                    println!("\ntool call failed [{call_id}: {name}], error: {error}")
                }
                StreamEvent::Error(e) => {
                    println!("stream occurs error: {e}")
                }
                StreamEvent::Completed => break,
                other => println!("{other:?}"),
            }
        }

        Ok(())
    }
}
