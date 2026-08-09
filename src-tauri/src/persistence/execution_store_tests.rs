use crate::{
    domain::{
        Evidence, EvidenceId, EvidenceKind, EvidenceResult, Execution, ExecutionId,
        ExecutionStatus, ExecutionUsage, SchemaMetadata, WorkItemId,
    },
    persistence::{EventStoreError, SqliteEventStore},
};
use tempfile::TempDir;

use super::sqlite_event_store_tests::create_command;

fn execution(id: &str, work_item_id: &str) -> Execution {
    Execution {
        schema: SchemaMetadata::current(),
        id: ExecutionId::from(id),
        work_item_id: WorkItemId::from(work_item_id),
        role: Default::default(),
        adapter_name: "codex-cli".to_owned(),
        status: ExecutionStatus::Pending,
        session_id: None,
        workspace_path: "/workspaces/task".to_owned(),
        usage: ExecutionUsage {
            input_tokens: 0,
            output_tokens: 0,
            cost_micros: None,
        },
        last_event_sequence: 0,
    }
}

fn evidence(id: &str, work_item_id: &str) -> Evidence {
    Evidence {
        schema: SchemaMetadata::current(),
        id: EvidenceId::from(id),
        work_item_id: WorkItemId::from(work_item_id),
        execution_id: None,
        kind: EvidenceKind::Check,
        result: EvidenceResult::Passed,
        summary: "The required checks passed.".to_owned(),
        recorded_at: "2026-08-08T00:00:00Z".to_owned(),
    }
}

#[test]
fn persists_idempotent_execution_and_evidence_records_for_board_tasks() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");
    store
        .create_work_item(create_command(
            "task-1",
            crate::domain::WorkItemState::Inbox,
        ))
        .expect("work item should persist");
    let execution = execution("execution-1", "task-1");
    let evidence = evidence("evidence-1", "task-1");

    assert_eq!(
        store
            .record_execution(execution.clone())
            .expect("execution should persist"),
        execution
    );
    assert_eq!(
        store
            .record_execution(execution.clone())
            .expect("matching execution should be idempotent"),
        execution
    );
    assert_eq!(
        store
            .record_evidence(evidence.clone())
            .expect("evidence should persist"),
        evidence
    );
    assert_eq!(
        store
            .record_evidence(evidence.clone())
            .expect("matching evidence should be idempotent"),
        evidence
    );
    assert_eq!(
        store
            .executions_for_work_items(&[WorkItemId::from("task-1")])
            .expect("execution records should load"),
        vec![execution]
    );
    assert_eq!(
        store
            .evidence_for_work_items(&[WorkItemId::from("task-1")])
            .expect("evidence records should load"),
        vec![evidence]
    );
}

#[test]
fn rejects_records_for_unknown_tasks_and_conflicting_identifiers() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");
    assert!(matches!(
        store.record_execution(execution("missing-execution", "missing-task")),
        Err(EventStoreError::WorkItemNotFound { .. })
    ));
    assert!(matches!(
        store.record_evidence(evidence("missing-evidence", "missing-task")),
        Err(EventStoreError::WorkItemNotFound { .. })
    ));

    store
        .create_work_item(create_command(
            "task-1",
            crate::domain::WorkItemState::Inbox,
        ))
        .expect("work item should persist");
    store
        .create_work_item(create_command(
            "task-2",
            crate::domain::WorkItemState::Inbox,
        ))
        .expect("second work item should persist");
    store
        .record_execution(execution("execution-1", "task-1"))
        .expect("execution should persist");
    store
        .record_evidence(evidence("evidence-1", "task-1"))
        .expect("evidence should persist");

    assert!(matches!(
        store.record_execution(execution("execution-1", "task-2")),
        Err(EventStoreError::ExecutionAlreadyExists { .. })
    ));
    assert!(matches!(
        store.record_evidence(evidence("evidence-1", "task-2")),
        Err(EventStoreError::EvidenceAlreadyExists { .. })
    ));
}

#[test]
fn returns_no_records_when_a_board_has_no_matching_work_items() {
    let store = SqliteEventStore::in_memory().expect("event store should open");

    assert!(
        store
            .executions_for_work_items(&[])
            .expect("empty execution query should succeed")
            .is_empty()
    );
    assert!(
        store
            .evidence_for_work_items(&[])
            .expect("empty evidence query should succeed")
            .is_empty()
    );
}

#[test]
fn retains_execution_and_evidence_records_after_reopening_the_database() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let database_path = temporary_directory.path().join("execution-store.sqlite");
    let mut store = SqliteEventStore::open(&database_path).expect("event store should open");
    store
        .create_work_item(create_command(
            "task-1",
            crate::domain::WorkItemState::Inbox,
        ))
        .expect("work item should persist");
    store
        .record_execution(execution("execution-1", "task-1"))
        .expect("execution should persist");
    store
        .record_evidence(evidence("evidence-1", "task-1"))
        .expect("evidence should persist");
    drop(store);

    let reopened_store = SqliteEventStore::open(&database_path).expect("event store should reopen");
    assert_eq!(
        reopened_store
            .executions_for_work_items(&[WorkItemId::from("task-1")])
            .expect("execution should remain available")
            .len(),
        1
    );
    assert_eq!(
        reopened_store
            .evidence_for_work_items(&[WorkItemId::from("task-1")])
            .expect("evidence should remain available")
            .len(),
        1
    );
}

#[test]
fn returns_bounded_recent_execution_and_evidence_records_without_discarding_history() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");
    store
        .create_work_item(create_command(
            "task-1",
            crate::domain::WorkItemState::Inbox,
        ))
        .expect("work item should persist");
    for index in 0..24 {
        store
            .record_execution(execution(&format!("execution-{index}"), "task-1"))
            .expect("execution should persist");
        store
            .record_evidence(evidence(&format!("evidence-{index}"), "task-1"))
            .expect("evidence should persist");
    }

    let task_id = WorkItemId::from("task-1");
    let execution_ids = store
        .recent_executions_for_work_items(std::slice::from_ref(&task_id), 20)
        .expect("recent executions should load")
        .into_iter()
        .map(|execution| execution.id.0)
        .collect::<Vec<_>>();
    let evidence_ids = store
        .recent_evidence_for_work_items(std::slice::from_ref(&task_id), 20)
        .expect("recent evidence should load")
        .into_iter()
        .map(|evidence| evidence.id.0)
        .collect::<Vec<_>>();

    assert_eq!(execution_ids.len(), 20);
    assert_eq!(execution_ids.first(), Some(&"execution-4".to_owned()));
    assert_eq!(execution_ids.last(), Some(&"execution-23".to_owned()));
    assert_eq!(evidence_ids.len(), 20);
    assert_eq!(evidence_ids.first(), Some(&"evidence-4".to_owned()));
    assert_eq!(evidence_ids.last(), Some(&"evidence-23".to_owned()));
    assert_eq!(
        store
            .executions_for_work_items(std::slice::from_ref(&task_id))
            .expect("complete execution history should load")
            .len(),
        24
    );
    assert_eq!(
        store
            .evidence_for_work_items(std::slice::from_ref(&task_id))
            .expect("complete evidence history should load")
            .len(),
        24
    );
    assert!(
        store
            .recent_executions_for_work_items(std::slice::from_ref(&task_id), 0)
            .expect("zero execution limit should succeed")
            .is_empty()
    );
    assert!(
        store
            .recent_evidence_for_work_items(std::slice::from_ref(&task_id), 0)
            .expect("zero evidence limit should succeed")
            .is_empty()
    );
}

#[test]
fn accepts_only_monotonic_execution_progress_with_a_stable_identity() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");
    store
        .create_work_item(create_command(
            "task-1",
            crate::domain::WorkItemState::Inbox,
        ))
        .expect("work item should persist");
    store
        .record_execution(execution("execution-1", "task-1"))
        .expect("pending execution should persist");

    let mut missing_session = execution("execution-1", "task-1");
    missing_session.status = ExecutionStatus::Running;
    assert!(matches!(
        store.update_execution(missing_session),
        Err(EventStoreError::InvalidExecutionUpdate {
            reason: "a running execution requires an attached session",
            ..
        })
    ));
    let mut running = execution("execution-1", "task-1");
    running.status = ExecutionStatus::Running;
    running.session_id = Some("session-1".to_owned());
    assert_eq!(
        store
            .update_execution(running.clone())
            .expect("pending execution should start"),
        running
    );

    let mut awaiting_input = running.clone();
    awaiting_input.last_event_sequence = 1;
    awaiting_input.usage.input_tokens = 5;
    assert_eq!(
        store
            .update_execution(awaiting_input.clone())
            .expect("running execution should retain usage progress"),
        awaiting_input
    );

    awaiting_input.status = ExecutionStatus::AwaitingInput;
    awaiting_input.last_event_sequence = 2;
    awaiting_input.usage.input_tokens = 10;
    awaiting_input.usage.output_tokens = 5;
    awaiting_input.usage.cost_micros = Some(100);
    assert_eq!(
        store
            .update_execution(awaiting_input.clone())
            .expect("running execution should wait for input"),
        awaiting_input
    );

    let mut invalid_status = awaiting_input.clone();
    invalid_status.status = ExecutionStatus::Pending;
    assert!(matches!(
        store.update_execution(invalid_status),
        Err(EventStoreError::InvalidExecutionUpdate {
            reason: "status transition is not allowed",
            ..
        })
    ));
    let mut changed_workspace = awaiting_input.clone();
    changed_workspace.workspace_path = "/workspaces/other".to_owned();
    assert!(matches!(
        store.update_execution(changed_workspace),
        Err(EventStoreError::InvalidExecutionUpdate {
            reason: "execution identity is immutable",
            ..
        })
    ));
    let mut lowered_usage = awaiting_input.clone();
    lowered_usage.usage.input_tokens = 9;
    assert!(matches!(
        store.update_execution(lowered_usage),
        Err(EventStoreError::InvalidExecutionUpdate {
            reason: "usage cannot decrease or discard recorded cost",
            ..
        })
    ));
    let mut blank_session = awaiting_input;
    blank_session.status = ExecutionStatus::Running;
    blank_session.session_id = Some(" ".to_owned());
    assert!(matches!(
        store.update_execution(blank_session),
        Err(EventStoreError::InvalidExecutionUpdate {
            reason: "session identity cannot be blank",
            ..
        })
    ));
}
