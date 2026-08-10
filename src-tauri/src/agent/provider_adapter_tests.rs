use super::{
    AgentAdapter, AgentProfile, AgentProfileKind, NativeProcessAdapter, NormalizedAgentEventKind,
    ProcessAgentDefinition, WorkerAgentAdapter, provider_adapter::native_definition,
};
use crate::domain::{AgentEffort, AgentModelPreference};

fn profile(kind: AgentProfileKind, program: &str, arguments: Vec<&str>) -> AgentProfile {
    AgentProfile {
        name: "local-provider".to_owned(),
        kind,
        program: program.to_owned(),
        arguments: arguments.into_iter().map(str::to_owned).collect(),
    }
}

#[test]
fn native_profiles_own_their_protocol_and_safety_arguments() {
    let codex = native_definition(
        profile(AgentProfileKind::CodexCli, "codex", Vec::new()),
        "execution-1",
        &AgentModelPreference::Named("gpt-5".to_owned()),
        AgentEffort::Thorough,
    );
    let claude = native_definition(
        profile(AgentProfileKind::ClaudeCode, "claude", Vec::new()),
        "execution-1",
        &AgentModelPreference::Named("sonnet".to_owned()),
        AgentEffort::Balanced,
    );
    let cline_pass = native_definition(
        profile(AgentProfileKind::ClinePassCli, "cline", Vec::new()),
        "execution-1",
        &AgentModelPreference::ProviderDefault,
        AgentEffort::Focused,
    );

    assert_eq!(codex.name, "local-provider-execution-1");
    assert_eq!(
        codex.arguments,
        [
            "exec",
            "--json",
            "--sandbox",
            "workspace-write",
            "--model",
            "gpt-5",
            "--config",
            "model_reasoning_effort=\"high\"",
            "-",
        ]
    );
    assert_eq!(claude.name, "local-provider-execution-1");
    assert_eq!(
        claude.arguments,
        [
            "--print",
            "--output-format",
            "stream-json",
            "--verbose",
            "--permission-mode",
            "acceptEdits",
            "--model",
            "sonnet",
            "--effort",
            "medium",
        ]
    );
    assert_eq!(cline_pass.name, "local-provider-execution-1");
    assert_eq!(
        cline_pass.arguments,
        [
            "--json",
            "--provider",
            "cline",
            "--auto-approve",
            "true",
            "--thinking",
            "low",
        ]
    );
}

#[test]
fn worker_adapter_selects_the_profile_protocol_at_the_runtime_boundary() {
    let structured = WorkerAgentAdapter::from_profile_for_execution(
        profile(AgentProfileKind::StructuredProcess, "provider", Vec::new()),
        "execution-1",
        &AgentModelPreference::ProviderDefault,
        AgentEffort::ProviderDefault,
    );
    let codex = WorkerAgentAdapter::from_profile_for_execution(
        profile(AgentProfileKind::CodexCli, "provider", Vec::new()),
        "execution-1",
        &AgentModelPreference::ProviderDefault,
        AgentEffort::ProviderDefault,
    );
    let claude = WorkerAgentAdapter::from_profile_for_execution(
        profile(AgentProfileKind::ClaudeCode, "provider", Vec::new()),
        "execution-1",
        &AgentModelPreference::ProviderDefault,
        AgentEffort::ProviderDefault,
    );
    let cline_pass = WorkerAgentAdapter::from_profile_for_execution(
        profile(AgentProfileKind::ClinePassCli, "provider", Vec::new()),
        "execution-1",
        &AgentModelPreference::ProviderDefault,
        AgentEffort::ProviderDefault,
    );

    assert!(matches!(structured, WorkerAgentAdapter::Structured(_)));
    assert!(matches!(codex, WorkerAgentAdapter::Native(_)));
    assert!(matches!(claude, WorkerAgentAdapter::Native(_)));
    assert!(matches!(cline_pass, WorkerAgentAdapter::Native(_)));
    for adapter in [&structured, &codex, &claude, &cline_pass] {
        let capabilities = adapter
            .discover()
            .expect("runtime adapter capabilities should be available");
        assert_eq!(adapter.name(), "local-provider-execution-1");
        assert!(capabilities.streams_structured_events);
        assert!(!capabilities.supports_feedback);
        assert!(!capabilities.supports_resume);
        assert!(!capabilities.supports_interrupt);
    }
}

#[test]
fn cline_pass_cli_adapter_passes_the_noninteractive_conformance_suite() {
    let mut adapter = NativeProcessAdapter::for_test(
        scripted_adapter(
            "cline-pass-fixture",
            "IFS= read -r brief; [ \"$brief\" = \"Build the bounded task.\" ] || exit 7; printf '%s\\n' '{\"type\":\"agent_event\",\"event\":{\"type\":\"iteration_start\"}}' '{\"type\":\"agent_event\",\"event\":{\"type\":\"usage\",\"usage\":{\"inputTokens\":13,\"outputTokens\":5}}}' '{\"type\":\"agent_event\",\"event\":{\"type\":\"done\"}}'",
        ),
        AgentProfileKind::ClinePassCli,
    );

    let events = super::conformance_tests::assert_noninteractive_conformance(
        &mut adapter,
        3,
        "ClinePass completed the task and is ready for review.",
    );

    assert!(matches!(
        events[1].kind,
        NormalizedAgentEventKind::UsageUpdated {
            input_tokens: 13,
            output_tokens: 5,
            cost_micros: None,
        }
    ));
}

#[test]
fn codex_cli_adapter_passes_the_noninteractive_conformance_suite() {
    let mut adapter = NativeProcessAdapter::for_test(
        scripted_adapter(
            "codex-fixture",
            "IFS= read -r brief; [ \"$brief\" = \"Build the bounded task.\" ] || exit 7; printf '%s\\n' '{\"type\":\"turn.started\"}' '{\"type\":\"turn.completed\",\"usage\":{\"input_tokens\":21,\"output_tokens\":8}}'",
        ),
        AgentProfileKind::CodexCli,
    );

    let events = super::conformance_tests::assert_noninteractive_conformance(
        &mut adapter,
        3,
        "Codex completed the task and is ready for review.",
    );

    assert!(matches!(
        events[1].kind,
        NormalizedAgentEventKind::UsageUpdated {
            input_tokens: 21,
            output_tokens: 8,
            cost_micros: None,
        }
    ));
}

#[test]
fn claude_code_adapter_passes_the_noninteractive_conformance_suite() {
    let mut adapter = NativeProcessAdapter::for_test(
        scripted_adapter(
            "claude-fixture",
            "IFS= read -r brief; [ \"$brief\" = \"Build the bounded task.\" ] || exit 7; printf '%s\\n' '{\"type\":\"system\",\"subtype\":\"init\"}' '{\"type\":\"result\",\"usage\":{\"input_tokens\":11,\"output_tokens\":4,\"total_cost_usd\":0.000015}}'",
        ),
        AgentProfileKind::ClaudeCode,
    );

    let events = super::conformance_tests::assert_noninteractive_conformance(
        &mut adapter,
        3,
        "Claude Code completed the task and is ready for review.",
    );

    assert!(matches!(
        events[1].kind,
        NormalizedAgentEventKind::UsageUpdated {
            input_tokens: 11,
            output_tokens: 4,
            cost_micros: Some(15),
        }
    ));
}

fn scripted_adapter(name: &str, script: &str) -> ProcessAgentDefinition {
    ProcessAgentDefinition {
        name: name.to_owned(),
        program: "sh".to_owned(),
        arguments: vec!["-c".to_owned(), script.to_owned()],
    }
}
