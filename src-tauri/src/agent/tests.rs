use std::error::Error;

use super::{
    AgentAdapter, AgentAdapterError, AgentCapabilities, AgentEventIngestor, FakeAgentAdapter,
    NormalizedAgentEvent, NormalizedAgentEventKind, StartAgentRequest,
};
use crate::domain::{TransitionConfig, WorkItemState};

const ALL_CAPABILITIES: AgentCapabilities = AgentCapabilities {
    supports_feedback: true,
    supports_interrupt: true,
    supports_resume: true,
    streams_structured_events: true,
};

fn request() -> StartAgentRequest {
    StartAgentRequest {
        work_item_id: "work-item-1".to_owned(),
        workspace_path: "/workspaces/work-item-1".to_owned(),
        task_brief: "Add the adapter contract.".to_owned(),
    }
}

fn event(sequence: u64, kind: NormalizedAgentEventKind) -> NormalizedAgentEvent {
    NormalizedAgentEvent { sequence, kind }
}

#[test]
fn fake_adapter_discovers_capabilities_and_runs_a_resumable_session() {
    let mut adapter = FakeAgentAdapter::new("fake", ALL_CAPABILITIES);

    assert_eq!(adapter.name(), "fake");
    assert_eq!(adapter.discover(), Ok(ALL_CAPABILITIES));
    let session = adapter.start(request()).expect("session should start");
    assert_eq!(session.id, "fake-session-1");
    assert!(session.resumable);
    assert_eq!(adapter.resume(&session.id), Ok(session.clone()));
    adapter
        .send_feedback(&session.id, "Please add a regression test.")
        .expect("feedback should be supported");
    let expected_feedback = ["Please add a regression test.".to_owned()];
    assert_eq!(
        adapter.feedback(&session.id),
        Ok(expected_feedback.as_slice())
    );
    adapter
        .interrupt(&session.id)
        .expect("interruption should be supported");
    adapter
        .health_check(&session.id)
        .expect("an interrupted session can still report health");
    adapter
        .terminate(&session.id)
        .expect("termination should be available");
    assert!(matches!(
        adapter.health_check(&session.id),
        Err(AgentAdapterError::CapabilityUnsupported { .. })
    ));
}

#[test]
fn fake_adapter_reports_unsupported_capabilities_and_unknown_sessions() {
    let mut adapter = FakeAgentAdapter::new("limited", AgentCapabilities::default());
    let session = adapter.start(request()).expect("session should start");

    assert!(matches!(
        adapter.resume(&session.id),
        Err(AgentAdapterError::CapabilityUnsupported {
            capability: "resume"
        })
    ));
    assert!(matches!(
        adapter.send_feedback(&session.id, "feedback"),
        Err(AgentAdapterError::CapabilityUnsupported {
            capability: "feedback"
        })
    ));
    assert!(matches!(
        adapter.interrupt(&session.id),
        Err(AgentAdapterError::CapabilityUnsupported {
            capability: "interrupt"
        })
    ));
    assert!(matches!(
        adapter.stream_events("missing", 0),
        Err(AgentAdapterError::UnknownSession { .. })
    ));
    assert!(matches!(
        adapter.terminate("missing"),
        Err(AgentAdapterError::UnknownSession { .. })
    ));
}

#[test]
fn fake_adapter_streams_normalized_events() {
    let mut adapter = FakeAgentAdapter::new("fake", ALL_CAPABILITIES);
    let session = adapter.start(request()).expect("session should start");
    let events = [
        event(
            1,
            NormalizedAgentEventKind::ApprovalRequested {
                question: "May I modify the lockfile?".to_owned(),
            },
        ),
        event(
            2,
            NormalizedAgentEventKind::Completed {
                summary: "Implementation is ready for review.".to_owned(),
            },
        ),
        event(
            3,
            NormalizedAgentEventKind::Failed {
                reason: "Check failed.".to_owned(),
            },
        ),
        event(
            4,
            NormalizedAgentEventKind::Interrupted {
                reason: "Process exited unexpectedly.".to_owned(),
            },
        ),
    ];

    for event in events.clone() {
        adapter
            .queue_event(&session.id, event)
            .expect("event should be queued");
    }

    assert_eq!(adapter.stream_events(&session.id, 0), Ok(events.to_vec()));
    assert_eq!(
        adapter.stream_events(&session.id, 2),
        Ok(events[2..].to_vec())
    );
}

#[test]
fn event_ingestor_maps_provider_events_to_guarded_nonterminal_work_item_states() {
    let mut ingestor = AgentEventIngestor::new(0);
    let config = TransitionConfig::default();

    let awaiting_input = ingestor
        .apply_to_work_item(
            WorkItemState::Running,
            &event(
                1,
                NormalizedAgentEventKind::AwaitingInput {
                    question: "Which API version should I use?".to_owned(),
                },
            ),
            config,
        )
        .expect("input request should be guarded");
    assert_eq!(awaiting_input, WorkItemState::AwaitingInput);
    let review = ingestor
        .apply_to_work_item(
            WorkItemState::Running,
            &event(
                2,
                NormalizedAgentEventKind::Completed {
                    summary: "Ready.".to_owned(),
                },
            ),
            config,
        )
        .expect("completion should require review");
    assert_eq!(review, WorkItemState::Review);
    let failed = ingestor
        .apply_to_work_item(
            WorkItemState::Running,
            &event(
                3,
                NormalizedAgentEventKind::Failed {
                    reason: "Compilation failed.".to_owned(),
                },
            ),
            config,
        )
        .expect("failure should stay distinct from completion");
    assert_eq!(failed, WorkItemState::Failed);
    let interrupted = ingestor
        .apply_to_work_item(
            WorkItemState::Running,
            &event(
                4,
                NormalizedAgentEventKind::Interrupted {
                    reason: "Stopped.".to_owned(),
                },
            ),
            config,
        )
        .expect("interruption should stay distinct from completion");
    assert_eq!(interrupted, WorkItemState::Interrupted);
    assert_eq!(ingestor.last_sequence(), 4);
}

#[test]
fn event_ingestor_rejects_duplicates_out_of_order_events_and_illegal_transitions() {
    let mut ingestor = AgentEventIngestor::new(0);
    let activity = event(
        1,
        NormalizedAgentEventKind::Activity {
            summary: "Reading files.".to_owned(),
        },
    );

    assert_eq!(
        ingestor.apply_to_work_item(
            WorkItemState::Running,
            &activity,
            TransitionConfig::default()
        ),
        Ok(WorkItemState::Running)
    );
    assert_eq!(
        ingestor.apply_to_work_item(
            WorkItemState::Running,
            &activity,
            TransitionConfig::default()
        ),
        Err(AgentAdapterError::DuplicateEvent { sequence: 1 })
    );
    assert_eq!(
        ingestor.apply_to_work_item(
            WorkItemState::Running,
            &event(
                3,
                NormalizedAgentEventKind::UsageUpdated {
                    input_tokens: 10,
                    output_tokens: 20,
                    cost_micros: None
                }
            ),
            TransitionConfig::default()
        ),
        Err(AgentAdapterError::EventOutOfOrder {
            expected: 2,
            received: 3
        })
    );
    let completed = event(
        2,
        NormalizedAgentEventKind::Completed {
            summary: "Ready.".to_owned(),
        },
    );
    assert!(matches!(
        ingestor.apply_to_work_item(
            WorkItemState::Ready,
            &completed,
            TransitionConfig::default()
        ),
        Err(AgentAdapterError::InvalidWorkItemTransition(_))
    ));
    assert_eq!(ingestor.last_sequence(), 1);
    assert_eq!(
        ingestor.apply_to_work_item(
            WorkItemState::Running,
            &completed,
            TransitionConfig::default()
        ),
        Ok(WorkItemState::Review)
    );
}

#[test]
fn adapter_errors_explain_actionable_failures_and_preserve_wrapped_sources() {
    let transition_error = AgentAdapterError::InvalidWorkItemTransition(
        crate::domain::TransitionError::IncompleteEvidence,
    );

    assert!(
        transition_error
            .to_string()
            .contains("cannot change the work item")
    );
    assert!(transition_error.source().is_some());
    assert_eq!(
        AgentAdapterError::DuplicateEvent { sequence: 4 }.to_string(),
        "agent event sequence 4 was already processed"
    );
    assert!(
        AgentAdapterError::UnknownSession {
            session_id: "missing".to_owned()
        }
        .source()
        .is_none()
    );
}
