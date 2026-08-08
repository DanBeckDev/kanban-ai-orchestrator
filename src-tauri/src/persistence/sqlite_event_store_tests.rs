use std::collections::{BTreeMap, BTreeSet};

use tempfile::TempDir;

use crate::{
    domain::{
        BoardId, CompletionEvidence, CreateWorkItemCommand, PolicyAction, PolicyDecision,
        PolicyDecisionId, PolicyDecisionKind, ProjectId, RestartReconciliationCommand,
        SchemaMetadata, TransitionConfig, TransitionWorkItemCommand, VersionedSchema, WorkItem,
        WorkItemBudget, WorkItemEventId, WorkItemEventKind, WorkItemId, WorkItemState,
    },
    persistence::{EventStoreError, SqliteEventStore},
    policy::{ProtectedGitAction, ProtectedGitApproval},
};

pub(super) fn work_item(id: &str, state: WorkItemState) -> WorkItem {
    WorkItem {
        schema: SchemaMetadata::current(),
        id: WorkItemId::from(id),
        board_id: BoardId::from("board-1"),
        title: format!("Task {id}"),
        description: "A persisted task.".to_owned(),
        acceptance_criteria: vec!["The task is recoverable.".to_owned()],
        budget: WorkItemBudget::default(),
        state,
        requires_human_review: false,
    }
}

pub(super) fn create_command(id: &str, state: WorkItemState) -> CreateWorkItemCommand {
    CreateWorkItemCommand {
        event_id: WorkItemEventId::from(format!("create-{id}").as_str()),
        work_item: work_item(id, state),
        recorded_at: "2026-08-08T00:00:00Z".to_owned(),
    }
}

pub(super) fn transition_command(
    event_id: &str,
    work_item_id: &str,
    next_state: WorkItemState,
) -> TransitionWorkItemCommand {
    TransitionWorkItemCommand {
        event_id: WorkItemEventId::from(event_id),
        work_item_id: WorkItemId::from(work_item_id),
        next_state,
        config: TransitionConfig::default(),
        evidence: None,
        reason: "The daemon accepted the lifecycle update.".to_owned(),
        recorded_at: "2026-08-08T00:00:00Z".to_owned(),
    }
}

fn create_running_work_item(store: &mut SqliteEventStore, id: &str) {
    store
        .create_work_item(create_command(id, WorkItemState::Inbox))
        .expect("work item creation should succeed");
    store
        .transition_work_item(transition_command("plan", id, WorkItemState::Planned))
        .expect("planning transition should succeed");
    store
        .transition_work_item(transition_command("ready", id, WorkItemState::Ready))
        .expect("ready transition should succeed");
    store
        .transition_work_item(transition_command("run", id, WorkItemState::Running))
        .expect("running transition should succeed");
}

pub(super) fn policy_decision(
    id: &str,
    decision: PolicyDecisionKind,
    actor: &str,
) -> PolicyDecision {
    PolicyDecision {
        schema: SchemaMetadata::current(),
        id: PolicyDecisionId::from(id),
        project_id: ProjectId::from("project-1"),
        work_item_id: Some(WorkItemId::from("task-1")),
        action: Some(PolicyAction::ProtectedGit {
            action: ProtectedGitAction::Push,
        }),
        decision,
        actor: actor.to_owned(),
        input_summary: "action=protected_git:push".to_owned(),
        outcome_summary: "A policy decision was recorded.".to_owned(),
        reason: "The policy engine evaluated this request.".to_owned(),
        decided_at: "2026-08-08T14:00:00Z".to_owned(),
    }
}

pub(super) fn protected_git_approval(decision_id: &str, actor: &str) -> ProtectedGitApproval {
    ProtectedGitApproval {
        decision_id: PolicyDecisionId::from(decision_id),
        project_id: ProjectId::from("project-1"),
        work_item_id: Some(WorkItemId::from("task-1")),
        action: ProtectedGitAction::Push,
        approved_by: actor.to_owned(),
        approved_at: "2026-08-08T14:01:00Z".to_owned(),
    }
}

#[test]
fn persists_an_append_only_event_history_and_materialized_snapshot_across_reopen() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let database_path = temporary_directory.path().join("event-store.sqlite");
    let mut store = SqliteEventStore::open(&database_path).expect("event store should open");

    let created = store
        .create_work_item(create_command("task-1", WorkItemState::Inbox))
        .expect("work item creation should persist");
    let planned = store
        .transition_work_item(transition_command(
            "plan-task-1",
            "task-1",
            WorkItemState::Planned,
        ))
        .expect("planning transition should persist");

    assert_eq!(
        store
            .database_schema_version()
            .expect("database schema version should load"),
        6
    );
    assert_eq!(created.sequence, 1);
    assert_eq!(planned.sequence, 2);
    drop(store);

    let reopened_store = SqliteEventStore::open(&database_path).expect("event store should reopen");
    let snapshot = reopened_store
        .materialized_work_item(&WorkItemId::from("task-1"))
        .expect("snapshot should load")
        .expect("snapshot should exist");
    let events = reopened_store
        .work_item_events(&WorkItemId::from("task-1"))
        .expect("event history should load");

    assert_eq!(snapshot.work_item.state, WorkItemState::Planned);
    assert_eq!(snapshot.last_event_sequence, 2);
    assert_eq!(events.len(), 2);
    assert!(events[0].event.uses_current_schema());
    assert!(matches!(
        events[0].event.kind,
        WorkItemEventKind::Created { .. }
    ));
    assert!(matches!(
        events[1].event.kind,
        WorkItemEventKind::StateTransitioned {
            from: WorkItemState::Inbox,
            to: WorkItemState::Planned,
            ..
        }
    ));
}

#[test]
fn repeats_the_same_command_idempotently_but_rejects_conflicting_reuse() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");
    let command = create_command("task-1", WorkItemState::Inbox);

    let first_event = store
        .create_work_item(command.clone())
        .expect("first command should persist");
    let repeated_event = store
        .create_work_item(command)
        .expect("identical command should be idempotent");
    let conflicting_command = CreateWorkItemCommand {
        event_id: WorkItemEventId::from("create-task-1"),
        work_item: work_item("task-2", WorkItemState::Inbox),
        recorded_at: "2026-08-08T00:00:00Z".to_owned(),
    };

    assert_eq!(first_event, repeated_event);
    assert!(matches!(
        store.create_work_item(conflicting_command),
        Err(EventStoreError::EventIdConflict { .. })
    ));
    assert_eq!(
        store
            .work_item_events(&WorkItemId::from("task-1"))
            .expect("events should load")
            .len(),
        1
    );
}

#[test]
fn rejects_invalid_transitions_without_mutating_history_or_snapshot() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");
    store
        .create_work_item(create_command("task-1", WorkItemState::Inbox))
        .expect("work item creation should persist");
    let invalid_completion = TransitionWorkItemCommand {
        next_state: WorkItemState::Done,
        evidence: Some(CompletionEvidence {
            checks_passed: true,
            completion_report_present: true,
            review_accepted: true,
        }),
        ..transition_command("complete-task-1", "task-1", WorkItemState::Done)
    };

    assert!(matches!(
        store.transition_work_item(invalid_completion),
        Err(EventStoreError::StateTransition(_))
    ));
    assert_eq!(
        store
            .materialized_work_item(&WorkItemId::from("task-1"))
            .expect("snapshot should load")
            .expect("snapshot should exist")
            .work_item
            .state,
        WorkItemState::Inbox
    );
    assert_eq!(
        store
            .work_item_events(&WorkItemId::from("task-1"))
            .expect("events should load")
            .len(),
        1
    );
}

#[test]
fn restart_reconciliation_preserves_history_and_interrupts_unconfirmed_work() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let database_path = temporary_directory.path().join("event-store.sqlite");
    let mut store = SqliteEventStore::open(&database_path).expect("event store should open");
    create_running_work_item(&mut store, "task-1");
    drop(store);

    let mut recovered_store =
        SqliteEventStore::open(&database_path).expect("event store should reopen");
    let recovered_events = recovered_store
        .reconcile_after_restart(RestartReconciliationCommand {
            confirmed_active_work_item_ids: BTreeSet::new(),
            recovery_event_ids: BTreeMap::from([(
                WorkItemId::from("task-1"),
                WorkItemEventId::from("recover-task-1"),
            )]),
            recorded_at: "2026-08-08T00:01:00Z".to_owned(),
        })
        .expect("restart reconciliation should persist interruption");

    assert_eq!(recovered_events.len(), 1);
    assert_eq!(recovered_events[0].sequence, 5);
    assert!(matches!(
        recovered_events[0].event.kind,
        WorkItemEventKind::StateTransitioned {
            from: WorkItemState::Running,
            to: WorkItemState::Interrupted,
            ..
        }
    ));
    assert_eq!(
        recovered_store
            .materialized_work_item(&WorkItemId::from("task-1"))
            .expect("snapshot should load")
            .expect("snapshot should exist")
            .work_item
            .state,
        WorkItemState::Interrupted
    );
    assert_eq!(
        recovered_store
            .work_item_events(&WorkItemId::from("task-1"))
            .expect("events should load")
            .len(),
        5
    );
}

#[test]
fn recovery_keeps_confirmed_running_work_and_requires_all_recovery_event_ids() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");
    create_running_work_item(&mut store, "confirmed-task");
    store
        .create_work_item(create_command(
            "uncertain-task",
            WorkItemState::AwaitingInput,
        ))
        .expect("awaiting-input work item should persist");

    assert!(matches!(
        store.reconcile_after_restart(RestartReconciliationCommand {
            confirmed_active_work_item_ids: BTreeSet::from([WorkItemId::from("confirmed-task")]),
            recovery_event_ids: BTreeMap::new(),
            recorded_at: "2026-08-08T00:01:00Z".to_owned(),
        }),
        Err(EventStoreError::MissingRecoveryEventId { .. })
    ));
    assert_eq!(
        store
            .materialized_work_item(&WorkItemId::from("confirmed-task"))
            .expect("snapshot should load")
            .expect("snapshot should exist")
            .work_item
            .state,
        WorkItemState::Running
    );
    assert_eq!(
        store
            .materialized_work_item(&WorkItemId::from("uncertain-task"))
            .expect("snapshot should load")
            .expect("snapshot should exist")
            .work_item
            .state,
        WorkItemState::AwaitingInput
    );
}

#[test]
fn reopening_for_a_ui_reconnect_does_not_change_state_without_reconciliation() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let database_path = temporary_directory.path().join("event-store.sqlite");
    let mut store = SqliteEventStore::open(&database_path).expect("event store should open");
    store
        .create_work_item(create_command("task-1", WorkItemState::Review))
        .expect("review work item should persist");
    drop(store);

    let reconnected_store =
        SqliteEventStore::open(&database_path).expect("event store should reopen");

    assert_eq!(
        reconnected_store
            .materialized_work_item(&WorkItemId::from("task-1"))
            .expect("snapshot should load")
            .expect("snapshot should exist")
            .work_item
            .state,
        WorkItemState::Review
    );
    assert_eq!(
        reconnected_store
            .work_item_events(&WorkItemId::from("task-1"))
            .expect("events should load")
            .len(),
        1
    );
}

#[test]
fn validates_transition_commands_and_repeats_matching_transitions_idempotently() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");
    store
        .create_work_item(create_command("task-1", WorkItemState::Inbox))
        .expect("work item creation should persist");

    assert!(matches!(
        store.transition_work_item(transition_command(
            "missing-task",
            "missing",
            WorkItemState::Planned
        )),
        Err(EventStoreError::WorkItemNotFound { .. })
    ));
    let mut blank_reason = transition_command("blank-reason", "task-1", WorkItemState::Planned);
    blank_reason.reason = " ".to_owned();
    assert!(matches!(
        store.transition_work_item(blank_reason),
        Err(EventStoreError::MissingTransitionReason { .. })
    ));
    let command = transition_command("plan-task-1", "task-1", WorkItemState::Planned);
    let first_event = store
        .transition_work_item(command.clone())
        .expect("planning transition should persist");
    let repeated_event = store
        .transition_work_item(command)
        .expect("matching transition should be idempotent");
    let conflicting_command = transition_command("plan-task-1", "task-1", WorkItemState::Ready);

    assert_eq!(first_event, repeated_event);
    assert!(matches!(
        store.transition_work_item(conflicting_command),
        Err(EventStoreError::EventIdConflict { .. })
    ));
    assert_eq!(
        store
            .materialized_work_item(&WorkItemId::from("task-1"))
            .expect("snapshot should load")
            .expect("snapshot should exist")
            .work_item
            .state,
        WorkItemState::Planned
    );
}

#[test]
fn reconciliation_leaves_non_uncertain_states_unchanged() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");
    store
        .create_work_item(create_command("review-task", WorkItemState::Review))
        .expect("review work item should persist");

    assert!(
        store
            .reconcile_after_restart(RestartReconciliationCommand {
                confirmed_active_work_item_ids: BTreeSet::new(),
                recovery_event_ids: BTreeMap::new(),
                recorded_at: "2026-08-08T00:01:00Z".to_owned(),
            })
            .expect("reconciliation should succeed")
            .is_empty()
    );
    assert_eq!(
        store
            .materialized_work_item(&WorkItemId::from("review-task"))
            .expect("snapshot should load")
            .expect("snapshot should exist")
            .work_item
            .state,
        WorkItemState::Review
    );
}
