use std::{collections::BTreeSet, error::Error};

use tempfile::TempDir;

use crate::{
    domain::{
        PolicyAction, PolicyDecisionId, PolicyDecisionKind, ProjectId, WorkItem, WorkItemEventId,
        WorkItemId,
    },
    persistence::{EventStoreError, SqliteEventStore},
    policy::{PolicyGate, PolicyLimits, PolicyRequest, PolicySet, PolicyUsage, ProtectedGitAction},
};

use super::sqlite_event_store_tests::{policy_decision, protected_git_approval};

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
    let approval_decision =
        policy_decision("user-approved-push", PolicyDecisionKind::Allow, "Daniel");
    let approval = protected_git_approval("user-approved-push", "Daniel");

    for decision in [
        denied.clone(),
        approval_required.clone(),
        approval_decision.clone(),
    ] {
        store
            .record_policy_decision(decision)
            .expect("policy decision should persist");
    }
    assert_eq!(
        store
            .record_policy_decision(approval_decision.clone())
            .expect("matching decision should be idempotent"),
        approval_decision
    );
    let mut conflicting_decision = approval_decision.clone();
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
    assert_eq!(
        store
            .record_protected_git_approval(approval.clone())
            .expect("user approval should persist"),
        approval
    );
    let mut conflicting_approval = approval.clone();
    conflicting_approval.approved_at = "2026-08-08T14:02:00Z".to_owned();
    assert!(matches!(
        store.record_protected_git_approval(conflicting_approval),
        Err(EventStoreError::ProtectedGitApprovalIdConflict { .. })
    ));
    drop(store);

    let reopened = SqliteEventStore::open(&database_path).expect("event store should reopen");
    assert_eq!(
        reopened
            .policy_decisions_for_project(&ProjectId::from("project-1"))
            .expect("decisions should reopen"),
        vec![denied, approval_required, approval_decision]
    );
    assert!(
        reopened
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

    let waiting = gate
        .authorize_and_record(request.clone(), &mut store)
        .expect("approval requirement should persist");
    store
        .record_policy_decision(approval_decision)
        .expect("user decision should persist");
    store
        .record_protected_git_approval(approval)
        .expect("user approval should persist");
    let mut approved_request = request;
    approved_request.decision_id = PolicyDecisionId::from("worker-push-after-approval");
    let approved = gate
        .authorize_and_record(approved_request, &mut store)
        .expect("allowed decision should persist");

    assert_eq!(
        waiting.decision().decision,
        PolicyDecisionKind::ApprovalRequired
    );
    assert!(waiting.authorized_action().is_none());
    assert_eq!(approved.decision().decision, PolicyDecisionKind::Allow);
    assert!(approved.authorized_action().is_some());
}

#[test]
fn event_store_errors_are_actionable_and_preserve_wrapped_sources() {
    let serialization_error =
        serde_json::from_str::<WorkItem>("not JSON").expect_err("invalid JSON should fail");
    let database_error = EventStoreError::Database(rusqlite::Error::InvalidQuery);
    let serialization_store_error = EventStoreError::Serialization(serialization_error);
    let transition_store_error =
        EventStoreError::StateTransition(crate::domain::TransitionError::IncompleteEvidence);
    let messages = [
        EventStoreError::WorkItemAlreadyExists {
            work_item_id: WorkItemId::from("task-1"),
        }
        .to_string(),
        EventStoreError::WorkItemNotFound {
            work_item_id: WorkItemId::from("task-1"),
        }
        .to_string(),
        EventStoreError::EventIdConflict {
            event_id: WorkItemEventId::from("event-1"),
        }
        .to_string(),
        EventStoreError::PolicyDecisionIdConflict {
            decision_id: PolicyDecisionId::from("policy-1"),
        }
        .to_string(),
        EventStoreError::ProtectedGitApprovalIdConflict {
            decision_id: PolicyDecisionId::from("policy-1"),
        }
        .to_string(),
        EventStoreError::PolicyApprovalDecisionNotFound {
            decision_id: PolicyDecisionId::from("policy-1"),
        }
        .to_string(),
        EventStoreError::PolicyApprovalDecisionMismatch {
            decision_id: PolicyDecisionId::from("policy-1"),
        }
        .to_string(),
        EventStoreError::MissingTransitionReason {
            event_id: WorkItemEventId::from("event-1"),
        }
        .to_string(),
        EventStoreError::MissingRecoveryEventId {
            work_item_id: WorkItemId::from("task-1"),
        }
        .to_string(),
        EventStoreError::InvalidEventSequence { value: -1 }.to_string(),
        EventStoreError::UnsupportedDatabaseSchemaVersion {
            current: 5,
            supported: 4,
        }
        .to_string(),
    ];
    assert!(messages.iter().all(|message| !message.is_empty()));
    assert!(database_error.source().is_some());
    assert!(serialization_store_error.source().is_some());
    assert!(transition_store_error.source().is_some());
    assert!(
        EventStoreError::InvalidEventSequence { value: -1 }
            .source()
            .is_none()
    );
}
