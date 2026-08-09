use std::{
    thread,
    time::{Duration, Instant},
};

use super::{
    AgentAdapter, AgentAdapterError, NormalizedAgentEvent, NormalizedAgentEventKind,
    StartAgentRequest,
};

pub(super) fn request() -> StartAgentRequest {
    StartAgentRequest {
        work_item_id: "work-item-1".to_owned(),
        workspace_path: std::env::current_dir()
            .expect("the test process should have a working directory")
            .display()
            .to_string(),
        task_brief: "Build the bounded task.".to_owned(),
    }
}

pub(super) fn assert_noninteractive_conformance(
    adapter: &mut impl AgentAdapter,
    event_count: usize,
    completed_summary: &str,
) -> Vec<NormalizedAgentEvent> {
    let capabilities = adapter
        .discover()
        .expect("capabilities should be available before start");
    assert!(capabilities.streams_structured_events);
    assert!(!capabilities.supports_feedback);
    assert!(!capabilities.supports_interrupt);
    assert!(!capabilities.supports_resume);

    let session = adapter.start(request()).expect("process should start");
    assert!(!session.resumable);
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
            capability: "process-tree interruption"
        })
    ));

    let events = wait_for_events(adapter, &session.id, event_count);
    assert!(matches!(
        events.last().map(|event| &event.kind),
        Some(NormalizedAgentEventKind::Completed { summary }) if summary == completed_summary
    ));
    assert_eq!(
        adapter.stream_events(&session.id, 1),
        Ok(events[1..].to_vec())
    );
    adapter
        .health_check(&session.id)
        .expect("a terminal lifecycle event should make a clean exit healthy");
    events
}

pub(super) fn wait_for_events(
    adapter: &impl AgentAdapter,
    session_id: &str,
    count: usize,
) -> Vec<NormalizedAgentEvent> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let events = adapter
            .stream_events(session_id, 0)
            .expect("the session should be readable");
        if events.len() >= count {
            return events;
        }
        if Instant::now() >= deadline {
            panic!("the process did not emit {count} lifecycle events in time");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

pub(super) fn wait_for_process_exit(adapter: &impl AgentAdapter, session_id: &str, failure: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if matches!(
            adapter.health_check(session_id),
            Err(AgentAdapterError::ProcessExited { .. })
        ) {
            return;
        }
        if Instant::now() >= deadline {
            panic!("{failure}");
        }
        thread::sleep(Duration::from_millis(10));
    }
}
