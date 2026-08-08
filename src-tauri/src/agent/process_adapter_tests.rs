#![cfg(unix)]

use super::{
    AgentAdapter, AgentAdapterError, NormalizedAgentEventKind, ProcessAgentAdapter,
    ProcessAgentDefinition,
};

fn adapter(script: &str) -> ProcessAgentAdapter {
    ProcessAgentAdapter::new(ProcessAgentDefinition {
        name: "structured-script".to_owned(),
        program: "sh".to_owned(),
        arguments: vec!["-c".to_owned(), script.to_owned()],
    })
}

#[test]
fn process_adapter_passes_the_brief_by_stdin_and_streams_structured_events() {
    let mut adapter = adapter(
        "IFS= read -r brief; [ \"$brief\" = \"Build the bounded task.\" ] || exit 7; printf '%s\\n' '{\"sequence\":1,\"type\":\"activity\",\"summary\":\"Starting\"}' '{\"sequence\":2,\"type\":\"completed\",\"summary\":\"Ready for review\"}'",
    );

    let events = super::conformance_tests::assert_noninteractive_conformance(
        &mut adapter,
        2,
        "Ready for review",
    );

    assert_eq!(events.len(), 2);
    assert!(matches!(
        events[0].kind,
        NormalizedAgentEventKind::Activity { ref summary } if summary == "Starting"
    ));
    assert!(matches!(
        events[1].kind,
        NormalizedAgentEventKind::Completed { ref summary } if summary == "Ready for review"
    ));
}

#[test]
fn process_adapter_converts_malformed_or_out_of_order_output_into_a_failure_event() {
    let mut malformed = adapter("cat >/dev/null; printf '%s\\n' 'not valid JSON'");
    let malformed_session = malformed
        .start(super::conformance_tests::request())
        .expect("process should start");
    let malformed_events =
        super::conformance_tests::wait_for_events(&malformed, &malformed_session.id, 1);
    assert!(matches!(
        malformed_events[0].kind,
        NormalizedAgentEventKind::Failed { ref reason } if reason.contains("invalid JSON event")
    ));

    let mut out_of_order = adapter(
        "cat >/dev/null; printf '%s\\n' '{\"sequence\":2,\"type\":\"activity\",\"summary\":\"Skipped\"}'",
    );
    let out_of_order_session = out_of_order
        .start(super::conformance_tests::request())
        .expect("process should start");
    let out_of_order_events =
        super::conformance_tests::wait_for_events(&out_of_order, &out_of_order_session.id, 1);
    assert!(matches!(
        out_of_order_events[0].kind,
        NormalizedAgentEventKind::Failed { ref reason } if reason.contains("expected 1")
    ));
}

#[test]
fn process_adapter_reports_an_exit_without_a_terminal_event() {
    let mut adapter = adapter("cat >/dev/null");
    let session = adapter
        .start(super::conformance_tests::request())
        .expect("process should start");

    super::conformance_tests::wait_for_process_exit(
        &adapter,
        &session.id,
        "an exit without a terminal event must be actionable",
    );
}

#[test]
fn process_adapter_can_terminate_its_direct_child_process() {
    let mut adapter = adapter("cat >/dev/null; sleep 30");
    let session = adapter
        .start(super::conformance_tests::request())
        .expect("process should start");

    adapter
        .terminate(&session.id)
        .expect("direct child should terminate");
    super::conformance_tests::wait_for_process_exit(
        &adapter,
        &session.id,
        "terminated agent process should exit",
    );
}

#[test]
fn process_adapter_fails_closed_when_its_program_cannot_launch() {
    let mut adapter = ProcessAgentAdapter::new(ProcessAgentDefinition {
        name: "missing".to_owned(),
        program: "kanban-agent-program-that-does-not-exist".to_owned(),
        arguments: Vec::new(),
    });

    assert!(matches!(
        adapter.start(super::conformance_tests::request()),
        Err(AgentAdapterError::ProcessLaunch { .. })
    ));
}
