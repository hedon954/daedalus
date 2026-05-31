use std::{env::current_dir, path::PathBuf};

use crate::{
    agent::stream_event::ToolCallFinished,
    model::{
        approval::{ApprovalPolicy, SandboxProfile},
        command_request::CommandRequest,
    },
    registry::{CapabilityRegistry, MatchedCapability},
    sandbox::simulated_sandbox_runner::SimulatedSandboxRunner,
    tool::function::run_pure_function,
};

pub struct ToolRuntime {
    context: ToolRuntimeContext,
}

pub struct ToolRuntimeContext {
    pub cwd: PathBuf,
    pub approval_policy: ApprovalPolicy,
    pub default_sandbox: SandboxProfile,
    pub registry: CapabilityRegistry,
    pub runner: SimulatedSandboxRunner,
}

#[derive(Debug, Clone, Copy)]
pub enum ToolKind {
    PureFunction,
    Command,
}

#[derive(Debug, Clone, Copy)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub kind: ToolKind,
}

enum ToolRuntimePlan {
    RunPureFunction {
        name: String,
        arguments: String,
    },
    RunCommand {
        request: CommandRequest,
        matched_capability: MatchedCapability,
    },
    Deny {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolRuntimeResult {
    Finished { output: String },
    Failed { error: String },
    Denied { reason: String },
    Skipped { reason: String },
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
];

impl ToolRuntime {
    pub fn new(context: ToolRuntimeContext) -> Self {
        Self { context }
    }

    pub fn run(&self, call: &ToolCallFinished) -> ToolRuntimeResult {
        let Some(definition) = self.find_tool(&call.name) else {
            return ToolRuntimeResult::Failed {
                error: format!("cannot find tool: {}", call.name),
            };
        };
        let plan = self.plan_call(definition, call);
        self.execute_plan(plan)
    }

    fn find_tool(&self, name: &str) -> Option<ToolDefinition> {
        TOOLS.iter().find(|t| t.name == name).copied()
    }

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
            ToolKind::Command => unimplemented!(),
        }
    }

    fn execute_plan(&self, plan: ToolRuntimePlan) -> ToolRuntimeResult {
        match plan {
            ToolRuntimePlan::RunPureFunction { name, arguments } => {
                match run_pure_function(&name, &arguments) {
                    Ok(output) => ToolRuntimeResult::Finished { output },
                    Err(err) => ToolRuntimeResult::Failed {
                        error: err.to_string(),
                    },
                }
            }
            _ => unimplemented!(),
        }
    }
}

impl Default for ToolRuntime {
    fn default() -> Self {
        Self {
            context: ToolRuntimeContext {
                cwd: current_dir().unwrap_or(".".into()),
                approval_policy: ApprovalPolicy::OnFailure,
                default_sandbox: SandboxProfile::ReadOnly,
                registry: CapabilityRegistry::new(),
                runner: SimulatedSandboxRunner::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        agent::stream_event::ToolCallFinished,
        tool::runtime::{ToolRuntime, ToolRuntimeResult},
    };

    fn tool_call(name: &str, arguments: &str) -> ToolCallFinished {
        ToolCallFinished {
            index: 0,
            call_id: "call_test".to_string(),
            name: name.to_string(),
            arguments: arguments.to_string(),
        }
    }

    #[test]
    fn pure_function_add_should_finish_with_output() {
        let runtime = ToolRuntime::default();

        let result = runtime.run(&tool_call("add", r#"{"a": 999, "b": 666}"#));

        assert_eq!(
            result,
            ToolRuntimeResult::Finished {
                output: "1665".to_string()
            }
        );
    }

    #[test]
    fn pure_function_sub_should_finish_with_output() {
        let runtime = ToolRuntime::default();

        let result = runtime.run(&tool_call("sub", r#"{"a": 321, "b": 123}"#));

        assert_eq!(
            result,
            ToolRuntimeResult::Finished {
                output: "198".to_string()
            }
        );
    }

    #[test]
    fn pure_function_invalid_arguments_should_fail_without_pinning_error_text() {
        let runtime = ToolRuntime::default();

        let result = runtime.run(&tool_call("add", r#"{"a": "999", "b": 666}"#));

        assert!(matches!(result, ToolRuntimeResult::Failed { .. }));
    }

    #[test]
    fn unknown_tool_should_fail_without_pinning_error_text() {
        let runtime = ToolRuntime::default();

        let result = runtime.run(&tool_call("mul", r#"{"a": 2, "b": 3}"#));

        assert!(matches!(result, ToolRuntimeResult::Failed { .. }));
    }
}
