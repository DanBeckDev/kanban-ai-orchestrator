use super::{
    AgentAdapter, AgentProfile, AgentProfileKind, ClaudeCodeAdapter, CodexCliAdapter,
    NormalizedAgentEventKind, ProcessAgentDefinition, WorkerAgentAdapter,
    provider_adapter::{claude_code_definition, codex_definition},
};

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
    let codex = codex_definition(
        profile(
            AgentProfileKind::CodexCli,
            "codex",
            vec!["--model", "gpt-5"],
        ),
        "execution-1",
    );
    let claude = claude_code_definition(
        profile(
            AgentProfileKind::ClaudeCode,
            "claude",
            vec!["--model", "sonnet"],
        ),
        "execution-1",
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
        ]
    );
}

#[test]
fn worker_adapter_selects_the_profile_protocol_at_the_runtime_boundary() {
    let structured = WorkerAgentAdapter::from_profile_for_execution(
        profile(AgentProfileKind::StructuredProcess, "provider", Vec::new()),
        "execution-1",
    );
    let codex = WorkerAgentAdapter::from_profile_for_execution(
        profile(AgentProfileKind::CodexCli, "provider", Vec::new()),
        "execution-1",
    );
    let claude = WorkerAgentAdapter::from_profile_for_execution(
        profile(AgentProfileKind::ClaudeCode, "provider", Vec::new()),
        "execution-1",
    );

    assert!(matches!(structured, WorkerAgentAdapter::Structured(_)));
    assert!(matches!(codex, WorkerAgentAdapter::Codex(_)));
    assert!(matches!(claude, WorkerAgentAdapter::ClaudeCode(_)));
    for adapter in [&structured, &codex, &claude] {
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
fn codex_cli_adapter_passes_the_noninteractive_conformance_suite() {
    let mut adapter = CodexCliAdapter::for_test(scripted_adapter(
        "codex-fixture",
        "IFS= read -r brief; [ \"$brief\" = \"Build the bounded task.\" ] || exit 7; printf '%s\\n' '{\"type\":\"turn.started\"}' '{\"type\":\"turn.completed\",\"usage\":{\"input_tokens\":21,\"output_tokens\":8}}'",
    ));

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
    let mut adapter = ClaudeCodeAdapter::for_test(scripted_adapter(
        "claude-fixture",
        "IFS= read -r brief; [ \"$brief\" = \"Build the bounded task.\" ] || exit 7; printf '%s\\n' '{\"type\":\"system\",\"subtype\":\"init\"}' '{\"type\":\"result\",\"usage\":{\"input_tokens\":11,\"output_tokens\":4,\"total_cost_usd\":0.000015}}'",
    ));

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
