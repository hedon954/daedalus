use std::sync::Arc;

use serde_json::json;

use crate::{
    agent::react::EventSender,
    model::{
        approval::{ApprovalPersistence, ApprovalRequirement},
        command_request::CommandRequest,
        event::{ExecutionAttempt, RetryDecision, UserApprovalDecision},
        execution::{ExecutionFailure, ExecutionResult},
    },
    tool::shell::{
        approval::{
            ApprovalGateway, ToolApprovalRequest, ToolCallContext, build_approval_scope,
            resolve_approval_requirement,
        },
        event_emitter::CommandEventEmitter,
        execution::ExecutionRunner,
        registry::MatchedCapability,
        retry::decide_retry,
    },
};

pub mod approval;
mod event_emitter;
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

/// OpenAI-compatible function tool schema for `run_command`.
pub fn run_command_spec() -> serde_json::Value {
    json!({
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
    })
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
    let emitter = CommandEventEmitter::new(tx.clone(), context);

    match resolve_approval_requirement(&request, &matched_capability) {
        ApprovalRequirement::Skip {
            bypass_sandbox,
            reason,
        } => {
            // 不需要审批但是需要先走一遍沙箱
            if !bypass_sandbox {
                return run_sandbox_first_flow(
                    request,
                    matched_capability,
                    execution_runner.as_ref(),
                    approval_gateway,
                    &emitter,
                )
                .await;
            }

            // 不需要审批且跳过沙箱，直接在本地运行
            return run_no_sandbox_first(request, execution_runner.as_ref(), reason, &emitter)
                .await;
        }
        ApprovalRequirement::NeedsApproval {
            reason,
            approval_scope,
        } => match request_approval(
            &approval_gateway,
            ToolApprovalRequest {
                reason: reason.clone(),
                scope: approval_scope,
            },
            &emitter,
        )
        .await
        {
            UserApprovalDecision::Approved { persistence: _ } => {
                return run_sandbox_first_flow(
                    request,
                    matched_capability,
                    execution_runner.as_ref(),
                    approval_gateway,
                    &emitter,
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
    emitter: &CommandEventEmitter,
) -> UserApprovalDecision {
    if approval_gateway.has_session_approval(&request.scope) {
        return UserApprovalDecision::Approved {
            persistence: ApprovalPersistence::Session,
        };
    }

    let pending = approval_gateway.create_pending_approval(request.clone());
    let approval_id = pending.approval_id.clone();

    let decision = match emitter
        .needs_approval(approval_id.clone(), request.reason, request.scope.clone())
        .await
    {
        Ok(_) => pending.wait().await,
        Err(_) => UserApprovalDecision::Rejected,
    };

    approval_gateway.cancel(&approval_id);
    approval_gateway.remember_approval(&request.scope, &decision);

    decision
}

async fn run_sandbox_first_flow(
    request: &CommandRequest,
    matched_capability: MatchedCapability,
    execution_runner: &dyn ExecutionRunner,
    approval_gateway: Arc<ApprovalGateway>,
    emitter: &CommandEventEmitter,
) -> RunCommandResult {
    let attempt = ExecutionAttempt::SandboxFirst {
        sandbox_profile: request.sandbox_profile,
    };

    // 先在沙箱尝试执行
    let sandbox_first_execution_result =
        run_execution_attempt(request, attempt, execution_runner, emitter).await;

    // 处理沙箱执行结果
    match sandbox_first_execution_result {
        // 沙箱成功直接返回
        ExecutionResult::Success { stdout } => {
            return RunCommandResult::Finished { output: stdout };
        }

        // 沙箱失败则判断是否要进行重试
        ExecutionResult::Failure(failure) => {
            let retry_decision = decide_retry(
                request,
                &failure,
                &build_approval_scope(request, &matched_capability),
                matched_capability.capability.policy.retry_policy,
                false,
            );

            emitter.retry_evaluated(retry_decision.clone()).await;

            // 处理重试决策
            match retry_decision {
                // 不重试直接返回失败
                RetryDecision::DoNotRetry { reason: _ } => {
                    return failure.into();
                }
                // 重试且不需要审批则直接运行
                RetryDecision::RetryWithoutApproval { reason } => {
                    return run_no_sandbox_retry(request, execution_runner, reason, emitter).await;
                }
                // 重新且需要审批则先走审批
                RetryDecision::RetryWithApproval {
                    reason,
                    approval_scope,
                } => match request_approval(
                    &approval_gateway,
                    ToolApprovalRequest {
                        reason: reason.clone(),
                        scope: approval_scope,
                    },
                    emitter,
                )
                .await
                {
                    // 审批通过则直接本机运行再次尝试
                    UserApprovalDecision::Approved { persistence: _ } => {
                        return run_no_sandbox_retry(request, execution_runner, reason, emitter)
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
    execution_runner: &dyn ExecutionRunner,
    reason: String,
    emitter: &CommandEventEmitter,
) -> RunCommandResult {
    let attempt = ExecutionAttempt::NoSandboxFirst { reason };

    run_execution_attempt(request, attempt, execution_runner, emitter)
        .await
        .into()
}

async fn run_no_sandbox_retry(
    request: &CommandRequest,
    execution_runner: &dyn ExecutionRunner,
    reason: String,
    emitter: &CommandEventEmitter,
) -> RunCommandResult {
    let attempt = ExecutionAttempt::NoSandboxRetry { reason };

    run_execution_attempt(request, attempt, execution_runner, emitter)
        .await
        .into()
}

async fn run_execution_attempt(
    request: &CommandRequest,
    attempt: ExecutionAttempt,
    execution_runner: &dyn ExecutionRunner,
    emitter: &CommandEventEmitter,
) -> ExecutionResult {
    emitter.execution_started(&attempt).await;
    let result = execution_runner.run(request, &attempt).await;

    match &result {
        ExecutionResult::Success { stdout } => {
            emitter.execution_finished(&attempt, stdout.clone()).await;
        }
        ExecutionResult::Failure(execution_failure) => match execution_failure {
            ExecutionFailure::CommandFailed { exit_code, stderr } => {
                emitter
                    .execution_failed(
                        &attempt,
                        format!("exit_code: {exit_code}, stderr: {stderr}"),
                    )
                    .await;
            }
            ExecutionFailure::SandboxDenied {
                output,
                network_context,
            } => {
                emitter
                    .execution_failed(
                        &attempt,
                        format!("denied: {output}, network_context: {:?}", network_context),
                    )
                    .await;
            }
        },
    }

    result
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
    use async_trait::async_trait;
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

    #[async_trait]
    impl ExecutionRunner for RecordingExecutionRunner {
        async fn run(
            &self,
            _request: &CommandRequest,
            attempt: &ExecutionAttempt,
        ) -> ExecutionResult {
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

    fn approved_for_session() -> UserApprovalDecision {
        UserApprovalDecision::Approved {
            persistence: ApprovalPersistence::Session,
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

    fn approval_request_count(events: &[StreamEvent]) -> usize {
        events
            .iter()
            .filter(|event| matches!(event, StreamEvent::CommandNeedsApproval { .. }))
            .count()
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
    async fn session_approval_reuses_same_scope_without_new_prompt() {
        let argv = ["npm", "install", "vite"];
        let request = request(
            &argv,
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnRequest,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Prompt,
        );
        let (runner, runner_trait) = execution_runner(vec![
            ExecutionResult::Success {
                stdout: "first install ok".to_string(),
            },
            ExecutionResult::Success {
                stdout: "second install ok".to_string(),
            },
        ]);
        let (decider, gateway) = approval_gateway(vec![approved_for_session()]);

        let (first_result, first_events) = run_command_case(
            &request,
            &argv,
            runner_trait.clone(),
            decider.clone(),
            gateway.clone(),
        )
        .await;
        let (second_result, second_events) =
            run_command_case(&request, &argv, runner_trait, decider.clone(), gateway).await;

        assert_finished(first_result);
        assert_finished(second_result);
        assert_eq!(approval_request_count(&first_events), 1);
        assert_eq!(approval_request_count(&second_events), 0);
        assert_eq!(decider.scopes().len(), 1);
        assert_eq!(
            runner.attempts(),
            vec![
                sandbox_first(SandboxProfile::WorkspaceWrite),
                sandbox_first(SandboxProfile::WorkspaceWrite)
            ]
        );
    }

    #[tokio::test]
    async fn once_approval_does_not_reuse_same_scope() {
        let argv = ["npm", "install", "vite"];
        let request = request(
            &argv,
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnRequest,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Prompt,
        );
        let (_runner, runner_trait) = execution_runner(vec![
            ExecutionResult::Success {
                stdout: "first install ok".to_string(),
            },
            ExecutionResult::Success {
                stdout: "second install ok".to_string(),
            },
        ]);
        let (decider, gateway) = approval_gateway(vec![approved(), approved()]);

        let (_first_result, first_events) = run_command_case(
            &request,
            &argv,
            runner_trait.clone(),
            decider.clone(),
            gateway.clone(),
        )
        .await;
        let (_second_result, second_events) =
            run_command_case(&request, &argv, runner_trait, decider.clone(), gateway).await;

        assert_eq!(approval_request_count(&first_events), 1);
        assert_eq!(approval_request_count(&second_events), 1);
        assert_eq!(decider.scopes().len(), 2);
    }

    #[tokio::test]
    async fn session_approval_does_not_reuse_when_scope_changes() {
        let argv = ["npm", "install", "vite"];
        let first_request = request(
            &argv,
            CapabilityKind::NetworkInstall,
            ApprovalPolicy::OnRequest,
            SandboxProfile::WorkspaceWrite,
            NetworkPolicy::Prompt,
        );
        let mut second_request = first_request.clone();
        second_request.cwd = PathBuf::from("/other-workspace");
        let (_runner, runner_trait) = execution_runner(vec![
            ExecutionResult::Success {
                stdout: "first install ok".to_string(),
            },
            ExecutionResult::Success {
                stdout: "second install ok".to_string(),
            },
        ]);
        let (decider, gateway) = approval_gateway(vec![approved_for_session(), approved()]);

        let (_first_result, first_events) = run_command_case(
            &first_request,
            &argv,
            runner_trait.clone(),
            decider.clone(),
            gateway.clone(),
        )
        .await;
        let (_second_result, second_events) = run_command_case(
            &second_request,
            &argv,
            runner_trait,
            decider.clone(),
            gateway,
        )
        .await;

        assert_eq!(approval_request_count(&first_events), 1);
        assert_eq!(approval_request_count(&second_events), 1);
        let scopes = decider.scopes();
        assert_eq!(scopes.len(), 2);
        assert_ne!(scopes[0].cwd, scopes[1].cwd);
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
