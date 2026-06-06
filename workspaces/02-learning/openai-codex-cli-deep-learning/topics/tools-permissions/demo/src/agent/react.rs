use std::sync::Arc;

use futures_util::{StreamExt, stream};
use serde_json::{Value, json};
use tokio::sync::mpsc;

use crate::{
    agent::{
        llm::{LLmRequest, Llm},
        stream_event::{EventStream, StreamEvent, ToolCallFinished},
    },
    tool::runtime::{ToolRuntime, ToolRuntimeResult},
};

pub type EventSender = mpsc::Sender<anyhow::Result<StreamEvent>>;

/// 最小 ReAct agent。
///
/// 它负责把用户 prompt 转成多轮 LLM 调用，并在模型请求工具时调用
/// `ToolRuntime`，再把 tool observation 回灌给下一轮模型。
pub struct ReActAgent {
    llm: Arc<dyn Llm + Send + Sync>,
    max_turns: u8,
    tool_runtime: Arc<ToolRuntime>,
}

const DEFAULT_MAX_TURNS: u8 = 32;

impl ReActAgent {
    /// 创建一个 ReAct agent。
    pub fn new(
        llm: Arc<dyn Llm + Send + Sync>,
        max_turns: Option<u8>,
        tool_runtime: Arc<ToolRuntime>,
    ) -> Self {
        Self {
            llm,
            max_turns: max_turns.unwrap_or(DEFAULT_MAX_TURNS),
            tool_runtime,
        }
    }

    /// 启动一次 agent run，并立即返回事件流。
    ///
    /// 实际 agent loop 在后台 task 中运行，通过 channel 逐步吐出事件。
    pub fn run(&self, prompt: String) -> anyhow::Result<EventStream> {
        if prompt.is_empty() {
            anyhow::bail!("prompt should not be empty");
        }

        let llm = self.llm.clone();
        let tool_runtime = self.tool_runtime.clone();
        let (tx, rx) = mpsc::channel(64);
        let max_turns = self.max_turns;

        tokio::spawn(async move {
            if let Err(err) =
                run_agent_loop(llm, prompt, max_turns, tx.clone(), &tool_runtime).await
            {
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
    max_turns: u8,
    tx: EventSender,
    tool_runtime: &ToolRuntime,
) -> anyhow::Result<()> {
    let mut messages = vec![
        json!({"role": "system", "content": ""}),
        json!({"role": "user", "content": prompt}),
    ];

    let mut run_turns = 0;

    loop {
        if run_turns >= max_turns {
            anyhow::bail!("max turns exceeded: {max_turns}");
        }
        run_turns += 1;

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
                StreamEvent::ToolCallFinished(tc) => {
                    pending_tool_calls.push(tc.clone());
                    emit(&tx, StreamEvent::ToolCallFinished(tc)).await?;
                }
                StreamEvent::Completed => break,
                StreamEvent::Error(err) => anyhow::bail!(err),
                other => emit(&tx, other).await?,
            }
        }

        if pending_tool_calls.is_empty() {
            break;
        }

        pending_tool_calls.sort_by_key(|call| call.index);

        // 将本轮 assistant 文本和 tool calls 放回消息历史，供下一轮模型参考。
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

        // 当前先按 index 顺序逐个执行工具。这样 observation 顺序稳定，便于测试。
        // TODO: 后续如果并发执行多个 tool，需要仍按原 index 回灌 messages。
        run_tools(&tx, pending_tool_calls, &mut messages, tool_runtime).await?;

        // TODO: 加入结构化 logger，记录每轮 LLM start/end、tool choice 和 observation。
    }

    emit(&tx, StreamEvent::Completed).await
}

async fn run_tools(
    tx: &EventSender,
    tool_calls: Vec<ToolCallFinished>,
    messages: &mut Vec<Value>,
    tool_runtime: &ToolRuntime,
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

        // TODO: 接入多 tool call 的 hard-deny 策略：
        // 一个工具被安全拒绝后，后续依赖它的工具应标记为 Skipped，而不是继续盲跑。

        match tool_runtime.run(&call, tx).await {
            ToolRuntimeResult::Finished { output } => {
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
            ToolRuntimeResult::Failed { error } => {
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
                    "content": error
                }));
            }
            ToolRuntimeResult::Denied { reason } => {
                emit(
                    tx,
                    StreamEvent::ToolRunFailed {
                        index: call.index,
                        call_id: call.call_id.clone(),
                        name: call.name.clone(),
                        error: reason.clone(),
                    },
                )
                .await?;

                messages.push(json!({
                    "role": "tool",
                    "tool_call_id": call.call_id,
                    "content": format!("tool denied: {reason}")
                }));
            }
            ToolRuntimeResult::Skipped { reason } => {
                emit(
                    tx,
                    StreamEvent::ToolRunFailed {
                        index: call.index,
                        call_id: call.call_id.clone(),
                        name: call.name.clone(),
                        error: reason.clone(),
                    },
                )
                .await?;

                messages.push(json!({
                    "role": "tool",
                    "tool_call_id": call.call_id,
                    "content": format!("tool skipped: {reason}")
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
    use std::{env::current_dir, sync::Arc};

    use crate::{
        agent::llm::{fake::FakeLlm, openai::OpenAiCompatibleLlmBuilder},
        model::{
            approval::{ApprovalPersistence, ApprovalPolicy},
            event::{ExecutionAttempt, RetryDecision, UserApprovalDecision},
        },
        tool::{
            function::{add_spec, sub_spec},
            runtime::{ToolRuntime, ToolRuntimeContext},
            shell::{
                approval::{ApprovalGateway, ToolApprovalResult},
                execution::simulated_execution_runner::SimulatedExecutionRunner,
                registry::CapabilityRegistry,
            },
        },
    };

    use super::*;

    struct TestAgent {
        agent: ReActAgent,
        approval_result_tx: mpsc::Sender<ToolApprovalResult>,
    }

    fn test_runtime() -> (Arc<ToolRuntime>, mpsc::Sender<ToolApprovalResult>) {
        let (approval_result_tx, approval_result_rx) = mpsc::channel(16);
        let runtime = Arc::new(ToolRuntime::new(ToolRuntimeContext {
            cwd: current_dir().unwrap_or_else(|_| ".".into()),
            approval_policy: ApprovalPolicy::OnFailure,
            execution_runner: Arc::new(SimulatedExecutionRunner::new()),
            registry: CapabilityRegistry::new(),
            approval_gateway: Arc::new(ApprovalGateway::new(approval_result_rx)),
        }));

        (runtime, approval_result_tx)
    }

    fn test_agent(llm: Arc<FakeLlm>, max_turns: Option<u8>) -> TestAgent {
        let (runtime, approval_result_tx) = test_runtime();
        TestAgent {
            agent: ReActAgent::new(llm, max_turns, runtime),
            approval_result_tx,
        }
    }

    #[tokio::test]
    #[ignore = "requires DEEPSEEK_API_KEY and network"]
    async fn react_agent_should_work() -> anyhow::Result<()> {
        let llm = OpenAiCompatibleLlmBuilder::default()
            .tools(vec![add_spec(), sub_spec()])
            .build()?;

        let (approval_result_tx, approval_result_rx) = mpsc::channel(16);
        let context = ToolRuntimeContext {
            cwd: current_dir().unwrap(),
            approval_policy: ApprovalPolicy::OnFailure,
            execution_runner: Arc::new(SimulatedExecutionRunner::new()),
            registry: CapabilityRegistry::new(),
            approval_gateway: Arc::new(ApprovalGateway::new(approval_result_rx)),
        };

        let agent = ReActAgent::new(Arc::new(llm), None, Arc::new(ToolRuntime::new(context)));

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
                StreamEvent::CommandNeedsApproval { approval_id, .. } => {
                    approval_result_tx
                        .send(ToolApprovalResult {
                            approval_id,
                            decision: UserApprovalDecision::Approved {
                                persistence: ApprovalPersistence::Once,
                            },
                        })
                        .await?;
                }
                StreamEvent::Completed => break,
                other => println!("{other:?}"),
            }
        }

        Ok(())
    }

    fn tool_call(index: i64, call_id: &str, name: &str, arguments: &str) -> StreamEvent {
        StreamEvent::ToolCallFinished(ToolCallFinished {
            index,
            call_id: call_id.to_string(),
            name: name.to_string(),
            arguments: arguments.to_string(),
        })
    }

    async fn collect_events(agent: &TestAgent, prompt: &str) -> anyhow::Result<Vec<StreamEvent>> {
        let mut stream = agent.agent.run(prompt.to_string())?;
        let mut events = vec![];

        while let Some(event) = stream.next().await {
            let event = event?;
            if let StreamEvent::CommandNeedsApproval { approval_id, .. } = &event {
                agent
                    .approval_result_tx
                    .send(ToolApprovalResult {
                        approval_id: approval_id.clone(),
                        decision: UserApprovalDecision::Approved {
                            persistence: ApprovalPersistence::Once,
                        },
                    })
                    .await?;
            }
            events.push(event);
        }

        Ok(events)
    }

    async fn collect_until_error(
        agent: &TestAgent,
        prompt: &str,
    ) -> anyhow::Result<(Vec<StreamEvent>, String)> {
        let mut stream = agent.agent.run(prompt.to_string())?;
        let mut events = vec![];

        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => {
                    if let StreamEvent::CommandNeedsApproval { approval_id, .. } = &event {
                        agent
                            .approval_result_tx
                            .send(ToolApprovalResult {
                                approval_id: approval_id.clone(),
                                decision: UserApprovalDecision::Approved {
                                    persistence: ApprovalPersistence::Once,
                                },
                            })
                            .await?;
                    }
                    events.push(event)
                }
                Err(err) => return Ok((events, err.to_string())),
            }
        }

        anyhow::bail!("expected stream error, but stream completed")
    }

    fn event_position(events: &[StreamEvent], predicate: impl Fn(&StreamEvent) -> bool) -> usize {
        events
            .iter()
            .position(predicate)
            .expect("expected event to be present")
    }

    fn assert_in_order(positions: &[usize]) {
        assert!(positions.windows(2).all(|window| window[0] < window[1]));
    }

    fn assert_second_turn_tool_observation(llm: &FakeLlm, tool_call_id: &str) {
        let requests = llm.requests();
        assert_eq!(requests.len(), 2);
        let observation = &requests[1][3];
        assert_eq!(observation["role"], "tool");
        assert_eq!(observation["tool_call_id"], tool_call_id);
        assert!(observation["content"].is_string());
    }

    #[tokio::test]
    async fn react_agent_should_return_final_answer_without_tool_call() -> anyhow::Result<()> {
        let llm = FakeLlm::new(vec![vec![
            StreamEvent::Started,
            StreamEvent::TextDelta("final answer".to_string()),
            StreamEvent::Completed,
        ]]);
        let agent = test_agent(llm.clone(), Some(3));

        let events = collect_events(&agent, "hello").await?;

        assert!(matches!(events[0], StreamEvent::Started));
        assert!(matches!(&events[1], StreamEvent::TextDelta(text) if text == "final answer"));
        assert!(matches!(events[2], StreamEvent::Completed));
        assert_eq!(events.len(), 3);
        assert_eq!(llm.requests().len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn react_agent_should_run_multiple_tool_calls_in_index_order() -> anyhow::Result<()> {
        let llm = FakeLlm::new(vec![
            vec![
                StreamEvent::Started,
                tool_call(1, "call_sub", "sub", r#"{"a": 5, "b": 2}"#),
                tool_call(0, "call_add", "add", r#"{"a": 1, "b": 2}"#),
                StreamEvent::Completed,
            ],
            vec![
                StreamEvent::TextDelta("final answer".to_string()),
                StreamEvent::Completed,
            ],
        ]);
        let agent = test_agent(llm.clone(), Some(3));

        let events = collect_events(&agent, "calculate").await?;

        let started_indexes = events
            .iter()
            .filter_map(|event| match event {
                StreamEvent::ToolRunStarted { index, .. } => Some(*index),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(started_indexes, vec![0, 1]);

        let outputs = events
            .iter()
            .filter_map(|event| match event {
                StreamEvent::ToolRunFinished { output, .. } => Some(output.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(outputs, vec!["3", "3"]);

        let requests = llm.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[1][2]["tool_calls"][0]["id"], "call_add");
        assert_eq!(requests[1][2]["tool_calls"][1]["id"], "call_sub");

        Ok(())
    }

    #[tokio::test]
    async fn react_agent_should_continue_after_multiple_tool_rounds() -> anyhow::Result<()> {
        let llm = FakeLlm::new(vec![
            vec![
                tool_call(0, "call_add_1", "add", r#"{"a": 1, "b": 2}"#),
                StreamEvent::Completed,
            ],
            vec![
                tool_call(0, "call_add_2", "add", r#"{"a": 3, "b": 4}"#),
                StreamEvent::Completed,
            ],
            vec![
                StreamEvent::TextDelta("done".to_string()),
                StreamEvent::Completed,
            ],
        ]);
        let agent = test_agent(llm.clone(), Some(3));

        let events = collect_events(&agent, "calculate").await?;

        let outputs = events
            .iter()
            .filter_map(|event| match event {
                StreamEvent::ToolRunFinished { output, .. } => Some(output.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(outputs, vec!["3", "7"]);
        assert!(matches!(events.last(), Some(StreamEvent::Completed)));
        assert_eq!(llm.requests().len(), 3);

        Ok(())
    }

    #[tokio::test]
    async fn react_agent_should_feed_tool_failure_back_as_observation() -> anyhow::Result<()> {
        let llm = FakeLlm::new(vec![
            vec![
                tool_call(0, "call_mul", "mul", r#"{"a": 2, "b": 3}"#),
                StreamEvent::Completed,
            ],
            vec![
                StreamEvent::TextDelta("handled failure".to_string()),
                StreamEvent::Completed,
            ],
        ]);
        let agent = test_agent(llm.clone(), Some(3));

        let events = collect_events(&agent, "calculate").await?;

        assert!(events.iter().any(|event| matches!(
            event,
            StreamEvent::ToolRunFailed { name, .. } if name == "mul"
        )));

        let requests = llm.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[1][3]["role"], "tool");
        assert_eq!(requests[1][3]["tool_call_id"], "call_mul");
        assert!(requests[1][3]["content"].is_string());

        Ok(())
    }

    #[tokio::test]
    async fn react_agent_should_feed_run_command_success_back_as_observation() -> anyhow::Result<()>
    {
        let llm = FakeLlm::new(vec![
            vec![
                tool_call(
                    0,
                    "call_read",
                    "run_command",
                    r#"{"command": "cat package.json", "justification": "read package metadata"}"#,
                ),
                StreamEvent::Completed,
            ],
            vec![
                StreamEvent::TextDelta("read command observed".to_string()),
                StreamEvent::Completed,
            ],
        ]);
        let agent = test_agent(llm.clone(), Some(3));

        let events = collect_events(&agent, "read package metadata").await?;

        assert!(events.iter().any(|event| matches!(
            event,
            StreamEvent::ToolRunFinished { name, .. } if name == "run_command"
        )));
        assert_second_turn_tool_observation(&llm, "call_read");

        Ok(())
    }

    #[tokio::test]
    async fn react_agent_should_stream_command_execution_events_before_tool_observation()
    -> anyhow::Result<()> {
        let llm = FakeLlm::new(vec![
            vec![
                tool_call(
                    0,
                    "call_read",
                    "run_command",
                    r#"{"command": "cat package.json", "justification": "read package metadata"}"#,
                ),
                StreamEvent::Completed,
            ],
            vec![
                StreamEvent::TextDelta("read command observed".to_string()),
                StreamEvent::Completed,
            ],
        ]);
        let agent = test_agent(llm, Some(3));

        let events = collect_events(&agent, "read package metadata").await?;

        let tool_started = event_position(&events, |event| {
            matches!(
                event,
                StreamEvent::ToolRunStarted {
                    call_id,
                    name,
                    ..
                } if call_id == "call_read" && name == "run_command"
            )
        });
        let command_started = event_position(&events, |event| {
            matches!(
                event,
                StreamEvent::CommandExecutionStarted {
                    call_id,
                    name,
                    attempt: ExecutionAttempt::SandboxFirst { .. },
                    ..
                } if call_id == "call_read" && name == "run_command"
            )
        });
        let command_finished = event_position(&events, |event| {
            matches!(
                event,
                StreamEvent::CommandExecutionFinished {
                    call_id,
                    name,
                    attempt: ExecutionAttempt::SandboxFirst { .. },
                    ..
                } if call_id == "call_read" && name == "run_command"
            )
        });
        let tool_finished = event_position(&events, |event| {
            matches!(
                event,
                StreamEvent::ToolRunFinished {
                    call_id,
                    name,
                    ..
                } if call_id == "call_read" && name == "run_command"
            )
        });

        assert_in_order(&[
            tool_started,
            command_started,
            command_finished,
            tool_finished,
        ]);

        Ok(())
    }

    #[tokio::test]
    async fn react_agent_should_feed_run_command_failure_back_as_observation() -> anyhow::Result<()>
    {
        let llm = FakeLlm::new(vec![
            vec![
                tool_call(
                    0,
                    "call_test",
                    "run_command",
                    r#"{"command": "npm test -- fail", "justification": "run failing test"}"#,
                ),
                StreamEvent::Completed,
            ],
            vec![
                StreamEvent::TextDelta("failure observed".to_string()),
                StreamEvent::Completed,
            ],
        ]);
        let agent = test_agent(llm.clone(), Some(3));

        let events = collect_events(&agent, "run tests").await?;

        assert!(events.iter().any(|event| matches!(
            event,
            StreamEvent::ToolRunFailed { name, .. } if name == "run_command"
        )));

        assert_second_turn_tool_observation(&llm, "call_test");

        Ok(())
    }

    #[tokio::test]
    async fn react_agent_should_stream_retry_decision_and_retry_execution_events()
    -> anyhow::Result<()> {
        let llm = FakeLlm::new(vec![
            vec![
                tool_call(
                    0,
                    "call_install",
                    "run_command",
                    r#"{"command": "npm install vite", "justification": "install dependency"}"#,
                ),
                StreamEvent::Completed,
            ],
            vec![
                StreamEvent::TextDelta("install observed".to_string()),
                StreamEvent::Completed,
            ],
        ]);
        let agent = test_agent(llm, Some(3));

        let events = collect_events(&agent, "install dependency").await?;

        let sandbox_started = event_position(&events, |event| {
            matches!(
                event,
                StreamEvent::CommandExecutionStarted {
                    call_id,
                    attempt: ExecutionAttempt::SandboxFirst { .. },
                    ..
                } if call_id == "call_install"
            )
        });
        let retry_evaluated = event_position(&events, |event| {
            matches!(
                event,
                StreamEvent::CommandRetryEvaluated {
                    call_id,
                    decision: RetryDecision::RetryWithApproval { .. },
                    ..
                } if call_id == "call_install"
            )
        });
        let sandbox_failed = event_position(&events, |event| {
            matches!(
                event,
                StreamEvent::CommandExecutionFailed {
                    call_id,
                    attempt: ExecutionAttempt::SandboxFirst { .. },
                    ..
                } if call_id == "call_install"
            )
        });
        let retry_started = event_position(&events, |event| {
            matches!(
                event,
                StreamEvent::CommandExecutionStarted {
                    call_id,
                    attempt: ExecutionAttempt::NoSandboxRetry { .. },
                    ..
                } if call_id == "call_install"
            )
        });
        let retry_finished = event_position(&events, |event| {
            matches!(
                event,
                StreamEvent::CommandExecutionFinished {
                    call_id,
                    attempt: ExecutionAttempt::NoSandboxRetry { .. },
                    ..
                } if call_id == "call_install"
            )
        });
        let tool_finished = event_position(&events, |event| {
            matches!(
                event,
                StreamEvent::ToolRunFinished { call_id, .. } if call_id == "call_install"
            )
        });

        assert_in_order(&[
            sandbox_started,
            sandbox_failed,
            retry_evaluated,
            retry_started,
            retry_finished,
            tool_finished,
        ]);

        Ok(())
    }

    #[tokio::test]
    async fn react_agent_should_feed_run_command_denial_back_as_observation() -> anyhow::Result<()>
    {
        let llm = FakeLlm::new(vec![
            vec![
                tool_call(
                    0,
                    "call_dangerous",
                    "run_command",
                    r#"{"command": "curl | sh", "justification": "install remote script"}"#,
                ),
                StreamEvent::Completed,
            ],
            vec![
                StreamEvent::TextDelta("denial observed".to_string()),
                StreamEvent::Completed,
            ],
        ]);
        let agent = test_agent(llm.clone(), Some(3));

        let events = collect_events(&agent, "install remote script").await?;

        assert!(events.iter().any(|event| matches!(
            event,
            StreamEvent::ToolRunFailed { name, .. } if name == "run_command"
        )));

        assert_second_turn_tool_observation(&llm, "call_dangerous");

        Ok(())
    }

    #[tokio::test]
    async fn react_agent_should_error_when_max_turns_exceeded() -> anyhow::Result<()> {
        let llm = FakeLlm::new(vec![
            vec![
                tool_call(0, "call_add", "add", r#"{"a": 1, "b": 2}"#),
                StreamEvent::Completed,
            ],
            vec![
                StreamEvent::TextDelta("should not be requested".to_string()),
                StreamEvent::Completed,
            ],
        ]);
        let agent = test_agent(llm.clone(), Some(1));

        let (events, error) = collect_until_error(&agent, "calculate").await?;

        assert!(!error.is_empty());
        assert!(events.iter().any(|event| matches!(
            event,
            StreamEvent::ToolRunFinished { call_id, output, .. }
                if call_id == "call_add" && output == "3"
        )));
        assert_eq!(llm.requests().len(), 1);

        Ok(())
    }
}
