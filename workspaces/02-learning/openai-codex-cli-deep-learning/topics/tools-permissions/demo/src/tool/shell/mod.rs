use std::sync::Arc;

use crate::{
    agent::{react::EventSender, stream_event::StreamEvent},
    model::{
        approval::ApprovalRequirement,
        command_request::CommandRequest,
        event::{ExecutionAttempt, RetryDecision, UserApprovalDecision},
        execution::{ExecutionFailure, ExecutionResult},
    },
    tool::shell::{
        approval::{
            ApprovalGateway, ToolApprovalRequest, ToolCallContext, build_approval_scope,
            resolve_approval_requirement,
        },
        execution::ExecutionRunner,
        registry::MatchedCapability,
        retry::decide_retry,
    },
};

pub mod approval;
pub mod execution;
pub mod registry;
mod retry;

/// `run_command` tool 的参数结构。
///
/// 模型只能提供命令文本和解释文本；cwd、capability、sandbox、network policy
/// 都由 host 侧的 `ToolRuntime` 组装。
#[derive(Debug, serde::Deserialize)]
pub struct RunCommandArgs {
    pub command: String,
    #[serde(default)]
    pub justification: Option<String>,
}

/// shell command runtime 的终态。
///
/// Approval / sandbox / retry 都在 `run_shell_command` 内部被消费并折叠为这三个结果。
pub enum RunCommandResult {
    Finished { output: String },
    Failed { error: String },
    Denied { reason: String },
}

/// 执行 shell 命令。
///
/// 当前目标是先跑通单命令主链路：
/// approval gate -> sandbox first -> retry gate -> optional no-sandbox retry。
pub async fn run_shell_command(
    request: &CommandRequest,
    context: ToolCallContext,
    matched_capability: MatchedCapability,
    execution_runner: Arc<dyn ExecutionRunner + Send + Sync + 'static>,
    approval_gateway: Arc<ApprovalGateway>,
    tx: &EventSender,
) -> RunCommandResult {
    match resolve_approval_requirement(&request, &matched_capability) {
        ApprovalRequirement::Skip {
            bypass_sandbox,
            reason,
        } => {
            // 不需要审批但是需要先走一遍沙箱
            if !bypass_sandbox {
                return run_sandbox_first_flow(
                    request,
                    context,
                    matched_capability,
                    execution_runner,
                    approval_gateway,
                    tx,
                )
                .await;
            }

            // 不需要审批且跳过沙箱，直接在本地运行
            return run_no_sandbox_first(request, &context, execution_runner.as_ref(), reason, tx)
                .await;
        }
        ApprovalRequirement::NeedsApproval {
            reason,
            approval_scope,
        } => match request_approval(
            &approval_gateway,
            ToolApprovalRequest {
                context: context.clone(),
                reason: reason.clone(),
                scope: approval_scope,
            },
            tx,
        )
        .await
        {
            UserApprovalDecision::Approved { persistence: _ } => {
                return run_sandbox_first_flow(
                    request,
                    context,
                    matched_capability,
                    execution_runner,
                    approval_gateway,
                    tx,
                )
                .await;
            }
            UserApprovalDecision::Rejected => {
                return RunCommandResult::Denied { reason };
            }
        },
        ApprovalRequirement::Forbidden { reason } => RunCommandResult::Denied { reason },
    }
}

async fn request_approval(
    approval_gateway: &Arc<ApprovalGateway>,
    request: ToolApprovalRequest,
    tx: &EventSender,
) -> UserApprovalDecision {
    let pending = approval_gateway.create_pending_approval(request.clone());
    let approval_id = pending.approval_id.clone();

    let decision = match tx
        .send(Ok(StreamEvent::CommandNeedsApproval {
            approval_id: approval_id.clone(),
            index: request.context.index,
            call_id: request.context.call_id,
            name: request.context.tool_name,
            reason: request.reason,
            scope: request.scope,
        }))
        .await
    {
        Ok(_) => pending.wait().await,
        Err(_) => UserApprovalDecision::Rejected,
    };

    approval_gateway.cancel(&approval_id);

    decision
}

async fn run_sandbox_first_flow(
    request: &CommandRequest,
    context: ToolCallContext,
    matched_capability: MatchedCapability,
    execution_runner: Arc<dyn ExecutionRunner + Send + Sync + 'static>,
    approval_gateway: Arc<ApprovalGateway>,
    tx: &EventSender,
) -> RunCommandResult {
    let attempt = ExecutionAttempt::SandboxFirst {
        sandbox_profile: request.sandbox_profile,
    };

    let _ = tx
        .send(Ok(StreamEvent::CommandExecutionStarted {
            index: context.index,
            call_id: context.call_id.clone(),
            name: context.tool_name.clone(),
            attempt: attempt.clone(),
        }))
        .await;

    // 先在沙箱尝试执行
    let sandbox_first_execution_result = execution_runner.run(&request, &attempt);
    match sandbox_first_execution_result {
        // 沙箱成功直接返回
        ExecutionResult::Success { stdout } => {
            let _ = tx
                .send(Ok(StreamEvent::CommandExecutionFinished {
                    index: context.index,
                    call_id: context.call_id.clone(),
                    name: context.tool_name.clone(),
                    attempt: attempt.clone(),
                    output: stdout.clone(),
                }))
                .await;
            return RunCommandResult::Finished { output: stdout };
        }

        // 沙箱失败则判断是否要进行重试
        ExecutionResult::Failure(failure) => {
            _ = tx
                .send(Ok(StreamEvent::CommandExecutionFailed {
                    index: context.index,
                    call_id: context.call_id.clone(),
                    name: context.tool_name.clone(),
                    attempt: attempt.clone(),
                    error: format!("failed: {:?}", failure),
                }))
                .await;

            let retry_decision = decide_retry(
                request,
                &failure,
                &build_approval_scope(request, &matched_capability),
                matched_capability.capability.policy.retry_policy,
                false,
            );

            _ = tx
                .send(Ok(StreamEvent::CommandRetryEvaluated {
                    index: context.index,
                    call_id: context.call_id.clone(),
                    name: context.tool_name.clone(),
                    decision: retry_decision.clone(),
                }))
                .await;

            match retry_decision {
                // 不重试直接返回失败
                RetryDecision::DoNotRetry { reason: _ } => {
                    return failure.into();
                }
                // 重试且不需要审批则直接运行
                RetryDecision::RetryWithoutApproval { reason } => {
                    return run_no_sandbox_retry(
                        request,
                        &context,
                        execution_runner.as_ref(),
                        reason,
                        tx,
                    )
                    .await;
                }
                // 重新且需要审批则先走审批
                RetryDecision::RetryWithApproval {
                    reason,
                    approval_scope,
                } => match request_approval(
                    &approval_gateway,
                    ToolApprovalRequest {
                        context: context.clone(),
                        reason: reason.clone(),
                        scope: approval_scope,
                    },
                    tx,
                )
                .await
                {
                    // 审批通过则直接本机运行再次尝试
                    UserApprovalDecision::Approved { persistence: _ } => {
                        return run_no_sandbox_retry(
                            request,
                            &context,
                            execution_runner.as_ref(),
                            reason,
                            tx,
                        )
                        .await;
                    }
                    // 审批不通过则直接拒绝
                    UserApprovalDecision::Rejected => {
                        return RunCommandResult::Denied {
                            reason: format!(
                                "failed {:?}, and disapprove retry: {}",
                                failure, reason
                            ),
                        };
                    }
                },
            }
        }
    }
}

async fn run_no_sandbox_first(
    request: &CommandRequest,
    context: &ToolCallContext,
    execution_runner: &dyn ExecutionRunner,
    reason: String,
    tx: &EventSender,
) -> RunCommandResult {
    let attempt = ExecutionAttempt::NoSandboxFirst { reason };

    let _ = tx
        .send(Ok(StreamEvent::CommandExecutionStarted {
            index: context.index,
            call_id: context.call_id.clone(),
            name: context.tool_name.clone(),
            attempt: attempt.clone(),
        }))
        .await;

    let result = execution_runner.run(request, &attempt);
    handle_execution_result(context, attempt, result, tx).await
}

async fn run_no_sandbox_retry(
    request: &CommandRequest,
    context: &ToolCallContext,
    execution_runner: &dyn ExecutionRunner,
    reason: String,
    tx: &EventSender,
) -> RunCommandResult {
    let attempt = ExecutionAttempt::NoSandboxRetry { reason };

    let _ = tx
        .send(Ok(StreamEvent::CommandExecutionStarted {
            index: context.index,
            call_id: context.call_id.clone(),
            name: context.tool_name.clone(),
            attempt: attempt.clone(),
        }))
        .await;

    let result = execution_runner.run(request, &attempt);
    handle_execution_result(context, attempt, result, tx).await
}

/// 处理 ExecutionResult:
///  1. 向外面透露执行事件
///  2. 转为 RunCommandResult
async fn handle_execution_result(
    context: &ToolCallContext,
    attempt: ExecutionAttempt,
    result: ExecutionResult,
    tx: &EventSender,
) -> RunCommandResult {
    match &result {
        ExecutionResult::Success { stdout } => {
            let _ = tx
                .send(Ok(StreamEvent::CommandExecutionFinished {
                    index: context.index,
                    call_id: context.call_id.clone(),
                    name: context.tool_name.clone(),
                    attempt,
                    output: stdout.to_string(),
                }))
                .await;
        }
        ExecutionResult::Failure(execution_failure) => match execution_failure {
            ExecutionFailure::CommandFailed { exit_code, stderr } => {
                let _ = tx
                    .send(Ok(StreamEvent::CommandExecutionFailed {
                        index: context.index,
                        call_id: context.call_id.clone(),
                        name: context.tool_name.clone(),
                        attempt: attempt.clone(),
                        error: format!("exit_code: {exit_code}, stderr: {stderr}"),
                    }))
                    .await;
            }
            ExecutionFailure::SandboxDenied {
                output,
                network_context,
            } => {
                let _ = tx
                    .send(Ok(StreamEvent::CommandExecutionFailed {
                        index: context.index,
                        call_id: context.call_id.clone(),
                        name: context.tool_name.clone(),
                        attempt: attempt.clone(),
                        error: format!("denied: {output}, network_context: {:?}", network_context),
                    }))
                    .await;
            }
        },
    }
    result.into()
}

/// 将底层执行失败折叠成 command runtime 终态。
impl From<ExecutionFailure> for RunCommandResult {
    fn from(value: ExecutionFailure) -> Self {
        match value {
            ExecutionFailure::CommandFailed { exit_code, stderr } => RunCommandResult::Failed {
                error: format!("exit_code: {exit_code}, stderr: {stderr}"),
            },
            ExecutionFailure::SandboxDenied {
                output,
                network_context,
            } => RunCommandResult::Denied {
                reason: format!("denied: {}, network_context: {:?}", output, network_context),
            },
        }
    }
}

impl From<ExecutionResult> for RunCommandResult {
    fn from(value: ExecutionResult) -> Self {
        match value {
            ExecutionResult::Success { stdout } => RunCommandResult::Finished { output: stdout },
            ExecutionResult::Failure(failure) => failure.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::stream_event::StreamEvent;
    use crate::model::{
        approval::{
            ApprovalPersistence, ApprovalPolicy, ApprovalScope, NetworkPolicy, SandboxProfile,
        },
        capability::CapabilityKind,
        execution::NetworkApprovalContext,
    };
    use crate::tool::shell::approval::ToolApprovalResult;
    use crate::tool::shell::registry::CapabilityRegistry;
    use std::{
        collections::VecDeque,
        path::PathBuf,
        sync::{Arc, Mutex},
    };
    use tokio::sync::mpsc;

    struct RecordingExecutionRunner {
        results: Mutex<VecDeque<ExecutionResult>>,
        attempts: Mutex<Vec<ExecutionAttempt>>,
    }

    impl RecordingExecutionRunner {
        fn new(results: Vec<ExecutionResult>) -> Self {
            Self {
                results: Mutex::new(VecDeque::from(results)),
                attempts: Mutex::new(Vec::new()),
            }
        }

        fn attempts(&self) -> Vec<ExecutionAttempt> {
            self.attempts.lock().unwrap().clone()
        }
    }

    impl ExecutionRunner for RecordingExecutionRunner {
        fn run(&self, _request: &CommandRequest, attempt: &ExecutionAttempt) -> ExecutionResult {
            self.attempts.lock().unwrap().push(attempt.clone());
            self.results
                .lock()
                .unwrap()
                .pop_front()
                .expect("test runner should have enough scripted results")
        }
    }

    struct RecordingApprovalResponder {
        decisions: Mutex<VecDeque<UserApprovalDecision>>,
        requests: Mutex<Vec<(ToolCallContext, ApprovalScope)>>,
        result_tx: mpsc::Sender<ToolApprovalResult>,
    }

    impl RecordingApprovalResponder {
        fn new(
            decisions: Vec<UserApprovalDecision>,
            result_tx: mpsc::Sender<ToolApprovalResult>,
        ) -> Self {
            Self {
                decisions: Mutex::new(VecDeque::from(decisions)),
                requests: Mutex::new(Vec::new()),
                result_tx,
            }
        }

        async fn handle_event(&self, event: &StreamEvent) {
            let StreamEvent::CommandNeedsApproval {
                approval_id,
                index,
                call_id,
                name,
                scope,
                ..
            } = event
            else {
                return;
            };

            self.requests.lock().unwrap().push((
                ToolCallContext {
                    index: *index,
                    call_id: call_id.clone(),
                    tool_name: name.clone(),
                },
                scope.clone(),
            ));

            let decision = self
                .decisions
                .lock()
                .unwrap()
                .pop_front()
                .expect("test approval responder should have enough scripted decisions");

            self.result_tx
                .send(ToolApprovalResult {
                    approval_id: approval_id.clone(),
                    decision,
                })
                .await
                .expect("test should send approval result");
        }

        fn scopes(&self) -> Vec<ApprovalScope> {
            self.requests
                .lock()
                .unwrap()
                .iter()
                .map(|(_, scope)| scope.clone())
                .collect()
        }

        fn contexts(&self) -> Vec<ToolCallContext> {
            self.requests
                .lock()
                .unwrap()
                .iter()
                .map(|(context, _)| context.clone())
                .collect()
        }
    }

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| item.to_string()).collect()
    }

    fn request(
        argv: &[&str],
        capability: CapabilityKind,
        approval_policy: ApprovalPolicy,
        sandbox_profile: SandboxProfile,
        network_policy: NetworkPolicy,
    ) -> CommandRequest {
        CommandRequest {
            raw_command: argv.join(" "),
            argv: strings(argv),
            cwd: PathBuf::from("/workspace"),
            capability,
            approval_policy,
            sandbox_profile,
            network_policy,
            justification: None,
        }
    }

    fn matched(argv: &[&str]) -> MatchedCapability {
        CapabilityRegistry::new()
            .match_capability(&strings(argv))
            .expect("test command should match a capability")
    }

    fn tool_call_context() -> ToolCallContext {
        ToolCallContext {
            index: 7,
            call_id: "call_shell".to_string(),
            tool_name: "run_command".to_string(),
        }
    }

    fn approved() -> UserApprovalDecision {
        UserApprovalDecision::Approved {
            persistence: ApprovalPersistence::Once,
        }
    }

    fn execution_runner(
        results: Vec<ExecutionResult>,
    ) -> (
        Arc<RecordingExecutionRunner>,
        Arc<dyn ExecutionRunner + Send + Sync + 'static>,
    ) {
        let runner = Arc::new(RecordingExecutionRunner::new(results));
        let trait_object = runner.clone() as Arc<dyn ExecutionRunner + Send + Sync + 'static>;
        (runner, trait_object)
    }

    fn approval_gateway(
        decisions: Vec<UserApprovalDecision>,
    ) -> (Arc<RecordingApprovalResponder>, Arc<ApprovalGateway>) {
        let (result_tx, result_rx) = mpsc::channel(16);
        let responder = Arc::new(RecordingApprovalResponder::new(decisions, result_tx));
        let gateway = Arc::new(ApprovalGateway::new(result_rx));
        (responder, gateway)
    }

    async fn run_command_case(
        request: &CommandRequest,
        argv: &[&str],
        execution_runner: Arc<dyn ExecutionRunner + Send + Sync + 'static>,
        approval_responder: Arc<RecordingApprovalResponder>,
        approval_gateway: Arc<ApprovalGateway>,
    ) -> (RunCommandResult, Vec<StreamEvent>) {
        let (tx, mut rx) = mpsc::channel(16);
        let mut run = Box::pin(run_shell_command(
            request,
            tool_call_context(),
            matched(argv),
            execution_runner,
            approval_gateway,
            &tx,
        ));
        let mut events = vec![];

        let mut result = None;
        while result.is_none() {
            tokio::select! {
                run_result = &mut run => {
                    result = Some(run_result);
                    while let Ok(event) = rx.try_recv() {
                        let event = event.expect("command event should be ok");
                        approval_responder.handle_event(&event).await;
                        events.push(event);
                    }
                }
                event = rx.recv() => {
                    let event = event
                        .expect("command should not close event channel before finishing")
                        .expect("command event should be ok");
                    approval_responder.handle_event(&event).await;
                    events.push(event);
                }
            }
        }

        (result.expect("command should return result"), events)
    }

    fn assert_finished(result: RunCommandResult) {
        let _ = finished_output(result);
    }

    fn finished_output(result: RunCommandResult) -> String {
        match result {
            RunCommandResult::Finished { output } => output,
            other => panic!("expected command to finish, got {}", result_label(&other)),
        }
    }

    fn assert_failed(result: RunCommandResult) {
        match result {
            RunCommandResult::Failed { .. } => {}
            other => panic!("expected command failure, got {}", result_label(&other)),
        }
    }

    fn assert_denied(result: RunCommandResult) {
        match result {
            RunCommandResult::Denied { .. } => {}
            other => panic!("expected command denial, got {}", result_label(&other)),
        }
    }

    fn sandbox_first(sandbox_profile: SandboxProfile) -> ExecutionAttempt {
        ExecutionAttempt::SandboxFirst { sandbox_profile }
    }

    fn started_attempts(events: &[StreamEvent]) -> Vec<ExecutionAttempt> {
        events
            .iter()
            .filter_map(|event| match event {
                StreamEvent::CommandExecutionStarted { attempt, .. } => Some(attempt.clone()),
                _ => None,
            })
            .collect()
    }

    fn finished_attempts(events: &[StreamEvent]) -> Vec<ExecutionAttempt> {
        events
            .iter()
            .filter_map(|event| match event {
                StreamEvent::CommandExecutionFinished { attempt, .. } => Some(attempt.clone()),
                _ => None,
            })
            .collect()
    }

    fn failed_attempts(events: &[StreamEvent]) -> Vec<ExecutionAttempt> {
        events
            .iter()
            .filter_map(|event| match event {
                StreamEvent::CommandExecutionFailed { attempt, .. } => Some(attempt.clone()),
                _ => None,
            })
            .collect()
    }

    fn retry_decisions(events: &[StreamEvent]) -> Vec<RetryDecision> {
        events
            .iter()
            .filter_map(|event| match event {
                StreamEvent::CommandRetryEvaluated { decision, .. } => Some(decision.clone()),
                _ => None,
            })
            .collect()
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

    fn assert_retry_success_trace(
        events: &[StreamEvent],
        sandbox_profile: SandboxProfile,
        retry_decision_predicate: impl Fn(&RetryDecision) -> bool,
    ) {
        assert!(matches!(
            started_attempts(events).as_slice(),
            [
                ExecutionAttempt::SandboxFirst { sandbox_profile: actual_profile },
                ExecutionAttempt::NoSandboxRetry { .. }
            ] if *actual_profile == sandbox_profile
        ));
        assert_eq!(
            failed_attempts(events),
            vec![sandbox_first(sandbox_profile)]
        );
        assert!(matches!(
            finished_attempts(events).as_slice(),
            [ExecutionAttempt::NoSandboxRetry { .. }]
        ));

        let sandbox_started = event_position(events, |event| {
            matches!(
                event,
                StreamEvent::CommandExecutionStarted {
                    attempt: ExecutionAttempt::SandboxFirst { sandbox_profile: actual_profile },
                    ..
                } if *actual_profile == sandbox_profile
            )
        });
        let sandbox_failed = event_position(events, |event| {
            matches!(
                event,
                StreamEvent::CommandExecutionFailed {
                    attempt: ExecutionAttempt::SandboxFirst { sandbox_profile: actual_profile },
                    ..
                } if *actual_profile == sandbox_profile
            )
        });
        let retry_evaluated = event_position(events, |event| match event {
            StreamEvent::CommandRetryEvaluated { decision, .. } => {
                retry_decision_predicate(decision)
            }
            _ => false,
        });
        let retry_started = event_position(events, |event| {
            matches!(
                event,
                StreamEvent::CommandExecutionStarted {
                    attempt: ExecutionAttempt::NoSandboxRetry { .. },
                    ..
                }
            )
        });
        let retry_finished = event_position(events, |event| {
            matches!(
                event,
                StreamEvent::CommandExecutionFinished {
                    attempt: ExecutionAttempt::NoSandboxRetry { .. },
                    ..
                }
            )
        });

        assert_in_order(&[
            sandbox_started,
            sandbox_failed,
            retry_evaluated,
            retry_started,
            retry_finished,
        ]);
    }

    fn assert_event_context(event: &StreamEvent) {
        match event {
            StreamEvent::CommandExecutionStarted {
                index,
                call_id,
                name,
                ..
            }
            | StreamEvent::CommandExecutionFinished {
                index,
                call_id,
                name,
                ..
            }
            | StreamEvent::CommandExecutionFailed {
                index,
                call_id,
                name,
                ..
            }
            | StreamEvent::CommandRetryEvaluated {
                index,
                call_id,
                name,
                ..
            }
            | StreamEvent::CommandNeedsApproval {
                index,
                call_id,
                name,
                ..
            } => {
                assert_eq!(*index, 7);
                assert_eq!(call_id, "call_shell");
                assert_eq!(name, "run_command");
            }
            other => panic!("expected command event, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn skip_without_bypass_runs_once_in_sandbox() {
        let argv = ["cat", "package.json"];
        let request = request(
            &argv,
            CapabilityKind::SafeRead,
            ApprovalPolicy::OnFailure,
            SandboxProfile::ReadOnly,
            NetworkPolicy::Deny,
        );
        let (runner, runner_trait) = execution_runner(vec![ExecutionResult::Success {
            stdout: "read ok".to_string(),
        }]);
        let (decider, gateway) = approval_gateway(vec![]);

        let (result, events) =
            run_command_case(&request, &argv, runner_trait, decider.clone(), gateway).await;

        assert_eq!(finished_output(result), "read ok");
        assert_eq!(
            runner.attempts(),
            vec![sandbox_first(SandboxProfile::ReadOnly)]
        );
        assert_eq!(
            started_attempts(&events),
            vec![sandbox_first(SandboxProfile::ReadOnly)]
        );
        assert_eq!(
            finished_attempts(&events),
            vec![sandbox_first(SandboxProfile::ReadOnly)]
        );
        assert!(failed_attempts(&events).is_empty());
        assert!(retry_decisions(&events).is_empty());
        events.iter().for_each(assert_event_context);
        assert!(decider.scopes().is_empty());
    }

    #[tokio::test]
    async fn safe_read_sandbox_denied_does_not_retry_because_capability_retry_policy_is_never() {
        let argv = ["cat", "/private/file"];
        let request = request(
            &argv,
            CapabilityKind::SafeRead,
            ApprovalPolicy::OnFailure,
            SandboxProfile::ReadOnly,
            NetworkPolicy::Deny,
        );
        let (runner, runner_trait) = execution_runner(vec![ExecutionResult::Failure(
            ExecutionFailure::SandboxDenied {
                output: "read denied by sandbox".to_string(),
                network_context: None,
            },
        )]);
        let (decider, gateway) = approval_gateway(vec![]);

        let (result, events) =
            run_command_case(&request, &argv, runner_trait, decider.clone(), gateway).await;

        assert_denied(result);
        assert_eq!(
            runner.attempts(),
            vec![sandbox_first(SandboxProfile::ReadOnly)]
        );
        assert_eq!(
            started_attempts(&events),
            vec![sandbox_first(SandboxProfile::ReadOnly)]
        );
        assert_eq!(
            failed_attempts(&events),
            vec![sandbox_first(SandboxProfile::ReadOnly)]
        );
        assert!(matches!(
            retry_decisions(&events).as_slice(),
            [RetryDecision::DoNotRetry { .. }]
        ));
        events.iter().for_each(assert_event_context);
        assert!(decider.scopes().is_empty());
    }

    #[tokio::test]
    async fn needs_approval_rejection_denies_without_running_command() {
        let argv = ["npm", "install", "vite"];
        let request = request(
            &argv,
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnRequest,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Prompt,
        );
        let (runner, runner_trait) = execution_runner(vec![]);
        let (decider, gateway) = approval_gateway(vec![UserApprovalDecision::Rejected]);

        let (result, events) =
            run_command_case(&request, &argv, runner_trait, decider.clone(), gateway).await;

        assert_denied(result);
        assert!(runner.attempts().is_empty());
        assert!(matches!(
            events.as_slice(),
            [StreamEvent::CommandNeedsApproval { .. }]
        ));
        let scopes = decider.scopes();
        assert_eq!(scopes.len(), 1);
        assert_eq!(scopes[0].sandbox_profile, SandboxProfile::WorkspaceWrite);
        let contexts = decider.contexts();
        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts[0].index, 7);
        assert_eq!(contexts[0].call_id, "call_shell");
        assert_eq!(contexts[0].tool_name, "run_command");
    }

    #[tokio::test]
    async fn needs_approval_approval_still_runs_sandbox_first() {
        let argv = ["npm", "install", "vite"];
        let request = request(
            &argv,
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnRequest,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Prompt,
        );
        let (runner, runner_trait) = execution_runner(vec![ExecutionResult::Success {
            stdout: "install ok in sandbox".to_string(),
        }]);
        let (decider, gateway) = approval_gateway(vec![approved()]);

        let (result, events) =
            run_command_case(&request, &argv, runner_trait, decider.clone(), gateway).await;

        assert_finished(result);
        assert_eq!(
            runner.attempts(),
            vec![sandbox_first(SandboxProfile::WorkspaceWrite)]
        );
        assert_eq!(
            started_attempts(&events),
            vec![sandbox_first(SandboxProfile::WorkspaceWrite)]
        );
        assert_eq!(
            finished_attempts(&events),
            vec![sandbox_first(SandboxProfile::WorkspaceWrite)]
        );
        events.iter().for_each(assert_event_context);
        let scopes = decider.scopes();
        assert_eq!(scopes.len(), 1);
        assert_eq!(scopes[0].sandbox_profile, SandboxProfile::WorkspaceWrite);
    }

    #[tokio::test]
    async fn command_failure_in_sandbox_does_not_retry() {
        let argv = ["npm", "test", "--", "fail"];
        let request = request(
            &argv,
            CapabilityKind::SafeTest,
            ApprovalPolicy::OnFailure,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Deny,
        );
        let (runner, runner_trait) = execution_runner(vec![ExecutionResult::Failure(
            ExecutionFailure::CommandFailed {
                exit_code: 1,
                stderr: "test failed".to_string(),
            },
        )]);
        let (decider, gateway) = approval_gateway(vec![]);

        let (result, events) =
            run_command_case(&request, &argv, runner_trait, decider, gateway).await;

        assert_failed(result);
        assert_eq!(
            runner.attempts(),
            vec![sandbox_first(SandboxProfile::WorkspaceWrite)]
        );
        assert_eq!(
            started_attempts(&events),
            vec![sandbox_first(SandboxProfile::WorkspaceWrite)]
        );
        assert_eq!(
            failed_attempts(&events),
            vec![sandbox_first(SandboxProfile::WorkspaceWrite)]
        );
        assert!(matches!(
            retry_decisions(&events).as_slice(),
            [RetryDecision::DoNotRetry { .. }]
        ));
        events.iter().for_each(assert_event_context);
    }

    #[tokio::test]
    async fn sandbox_denied_with_network_allow_retries_without_second_approval() {
        let argv = ["npm", "install", "vite"];
        let request = request(
            &argv,
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnFailure,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Allow,
        );
        let (runner, runner_trait) = execution_runner(vec![
            ExecutionResult::Failure(ExecutionFailure::SandboxDenied {
                output: "network blocked by sandbox".to_string(),
                network_context: Some(NetworkApprovalContext {
                    host: "registry.npmjs.org".to_string(),
                    protocol: "https".to_string(),
                }),
            }),
            ExecutionResult::Success {
                stdout: "install ok without sandbox".to_string(),
            },
        ]);
        let (decider, gateway) = approval_gateway(vec![approved()]);

        let (result, events) =
            run_command_case(&request, &argv, runner_trait, decider.clone(), gateway).await;

        assert_finished(result);
        let attempts = runner.attempts();
        assert_eq!(attempts.len(), 2);
        assert_eq!(attempts[0], sandbox_first(SandboxProfile::WorkspaceWrite));
        assert!(matches!(
            attempts[1],
            ExecutionAttempt::NoSandboxRetry { .. }
        ));
        assert_retry_success_trace(&events, SandboxProfile::WorkspaceWrite, |decision| {
            matches!(decision, RetryDecision::RetryWithoutApproval { .. })
        });
        events.iter().for_each(assert_event_context);
        assert_eq!(decider.scopes().len(), 1);
    }

    #[tokio::test]
    async fn sandbox_denied_with_network_prompt_requires_retry_approval_with_no_sandbox_scope() {
        let argv = ["npm", "install", "vite"];
        let request = request(
            &argv,
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnFailure,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Prompt,
        );
        let (runner, runner_trait) = execution_runner(vec![
            ExecutionResult::Failure(ExecutionFailure::SandboxDenied {
                output: "network blocked by sandbox".to_string(),
                network_context: Some(NetworkApprovalContext {
                    host: "registry.npmjs.org".to_string(),
                    protocol: "https".to_string(),
                }),
            }),
            ExecutionResult::Success {
                stdout: "install ok after approval".to_string(),
            },
        ]);
        let (decider, gateway) = approval_gateway(vec![approved(), approved()]);

        let (result, events) =
            run_command_case(&request, &argv, runner_trait, decider.clone(), gateway).await;

        assert_finished(result);
        let attempts = runner.attempts();
        assert_eq!(attempts.len(), 2);
        assert_eq!(attempts[0], sandbox_first(SandboxProfile::WorkspaceWrite));
        assert!(matches!(
            attempts[1],
            ExecutionAttempt::NoSandboxRetry { .. }
        ));
        assert_retry_success_trace(&events, SandboxProfile::WorkspaceWrite, |decision| {
            matches!(decision, RetryDecision::RetryWithApproval { .. })
        });
        events.iter().for_each(assert_event_context);
        let scopes = decider.scopes();
        assert_eq!(scopes.len(), 2);
        assert_eq!(scopes[0].sandbox_profile, SandboxProfile::WorkspaceWrite);
        assert_eq!(scopes[1].sandbox_profile, SandboxProfile::NoSandbox);
        let contexts = decider.contexts();
        assert_eq!(contexts.len(), 2);
        assert!(
            contexts.iter().all(
                |context| context.call_id == "call_shell" && context.tool_name == "run_command"
            )
        );
    }

    #[tokio::test]
    async fn sandbox_denied_retry_approval_rejection_denies_without_no_sandbox_retry() {
        let argv = ["npm", "install", "vite"];
        let request = request(
            &argv,
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnFailure,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Prompt,
        );
        let (runner, runner_trait) = execution_runner(vec![ExecutionResult::Failure(
            ExecutionFailure::SandboxDenied {
                output: "network blocked by sandbox".to_string(),
                network_context: Some(NetworkApprovalContext {
                    host: "registry.npmjs.org".to_string(),
                    protocol: "https".to_string(),
                }),
            },
        )]);
        let (decider, gateway) = approval_gateway(vec![approved(), UserApprovalDecision::Rejected]);

        let (result, events) =
            run_command_case(&request, &argv, runner_trait, decider.clone(), gateway).await;

        assert_denied(result);
        assert_eq!(
            runner.attempts(),
            vec![sandbox_first(SandboxProfile::WorkspaceWrite)]
        );
        assert_eq!(
            started_attempts(&events),
            vec![sandbox_first(SandboxProfile::WorkspaceWrite)]
        );
        assert!(matches!(
            retry_decisions(&events).as_slice(),
            [RetryDecision::RetryWithApproval { .. }]
        ));
        assert_eq!(
            failed_attempts(&events),
            vec![sandbox_first(SandboxProfile::WorkspaceWrite)]
        );
        events.iter().for_each(assert_event_context);
        let scopes = decider.scopes();
        assert_eq!(scopes.len(), 2);
        assert_eq!(scopes[1].sandbox_profile, SandboxProfile::NoSandbox);
    }

    fn result_label(result: &RunCommandResult) -> &'static str {
        match result {
            RunCommandResult::Finished { .. } => "Finished",
            RunCommandResult::Failed { .. } => "Failed",
            RunCommandResult::Denied { .. } => "Denied",
        }
    }
}
