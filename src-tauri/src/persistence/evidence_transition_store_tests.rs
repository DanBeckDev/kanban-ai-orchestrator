use crate::{
    domain::{
        Evidence, EvidenceId, EvidenceKind, EvidenceResult, SchemaMetadata, WorkItemId,
        WorkItemState,
    },
    persistence::{EventStoreError, SqliteEventStore},
};

use super::sqlite_event_store_tests::{create_command, transition_command};

#[test]
fn rolls_back_failed_review_evidence_when_its_remediation_transition_cannot_persist() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");
    store
        .create_work_item(create_command("task-1", WorkItemState::Inbox))
        .expect("work item should persist");
    store
        .connection
        .execute_batch(
            "CREATE TRIGGER reject_remediation_transition
             BEFORE INSERT ON work_item_events
             BEGIN
                 SELECT RAISE(ABORT, 'simulated transition failure');
             END",
        )
        .expect("test trigger should install");
    let evidence = failed_review_evidence("failed-clean-code-review", "task-1");

    assert!(matches!(
        store.record_evidence_and_transition(
            evidence.clone(),
            transition_command("return-task-1-to-ready", "task-1", WorkItemState::Planned),
        ),
        Err(EventStoreError::Database(_))
    ));
    assert!(
        store
            .evidence(&evidence.id)
            .expect("evidence lookup should succeed")
            .is_none()
    );
    assert_eq!(
        store
            .materialized_work_item(&WorkItemId::from("task-1"))
            .expect("work item lookup should succeed")
            .expect("work item should remain materialized")
            .work_item
            .state,
        WorkItemState::Inbox
    );
}

#[test]
fn transitions_with_matching_evidence_that_was_already_recorded() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");
    store
        .create_work_item(create_command("task-1", WorkItemState::Inbox))
        .expect("work item should persist");
    let evidence = failed_review_evidence("pre-recorded-clean-code-review", "task-1");
    store
        .record_evidence(evidence.clone())
        .expect("evidence should persist");

    let event = store
        .record_evidence_and_transition(
            evidence.clone(),
            transition_command("plan-task-1", "task-1", WorkItemState::Planned),
        )
        .expect("matching evidence and transition should persist together");
    assert_eq!(event.event.work_item_id, WorkItemId::from("task-1"));
    assert_eq!(
        store
            .evidence(&evidence.id)
            .expect("evidence lookup should succeed"),
        Some(evidence)
    );
}

#[test]
fn rejects_evidence_that_does_not_belong_to_the_transition_work_item() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");
    for work_item_id in ["task-1", "task-2"] {
        store
            .create_work_item(create_command(work_item_id, WorkItemState::Inbox))
            .expect("work item should persist");
    }
    let evidence = failed_review_evidence("mismatched-clean-code-review", "task-1");

    assert!(matches!(
        store.record_evidence_and_transition(
            evidence.clone(),
            transition_command("plan-task-2", "task-2", WorkItemState::Planned),
        ),
        Err(EventStoreError::EvidenceWorkItemMismatch { .. })
    ));
    assert!(
        store
            .evidence(&evidence.id)
            .expect("evidence lookup should succeed")
            .is_none()
    );
}

fn failed_review_evidence(id: &str, work_item_id: &str) -> Evidence {
    Evidence {
        schema: SchemaMetadata::current(),
        id: EvidenceId::from(id),
        work_item_id: WorkItemId::from(work_item_id),
        execution_id: None,
        kind: EvidenceKind::CleanCodeReview,
        result: EvidenceResult::Failed,
        summary: "Two actionable findings require remediation.".to_owned(),
        recorded_at: "2026-08-08T00:00:00Z".to_owned(),
    }
}
