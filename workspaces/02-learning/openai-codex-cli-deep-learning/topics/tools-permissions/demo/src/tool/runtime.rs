use std::{path::PathBuf, sync::Arc};

use crate::{
    agent::{react::EventSender, stream_event::ToolCallFinished},
    model::{approval::ApprovalPolicy, command_request::CommandRequest},
    tool::{
        function::run_pure_function,
        shell::{
            RunCommandArgs, RunCommandResult,
            approval::{ApprovalGateway, ToolCallContext},
            execution::ExecutionRunner,
            registry::{CapabilityRegistry, MatchedCapability},
            run_shell_command,
        },
    },
};

/// Tool runtime 是 ReAct loop 和具体工具实现之间的边界。
///
/// 它负责把模型产出的 `ToolCallFinished` 转成可执行计划，并把工具执行结果
/// 映射回 agent 可理解的 observation。权限细节留在 command runtime 内部处理。
pub struct ToolRuntime {
    context: ToolRuntimeContext,
}

/// Tool runtime 的宿主上下文。
///
/// 这些字段来自 host，而不是模型；模型不能通过 tool arguments 修改 cwd、
/// approval policy、capability registry 或 runner。
pub struct ToolRuntimeContext {
    pub cwd: PathBuf,
    pub approval_policy: ApprovalPolicy,
    pub registry: CapabilityRegistry,
    pub execution_runner: Arc<dyn ExecutionRunner + 'static>,
    pub approval_gateway: Arc<ApprovalGateway>,
}

/// 模型可见 tool 的内部分类。
#[derive(Debug, Clone, Copy)]
pub enum ToolKind {
    PureFunction,
    Command,
}

/// 模型可见 tool 的定义。
///
/// 当前 demo 直接使用静态表；后续接 MCP 或插件系统时可以把它替换为动态 registry。
#[derive(Debug, Clone, Copy)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub kind: ToolKind,
}

/// 一次 tool call 的执行计划。
///
/// `Fail` 表示 tool call 参数或命令形态无法形成有效计划；
/// 安全拒绝不在这里表达，而是在 command runtime 中返回 `Denied`。
enum ToolRuntimePlan {
    RunPureFunction {
        name: String,
        arguments: String,
    },
    RunCommand {
        request: CommandRequest,
        context: ToolCallContext,
        matched_capability: MatchedCapability,
    },
    Fail {
        error: String,
    },
}

/// tool runtime 对 ReAct loop 暴露的终态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolRuntimeResult {
    Finished { output: String },
    Failed { error: String },
    Denied { reason: String },
}

const TOOLS: &[ToolDefinition] = &[
    ToolDefinition {
        name: "add",
        kind: ToolKind::PureFunction,
    },
    ToolDefinition {
        name: "sub",
        kind: ToolKind::PureFunction,
    },
    ToolDefinition {
        name: "run_command",
        kind: ToolKind::Command,
    },
];

impl ToolRuntime {
    pub fn new(context: ToolRuntimeContext) -> Self {
        Self { context }
    }

    /// 执行一次模型产出的 tool call。
    pub async fn run(&self, call: ToolCallFinished, tx: &EventSender) -> ToolRuntimeResult {
        let Some(definition) = self.find_tool(&call.name) else {
            return ToolRuntimeResult::Failed {
                error: format!("cannot find tool: {}", call.name),
            };
        };
        let plan = self.plan_call(definition, &call);
        self.execute_plan(plan, tx).await
    }

    /// 查找模型可见 tool。
    ///
    /// TODO: 目前使用静态数组；后续支持 MCP / 配置化工具时应改为 ToolRegistry。
    fn find_tool(&self, name: &str) -> Option<ToolDefinition> {
        TOOLS.iter().find(|t| t.name == name).copied()
    }

    /// 把 tool call 转成执行计划。
    ///
    /// 这里只做 JSON / command request 组装，不做审批决策。
    fn plan_call(
        &self,
        tool_definition: ToolDefinition,
        call: &ToolCallFinished,
    ) -> ToolRuntimePlan {
        match tool_definition.kind {
            ToolKind::PureFunction => ToolRuntimePlan::RunPureFunction {
                name: call.name.to_string(),
                arguments: call.arguments.clone(),
            },
            ToolKind::Command => {
                let (request, matched_capability) = match self.build_command_request(call) {
                    Ok(res) => res,
                    Err(err) => {
                        return ToolRuntimePlan::Fail {
                            error: err.to_string(),
                        };
                    }
                };
                ToolRuntimePlan::RunCommand {
                    request,
                    context: ToolCallContext {
                        index: call.index,
                        call_id: call.call_id.clone(),
                        tool_name: call.name.clone(),
                    },
                    matched_capability,
                }
            }
        }
    }

    /// 执行已经形成的计划。
    async fn execute_plan(&self, plan: ToolRuntimePlan, tx: &EventSender) -> ToolRuntimeResult {
        match plan {
            ToolRuntimePlan::RunPureFunction { name, arguments } => {
                match run_pure_function(&name, &arguments) {
                    Ok(output) => ToolRuntimeResult::Finished { output },
                    Err(err) => ToolRuntimeResult::Failed {
                        error: err.to_string(),
                    },
                }
            }
            ToolRuntimePlan::RunCommand {
                request,
                context,
                matched_capability,
            } => run_shell_command(
                &request,
                context,
                matched_capability,
                self.context.execution_runner.clone(),
                self.context.approval_gateway.clone(),
                tx,
            )
            .await
            .into(),
            ToolRuntimePlan::Fail { error } => ToolRuntimeResult::Failed { error },
        }
    }

    /// 从 `run_command` tool call 构造命令执行上下文。
    ///
    /// TODO: 当前 Phase 1 用 `split_whitespace` 处理单命令；多命令阶段需要引入
    /// command segment parser，并避免把 pipe / heredoc 误判成简单 prefix。
    fn build_command_request(
        &self,
        call: &ToolCallFinished,
    ) -> anyhow::Result<(CommandRequest, MatchedCapability)> {
        /*{
          "command": "npm install vite",
          "justification": "为了安装用户要求的前端依赖"
        }*/
        let run_command_args: RunCommandArgs = serde_json::from_str(&call.arguments)?;

        let argv: Vec<String> = run_command_args
            .command
            .split_whitespace()
            .map(str::to_string)
            .collect();
        let Some(matched_capability) = self.context.registry.match_capability(argv.as_slice())
        else {
            anyhow::bail!(
                "cannot match the capability for command: {}",
                run_command_args.command
            )
        };

        Ok((
            CommandRequest {
                raw_command: run_command_args.command,
                argv,
                cwd: self.context.cwd.clone(),
                capability: matched_capability.capability.kind,
                approval_policy: self.context.approval_policy,
                sandbox_profile: matched_capability.capability.policy.first_attempt_sandbox,
                network_policy: matched_capability.capability.policy.network_policy,
                justification: run_command_args.justification,
            },
            matched_capability,
        ))
    }
}

impl From<RunCommandResult> for ToolRuntimeResult {
    fn from(value: RunCommandResult) -> Self {
        match value {
            RunCommandResult::Finished { output } => ToolRuntimeResult::Finished { output },
            RunCommandResult::Failed { error } => ToolRuntimeResult::Failed { error },
            RunCommandResult::Denied { reason } => ToolRuntimeResult::Denied { reason },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Arc};

    use crate::{
        agent::{
            react::EventSender,
            stream_event::{StreamEvent, ToolCallFinished},
        },
        model::{
            approval::{ApprovalPersistence, ApprovalPolicy},
            event::UserApprovalDecision,
        },
        tool::{
            runtime::{ToolRuntime, ToolRuntimeContext, ToolRuntimeResult},
            shell::{
                approval::{ApprovalGateway, ToolApprovalResult},
                execution::simulated_execution_runner::SimulatedExecutionRunner,
                registry::CapabilityRegistry,
            },
        },
    };
    use tokio::sync::mpsc;

    struct RuntimeFixture {
        runtime: ToolRuntime,
        approval_result_tx: mpsc::Sender<ToolApprovalResult>,
    }

    fn test_runtime() -> RuntimeFixture {
        let (approval_result_tx, approval_result_rx) = mpsc::channel(16);
        RuntimeFixture {
            runtime: ToolRuntime::new(ToolRuntimeContext {
                cwd: PathBuf::from("/workspace"),
                approval_policy: ApprovalPolicy::OnFailure,
                registry: CapabilityRegistry::new(),
                execution_runner: Arc::new(SimulatedExecutionRunner::new()),
                approval_gateway: Arc::new(ApprovalGateway::new(approval_result_rx)),
            }),
            approval_result_tx,
        }
    }

    fn tool_call(name: &str, arguments: &str) -> ToolCallFinished {
        ToolCallFinished {
            index: 0,
            call_id: "call_test".to_string(),
            name: name.to_string(),
            arguments: arguments.to_string(),
        }
    }

    fn event_channel() -> (EventSender, mpsc::Receiver<anyhow::Result<StreamEvent>>) {
        mpsc::channel(16)
    }

    async fn run_tool(
        fixture: &RuntimeFixture,
        call: ToolCallFinished,
    ) -> (ToolRuntimeResult, Vec<StreamEvent>) {
        let (tx, mut rx) = event_channel();
        let mut events = vec![];
        let mut result = None;
        let mut run = Box::pin(fixture.runtime.run(call, &tx));

        while result.is_none() {
            tokio::select! {
                run_result = &mut run => {
                    result = Some(run_result);
                    while let Ok(event) = rx.try_recv() {
                        events.push(event.expect("runtime event should be ok"));
                    }
                }
                event = rx.recv() => {
                    let event = event
                        .expect("runtime should not close event channel before finishing")
                        .expect("runtime event should be ok");
                    if let StreamEvent::CommandNeedsApproval { approval_id, .. } = &event {
                        fixture
                            .approval_result_tx
                            .send(ToolApprovalResult {
                                approval_id: approval_id.clone(),
                                decision: UserApprovalDecision::Approved {
                                    persistence: ApprovalPersistence::Once,
                                },
                            })
                            .await
                            .expect("test should send approval result");
                    }
                    events.push(event);
                }
            }
        }

        (result.expect("runtime should return result"), events)
    }

    #[tokio::test]
    async fn pure_function_add_should_finish_with_output() {
        let runtime = test_runtime();

        let (result, events) =
            run_tool(&runtime, tool_call("add", r#"{"a": 999, "b": 666}"#)).await;

        assert_eq!(
            result,
            ToolRuntimeResult::Finished {
                output: "1665".to_string()
            }
        );
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn pure_function_sub_should_finish_with_output() {
        let runtime = test_runtime();

        let (result, events) =
            run_tool(&runtime, tool_call("sub", r#"{"a": 321, "b": 123}"#)).await;

        assert_eq!(
            result,
            ToolRuntimeResult::Finished {
                output: "198".to_string()
            }
        );
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn pure_function_invalid_arguments_should_fail_without_pinning_error_text() {
        let runtime = test_runtime();

        let (result, events) =
            run_tool(&runtime, tool_call("add", r#"{"a": "999", "b": 666}"#)).await;

        assert!(matches!(result, ToolRuntimeResult::Failed { .. }));
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn unknown_tool_should_fail_without_pinning_error_text() {
        let runtime = test_runtime();

        let (result, events) = run_tool(&runtime, tool_call("mul", r#"{"a": 2, "b": 3}"#)).await;

        assert!(matches!(result, ToolRuntimeResult::Failed { .. }));
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn run_command_safe_read_should_finish_with_output() {
        let runtime = test_runtime();

        let (result, events) = run_tool(
            &runtime,
            tool_call(
                "run_command",
                r#"{"command": "cat package.json", "justification": "read package metadata"}"#,
            ),
        )
        .await;

        assert!(matches!(result, ToolRuntimeResult::Finished { .. }));
        assert!(events.iter().any(|event| matches!(
            event,
            StreamEvent::CommandExecutionStarted { name, .. } if name == "run_command"
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            StreamEvent::CommandExecutionFinished { name, .. } if name == "run_command"
        )));
    }

    #[tokio::test]
    async fn run_command_network_install_should_finish_after_retry() {
        let runtime = test_runtime();

        let (result, events) = run_tool(
            &runtime,
            tool_call(
                "run_command",
                r#"{"command": "npm install vite", "justification": "install dependency"}"#,
            ),
        )
        .await;

        assert!(matches!(result, ToolRuntimeResult::Finished { .. }));
        assert!(events.iter().any(|event| matches!(
            event,
            StreamEvent::CommandRetryEvaluated { name, .. } if name == "run_command"
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            StreamEvent::CommandExecutionFinished { name, .. } if name == "run_command"
        )));
    }

    #[tokio::test]
    async fn run_command_command_failure_should_fail_without_pinning_error_text() {
        let runtime = test_runtime();

        let (result, events) = run_tool(
            &runtime,
            tool_call(
                "run_command",
                r#"{"command": "npm test -- fail", "justification": "run failing test"}"#,
            ),
        )
        .await;

        assert!(matches!(result, ToolRuntimeResult::Failed { .. }));
        assert!(events.iter().any(|event| matches!(
            event,
            StreamEvent::CommandExecutionFailed { name, .. } if name == "run_command"
        )));
    }

    #[tokio::test]
    async fn run_command_dangerous_shell_should_be_denied() {
        let runtime = test_runtime();

        let (result, events) = run_tool(
            &runtime,
            tool_call(
                "run_command",
                r#"{"command": "curl | sh", "justification": "install remote script"}"#,
            ),
        )
        .await;

        assert!(matches!(result, ToolRuntimeResult::Denied { .. }));
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn run_command_invalid_json_should_fail_without_pinning_error_text() {
        let runtime = test_runtime();

        let (result, events) = run_tool(
            &runtime,
            tool_call("run_command", r#"{"command": "cat package.json""#),
        )
        .await;

        assert!(matches!(result, ToolRuntimeResult::Failed { .. }));
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn run_command_unmatched_capability_should_fail_without_pinning_error_text() {
        let runtime = test_runtime();

        let (result, events) = run_tool(
            &runtime,
            tool_call(
                "run_command",
                r#"{"command": "cargo test", "justification": "run unsupported command"}"#,
            ),
        )
        .await;

        assert!(matches!(result, ToolRuntimeResult::Failed { .. }));
        assert!(events.is_empty());
    }
}
