use crate::{
    domain::{
        CompletionEvidence, CreateWorkItemCommand, TransitionConfig, WorkItemEventId, WorkItemState,
    },
    persistence::{EventStoreError, SqliteEventStore},
};

use super::sqlite_event_store_tests::{create_command, transition_command, work_item};

#[test]
fn rejects_duplicate_work_items_and_every_conflicting_transition_replay() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");
    store
        .create_work_item(create_command("task-1", WorkItemState::Inbox))
        .expect("work item should persist");
    let duplicate_work_item = CreateWorkItemCommand {
        event_id: WorkItemEventId::from("different-create-event"),
        work_item: work_item("task-1", WorkItemState::Inbox),
        recorded_at: "2026-08-08T00:00:00Z".to_owned(),
    };
    store
        .transition_work_item(transition_command(
            "plan-task-1",
            "task-1",
            WorkItemState::Planned,
        ))
        .expect("transition should persist");
    let mut mismatched_config = transition_command("plan-task-1", "task-1", WorkItemState::Planned);
    mismatched_config.config = TransitionConfig {
        human_review_required: true,
    };
    let mut mismatched_evidence =
        transition_command("plan-task-1", "task-1", WorkItemState::Planned);
    mismatched_evidence.evidence = Some(CompletionEvidence {
        checks_passed: true,
        completion_report_present: true,
        review_accepted: true,
    });
    let mut mismatched_reason = transition_command("plan-task-1", "task-1", WorkItemState::Planned);
    mismatched_reason.reason = "A different audit reason.".to_owned();

    assert!(matches!(
        store.create_work_item(duplicate_work_item),
        Err(EventStoreError::WorkItemAlreadyExists { .. })
    ));
    for command in [
        transition_command("plan-task-1", "task-2", WorkItemState::Planned),
        transition_command("plan-task-1", "task-1", WorkItemState::Ready),
        mismatched_config,
        mismatched_evidence,
        mismatched_reason,
    ] {
        assert!(matches!(
            store.transition_work_item(command),
            Err(EventStoreError::EventIdConflict { .. })
        ));
    }
}
