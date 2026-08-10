use std::collections::BTreeSet;

use tempfile::TempDir;

use crate::{
    domain::{
        AgentEffort, AgentModelPreference, BoardId, BoardSupervision, BoardSupervisionLimits,
        BoardSupervisionMode, OrganiserDefaults, SchemaMetadata, SupervisionAction,
        SupervisionDecision, SupervisionDecisionId, SupervisionDecisionOutcome,
        SupervisionPolicyResult, TicketWorkerDefaults, WorkItemId,
    },
    persistence::{EventStoreError, SqliteEventStore},
};

#[test]
fn retains_durable_supervision_and_resolved_decisions_after_reopen() {
    let directory = TempDir::new().expect("temporary directory should exist");
    let path = directory.path().join("supervision.sqlite");
    let mut store = SqliteEventStore::open(&path).expect("store should open");
    let saved_supervision = store
        .save_board_supervision(supervision())
        .expect("supervision should save");
    let mut decision = decision("decision-1", "revision-1:task-1:1:PrepareWork");
    store
        .record_supervision_decision(decision.clone())
        .expect("decision should save");
    decision.outcome = SupervisionDecisionOutcome::Executed;
    decision.resolved_at = Some("2026-08-10T10:01:00Z".to_owned());
    store
        .resolve_supervision_decision(decision.clone())
        .expect("decision should resolve");
    drop(store);

    let reopened = SqliteEventStore::open(&path).expect("store should reopen");
    assert_eq!(
        reopened
            .board_supervision(&BoardId::from("board-1"))
            .expect("supervision should load"),
        Some(saved_supervision)
    );
    assert_eq!(
        reopened
            .supervision_decisions_for_board(&BoardId::from("board-1"))
            .expect("decisions should load"),
        vec![decision]
    );
}

#[test]
fn makes_duplicate_deliveries_idempotent_and_rejects_conflicting_ids() {
    let mut store = SqliteEventStore::in_memory().expect("store should open");
    let original = decision("decision-1", "revision-1:task-1:1:PrepareWork");
    store
        .record_supervision_decision(original.clone())
        .expect("first delivery should save");
    let duplicate = decision("different-id", "revision-1:task-1:1:PrepareWork");
    assert_eq!(
        store
            .record_supervision_decision(duplicate)
            .expect("duplicate delivery should return recorded decision"),
        original
    );

    let mut conflicting = original.clone();
    conflicting.recommendation = "Start the task instead.".to_owned();
    assert!(matches!(
        store.record_supervision_decision(conflicting),
        Err(EventStoreError::SupervisionDecisionConflict { decision_id }) if decision_id == SupervisionDecisionId::from("decision-1")
    ));
}

#[test]
fn resolves_only_the_original_pending_decision_once() {
    let mut store = SqliteEventStore::in_memory().expect("store should open");
    let mut resolved = decision("decision-1", "revision-1:task-1:1:PrepareWork");
    store
        .record_supervision_decision(resolved.clone())
        .expect("pending decision should save");
    resolved.outcome = SupervisionDecisionOutcome::Executed;
    resolved.resolved_at = Some("2026-08-10T10:01:00Z".to_owned());
    store
        .resolve_supervision_decision(resolved.clone())
        .expect("pending decision should resolve");

    assert!(matches!(
        store.resolve_supervision_decision(resolved),
        Err(EventStoreError::SupervisionDecisionConflict { decision_id }) if decision_id == SupervisionDecisionId::from("decision-1")
    ));
}

#[test]
fn refuses_to_resolve_an_unknown_or_mutated_pending_decision() {
    let mut store = SqliteEventStore::in_memory().expect("store should open");
    let unknown = decision("unknown", "unknown-key");
    assert!(matches!(
        store.resolve_supervision_decision(unknown),
        Err(EventStoreError::SupervisionDecisionNotFound { .. })
    ));

    let original = decision("decision-1", "revision-1:task-1:1:PrepareWork");
    store
        .record_supervision_decision(original.clone())
        .expect("pending decision should save");
    let mut mutated = original;
    mutated.rationale = "A later process cannot edit this audit record.".to_owned();
    mutated.outcome = SupervisionDecisionOutcome::Executed;
    mutated.resolved_at = Some("2026-08-10T10:01:00Z".to_owned());

    assert!(matches!(
        store.resolve_supervision_decision(mutated),
        Err(EventStoreError::SupervisionDecisionConflict { .. })
    ));
    assert_eq!(
        store
            .supervision_decisions_for_board(&BoardId::from("board-1"))
            .expect("decision should remain queryable")[0]
            .outcome,
        SupervisionDecisionOutcome::Pending
    );
}

fn supervision() -> BoardSupervision {
    BoardSupervision {
        schema: SchemaMetadata::current(),
        board_id: BoardId::from("board-1"),
        mode: BoardSupervisionMode::Autonomous,
        organiser: OrganiserDefaults {
            planner_profile_name: "organiser".to_owned(),
            model: AgentModelPreference::ProviderDefault,
            effort: AgentEffort::ProviderDefault,
        },
        ticket_worker: TicketWorkerDefaults {
            agent_profile_name: "worker".to_owned(),
            model: AgentModelPreference::ProviderDefault,
            effort: AgentEffort::ProviderDefault,
        },
        limits: BoardSupervisionLimits::default(),
        permitted_actions: BTreeSet::from([SupervisionAction::PrepareWork]),
        configured_by: "Alex".to_owned(),
        configured_at: "2026-08-10T10:00:00Z".to_owned(),
        paused_by: None,
        paused_at: None,
        revision: 1,
    }
}

fn decision(id: &str, key: &str) -> SupervisionDecision {
    SupervisionDecision {
        schema: SchemaMetadata::current(),
        id: SupervisionDecisionId::from(id),
        board_id: BoardId::from("board-1"),
        work_item_id: Some(WorkItemId::from("task-1")),
        organiser_profile_name: "organiser".to_owned(),
        action: SupervisionAction::PrepareWork,
        recommendation: "Prepare the task.".to_owned(),
        rationale: "It is confirmed work.".to_owned(),
        policy_result: SupervisionPolicyResult::NotRequired,
        outcome: SupervisionDecisionOutcome::Pending,
        idempotency_key: key.to_owned(),
        expected_work_item_sequence: Some(1),
        recorded_at: "2026-08-10T10:00:00Z".to_owned(),
        resolved_at: None,
    }
}
