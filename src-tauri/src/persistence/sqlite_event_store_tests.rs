use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
};

use tempfile::TempDir;

use crate::{
    domain::{
        BoardId, CompletionEvidence, CreateWorkItemCommand, PolicyAction, PolicyDecision,
        PolicyDecisionId, PolicyDecisionKind, ProjectId, RestartReconciliationCommand,
        SchemaMetadata, TransitionConfig, TransitionWorkItemCommand, VersionedSchema, WorkItem,
        WorkItemBudget, WorkItemEventId, WorkItemEventKind, WorkItemId, WorkItemState,
    },
    persistence::{EventStoreError, SqliteEventStore},
    policy::{
        PolicyGate, PolicyLimits, PolicyRequest, PolicySet, PolicyUsage, ProtectedGitAction,
        ProtectedGitApproval,
    },
};

fn work_item(id: &str, state: WorkItemState) -> WorkItem {
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

fn create_command(id: &str, state: WorkItemState) -> CreateWorkItemCommand {
    CreateWorkItemCommand {
        event_id: WorkItemEventId::from(format!("create-{id}").as_str()),
        work_item: work_item(id, state),
        recorded_at: "2026-08-08T00:00:00Z".to_owned(),
    }
}

fn transition_command(
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

fn policy_decision(id: &str, decision: PolicyDecisionKind, actor: &str) -> PolicyDecision {
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

fn protected_git_approval(decision_id: &str, actor: &str) -> ProtectedGitApproval {
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
        3
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

#[test]
fn rejects_databases_created_by_a_newer_schema_version() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let database_path = temporary_directory.path().join("newer-schema.sqlite");
    let connection = rusqlite::Connection::open(&database_path)
        .expect("database with a future schema version should be created");
    connection
        .execute_batch(
            "CREATE TABLE schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            INSERT INTO schema_migrations (version, applied_at)
            VALUES (4, '2026-08-08T00:00:00Z');",
        )
        .expect("future schema version should be recorded");
    drop(connection);

    assert!(matches!(
        SqliteEventStore::open(&database_path),
        Err(EventStoreError::UnsupportedDatabaseSchemaVersion {
            current: 4,
            supported: 3,
        })
    ));
}

#[test]
fn migrates_existing_event_stores_to_the_policy_audit_schema() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let database_path = temporary_directory.path().join("version-one.sqlite");
    let connection =
        rusqlite::Connection::open(&database_path).expect("version-one database should be created");
    connection
        .execute_batch(
            "CREATE TABLE schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            INSERT INTO schema_migrations (version, applied_at)
            VALUES (1, '2026-08-08T00:00:00Z');
            CREATE TABLE work_item_events (
                sequence INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id TEXT NOT NULL UNIQUE,
                work_item_id TEXT NOT NULL,
                event_json TEXT NOT NULL
            );
            CREATE TABLE materialized_work_items (
                work_item_id TEXT PRIMARY KEY,
                work_item_json TEXT NOT NULL,
                last_event_sequence INTEGER NOT NULL
            );",
        )
        .expect("version-one tables should be created");
    drop(connection);

    let mut store = SqliteEventStore::open(&database_path).expect("store should migrate");
    let decision = policy_decision(
        "migrated-policy-decision",
        PolicyDecisionKind::Deny,
        "daemon",
    );

    assert_eq!(
        store
            .database_schema_version()
            .expect("schema version should load"),
        3
    );
    assert_eq!(
        store
            .record_policy_decision(decision.clone())
            .expect("migrated table should accept policy decisions"),
        decision
    );
}

#[test]
fn persists_policy_audit_records_and_verifies_protected_git_approvals() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let database_path = temporary_directory.path().join("policy-audit.sqlite");
    let mut store = SqliteEventStore::open(&database_path).expect("event store should open");
    let denied = policy_decision("deny-network", PolicyDecisionKind::Deny, "worker-agent-1");
    let approval_required = policy_decision(
        "push-needs-approval",
        PolicyDecisionKind::ApprovalRequired,
        "worker-agent-1",
    );
    let user_approval = policy_decision("user-approved-push", PolicyDecisionKind::Allow, "Daniel");
    let approval = protected_git_approval("user-approved-push", "Daniel");

    assert_eq!(
        store
            .record_policy_decision(denied.clone())
            .expect("deny decision should persist"),
        denied
    );
    assert_eq!(
        store
            .record_policy_decision(approval_required.clone())
            .expect("approval-required decision should persist"),
        approval_required
    );
    assert_eq!(
        store
            .record_policy_decision(user_approval.clone())
            .expect("user approval decision should persist"),
        user_approval
    );
    assert_eq!(
        store
            .record_policy_decision(user_approval.clone())
            .expect("identical policy decision should be idempotent"),
        user_approval
    );
    let mut conflicting_decision = user_approval.clone();
    conflicting_decision.actor = "Someone else".to_owned();
    assert!(matches!(
        store.record_policy_decision(conflicting_decision),
        Err(EventStoreError::PolicyDecisionIdConflict { .. })
    ));
    assert!(matches!(
        store.record_protected_git_approval(protected_git_approval("missing", "Daniel")),
        Err(EventStoreError::PolicyApprovalDecisionNotFound { .. })
    ));
    assert!(matches!(
        store.record_protected_git_approval(protected_git_approval("deny-network", "Daniel")),
        Err(EventStoreError::PolicyApprovalDecisionMismatch { .. })
    ));
    let mut wrong_action_decision =
        policy_decision("user-approved-commit", PolicyDecisionKind::Allow, "Daniel");
    wrong_action_decision.action = Some(PolicyAction::ProtectedGit {
        action: ProtectedGitAction::Commit,
    });
    store
        .record_policy_decision(wrong_action_decision.clone())
        .expect("different Git approval decision should persist");
    assert!(matches!(
        store.record_protected_git_approval(protected_git_approval(
            "user-approved-commit",
            "Daniel"
        )),
        Err(EventStoreError::PolicyApprovalDecisionMismatch { .. })
    ));
    assert_eq!(
        store
            .record_protected_git_approval(approval.clone())
            .expect("approval backed by a user decision should persist"),
        approval
    );
    assert!(
        store
            .has_recorded_protected_git_approval(&approval)
            .expect("approval lookup should succeed")
    );
    let mut conflicting_approval = approval.clone();
    conflicting_approval.approved_at = "2026-08-08T14:02:00Z".to_owned();
    assert!(matches!(
        store.record_protected_git_approval(conflicting_approval),
        Err(EventStoreError::ProtectedGitApprovalIdConflict { .. })
    ));
    drop(store);

    let reopened_store = SqliteEventStore::open(&database_path).expect("event store should reopen");
    assert_eq!(
        reopened_store
            .policy_decisions_for_project(&ProjectId::from("project-1"))
            .expect("policy decisions should reopen"),
        vec![
            denied,
            approval_required,
            wrong_action_decision,
            user_approval
        ]
    );
    assert!(
        reopened_store
            .has_recorded_protected_git_approval(&approval)
            .expect("approval should reopen")
    );
}

#[test]
fn policy_gate_issues_a_capability_only_after_sqlite_verifies_the_approval() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");
    let approval_decision =
        policy_decision("user-approved-push", PolicyDecisionKind::Allow, "Daniel");
    let approval = protected_git_approval("user-approved-push", "Daniel");
    let gate = PolicyGate::new(PolicySet {
        limits: PolicyLimits {
            max_parallel_executions: 1,
            ..Default::default()
        },
        protected_git_actions: BTreeSet::from([ProtectedGitAction::Push]),
        ..Default::default()
    });
    let request = PolicyRequest {
        decision_id: PolicyDecisionId::from("worker-push"),
        project_id: ProjectId::from("project-1"),
        work_item_id: Some(WorkItemId::from("task-1")),
        actor: "worker-agent-1".to_owned(),
        action: PolicyAction::ProtectedGit {
            action: ProtectedGitAction::Push,
        },
        usage: PolicyUsage::default(),
        work_item_budget: None,
        protected_git_approval: Some(approval.clone()),
        decided_at: "2026-08-08T14:02:00Z".to_owned(),
    };

    let waiting_for_approval = gate
        .authorize_and_record(request.clone(), &mut store)
        .expect("approval-required decision should persist");
    store
        .record_policy_decision(approval_decision)
        .expect("user approval decision should persist");
    store
        .record_protected_git_approval(approval)
        .expect("user approval should persist");
    let mut approved_request = request;
    approved_request.decision_id = PolicyDecisionId::from("worker-push-after-approval");
    let approved = gate
        .authorize_and_record(approved_request, &mut store)
        .expect("allowed decision should persist");

    assert_eq!(
        waiting_for_approval.decision().decision,
        PolicyDecisionKind::ApprovalRequired
    );
    assert!(waiting_for_approval.authorized_action().is_none());
    assert_eq!(approved.decision().decision, PolicyDecisionKind::Allow);
    assert!(approved.authorized_action().is_some());
}

#[test]
fn formats_event_store_errors_and_preserves_their_sources() {
    let serialization_error = serde_json::from_str::<WorkItem>("not JSON")
        .expect_err("invalid JSON should fail to deserialize");
    let database_error = EventStoreError::Database(rusqlite::Error::InvalidQuery);
    let serialization_store_error = EventStoreError::Serialization(serialization_error);
    let transition_store_error =
        EventStoreError::StateTransition(crate::domain::TransitionError::IncompleteEvidence);

    assert_eq!(
        EventStoreError::WorkItemAlreadyExists {
            work_item_id: WorkItemId::from("task-1"),
        }
        .to_string(),
        "work item task-1 already exists"
    );
    assert_eq!(
        EventStoreError::WorkItemNotFound {
            work_item_id: WorkItemId::from("task-1"),
        }
        .to_string(),
        "work item task-1 was not found"
    );
    assert_eq!(
        EventStoreError::EventIdConflict {
            event_id: WorkItemEventId::from("event-1"),
        }
        .to_string(),
        "event id event-1 conflicts with a recorded event"
    );
    assert_eq!(
        EventStoreError::PolicyDecisionIdConflict {
            decision_id: PolicyDecisionId::from("policy-1"),
        }
        .to_string(),
        "policy decision id policy-1 conflicts with a recorded decision"
    );
    assert_eq!(
        EventStoreError::ProtectedGitApprovalIdConflict {
            decision_id: PolicyDecisionId::from("policy-1"),
        }
        .to_string(),
        "protected Git approval id policy-1 conflicts with a recorded approval"
    );
    assert_eq!(
        EventStoreError::PolicyApprovalDecisionNotFound {
            decision_id: PolicyDecisionId::from("policy-1"),
        }
        .to_string(),
        "protected Git approval requires recorded policy decision policy-1"
    );
    assert_eq!(
        EventStoreError::PolicyApprovalDecisionMismatch {
            decision_id: PolicyDecisionId::from("policy-1"),
        }
        .to_string(),
        "policy decision policy-1 does not authorize the protected Git approval"
    );
    assert_eq!(
        EventStoreError::MissingTransitionReason {
            event_id: WorkItemEventId::from("event-1"),
        }
        .to_string(),
        "state-transition event event-1 requires a reason"
    );
    assert_eq!(
        EventStoreError::MissingRecoveryEventId {
            work_item_id: WorkItemId::from("task-1"),
        }
        .to_string(),
        "restart reconciliation requires an event id for uncertain work item task-1"
    );
    assert_eq!(
        EventStoreError::UnsupportedDatabaseSchemaVersion {
            current: 4,
            supported: 3,
        }
        .to_string(),
        "database schema version 4 is newer than the supported version 3"
    );
    assert_eq!(
        EventStoreError::InvalidEventSequence { value: -1 }.to_string(),
        "event sequence -1 is outside the supported range"
    );
    assert!(database_error.source().is_some());
    assert!(serialization_store_error.source().is_some());
    assert!(transition_store_error.source().is_some());
    assert!(
        EventStoreError::InvalidEventSequence { value: -1 }
            .source()
            .is_none()
    );
    assert!(
        EventStoreError::PolicyApprovalDecisionMismatch {
            decision_id: PolicyDecisionId::from("policy-1"),
        }
        .source()
        .is_none()
    );
    assert!(
        EventStoreError::UnsupportedDatabaseSchemaVersion {
            current: 4,
            supported: 3,
        }
        .source()
        .is_none()
    );
}

#[test]
fn rejects_duplicate_work_items_and_every_conflicting_transition_replay() {
    let mut store = SqliteEventStore::in_memory().expect("event store should open");
    store
        .create_work_item(create_command("task-1", WorkItemState::Inbox))
        .expect("work item creation should persist");
    let duplicate_work_item = CreateWorkItemCommand {
        event_id: WorkItemEventId::from("different-create-event"),
        work_item: work_item("task-1", WorkItemState::Inbox),
        recorded_at: "2026-08-08T00:00:00Z".to_owned(),
    };
    let first_transition = transition_command("plan-task-1", "task-1", WorkItemState::Planned);
    store
        .transition_work_item(first_transition)
        .expect("planning transition should persist");
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
    for conflicting_command in [
        transition_command("plan-task-1", "task-2", WorkItemState::Planned),
        transition_command("plan-task-1", "task-1", WorkItemState::Ready),
        mismatched_config,
        mismatched_evidence,
        mismatched_reason,
    ] {
        assert!(matches!(
            store.transition_work_item(conflicting_command),
            Err(EventStoreError::EventIdConflict { .. })
        ));
    }
}
