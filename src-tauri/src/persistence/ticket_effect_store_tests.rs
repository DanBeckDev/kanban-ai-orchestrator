use tempfile::TempDir;

use crate::{
    domain::{
        BoardId, BoardSupervisionMode, SchemaMetadata, SupervisionPolicyResult, TicketEffect,
        TicketEffectAction, TicketEffectId, TicketEffectOutcome, TicketEffectProposal, WorkItemId,
    },
    persistence::{EventStoreError, SqliteEventStore},
};

#[test]
fn retains_a_safe_ticket_effect_after_reopen() {
    let directory = TempDir::new().expect("temporary directory should exist");
    let path = directory.path().join("ticket-effects.sqlite");
    let mut store = SqliteEventStore::open(&path).expect("store should open");
    let mut effect = effect("effect-1", "request-1");
    store
        .record_ticket_effect(effect.clone())
        .expect("effect should save");
    effect.outcome = TicketEffectOutcome::AwaitingApproval;
    effect.outcome_at = Some("2026-08-10T12:01:00Z".to_owned());
    store
        .update_ticket_effect(effect.clone())
        .expect("effect should await approval");
    drop(store);

    let reopened = SqliteEventStore::open(&path).expect("store should reopen");
    assert_eq!(
        reopened
            .ticket_effect(&TicketEffectId::from("effect-1"))
            .expect("effect should load"),
        Some(effect)
    );
}

#[test]
fn reuses_idempotent_requests_and_rejects_conflicting_effects() {
    let mut store = SqliteEventStore::in_memory().expect("store should open");
    let original = effect("effect-1", "request-1");
    store
        .record_ticket_effect(original.clone())
        .expect("effect should save");

    assert_eq!(
        store
            .record_ticket_effect(effect("effect-2", "request-1"))
            .expect("duplicate request should reuse the saved effect"),
        original
    );

    let mut conflict = original.clone();
    conflict.recommendation = "A different recommendation.".to_owned();
    assert!(matches!(
        store.record_ticket_effect(conflict),
        Err(EventStoreError::TicketEffectConflict { effect_id }) if effect_id == TicketEffectId::from("effect-1")
    ));
}

#[test]
fn permits_reviewable_outcomes_but_never_reopens_a_terminal_effect() {
    let mut store = SqliteEventStore::in_memory().expect("store should open");
    let mut effect = effect("effect-1", "request-1");
    store
        .record_ticket_effect(effect.clone())
        .expect("effect should save");
    effect.outcome = TicketEffectOutcome::AwaitingApproval;
    effect.outcome_at = Some("2026-08-10T12:01:00Z".to_owned());
    store
        .update_ticket_effect(effect.clone())
        .expect("manual proposal should become reviewable");
    effect.outcome = TicketEffectOutcome::Cancelled;
    effect.outcome_at = Some("2026-08-10T12:02:00Z".to_owned());
    store
        .update_ticket_effect(effect.clone())
        .expect("manual proposal should be cancellable");

    effect.outcome = TicketEffectOutcome::Applied;
    effect.outcome_at = Some("2026-08-10T12:03:00Z".to_owned());
    assert!(matches!(
        store.update_ticket_effect(effect),
        Err(EventStoreError::TicketEffectInvalidOutcomeTransition { .. })
    ));
}

#[test]
fn permits_each_safe_pending_and_reviewable_terminal_outcome() {
    let mut store = SqliteEventStore::in_memory().expect("store should open");
    for (id, initial, terminal) in [
        (
            "pending-applied",
            TicketEffectOutcome::Pending,
            TicketEffectOutcome::Applied,
        ),
        (
            "pending-denied",
            TicketEffectOutcome::Pending,
            TicketEffectOutcome::Denied,
        ),
        (
            "pending-stale",
            TicketEffectOutcome::Pending,
            TicketEffectOutcome::Stale,
        ),
        (
            "pending-recovered",
            TicketEffectOutcome::Pending,
            TicketEffectOutcome::Recovered,
        ),
        (
            "reviewable-rejected",
            TicketEffectOutcome::AwaitingApproval,
            TicketEffectOutcome::Rejected,
        ),
        (
            "reviewable-denied",
            TicketEffectOutcome::AwaitingApproval,
            TicketEffectOutcome::Denied,
        ),
        (
            "reviewable-stale",
            TicketEffectOutcome::AwaitingApproval,
            TicketEffectOutcome::Stale,
        ),
    ] {
        let mut effect = effect(id, id);
        store
            .record_ticket_effect(effect.clone())
            .expect("effect should save");
        if initial == TicketEffectOutcome::AwaitingApproval {
            effect.outcome = TicketEffectOutcome::AwaitingApproval;
            effect.outcome_at = Some("2026-08-10T12:01:00Z".to_owned());
            store
                .update_ticket_effect(effect.clone())
                .expect("manual effect should become reviewable");
        }
        effect.outcome = terminal;
        effect.outcome_at = Some("2026-08-10T12:02:00Z".to_owned());
        store
            .update_ticket_effect(effect)
            .expect("declared terminal outcome should persist");
    }
}

#[test]
fn rejects_a_mutation_of_the_late_audit_fields_before_an_outcome_can_change() {
    let mut store = SqliteEventStore::in_memory().expect("store should open");
    for field in [
        "board",
        "work item",
        "organiser",
        "action",
        "prompt",
        "recommendation",
        "rationale",
        "proposal",
        "authority",
        "revision",
        "idempotency key",
        "sequence",
        "recorded at",
    ] {
        let original = effect(field, field);
        store
            .record_ticket_effect(original.clone())
            .expect("effect should save");
        let mut mutated = original;
        match field {
            "board" => mutated.board_id = BoardId::from("another-board"),
            "work item" => mutated.work_item_id = WorkItemId::from("another-task"),
            "organiser" => mutated.organiser_profile_name = "another-organiser".to_owned(),
            "action" => mutated.action = TicketEffectAction::ExplainEvidence,
            "prompt" => mutated.prompt_summary = "A different safe summary.".to_owned(),
            "recommendation" => mutated.recommendation = "A different recommendation.".to_owned(),
            "rationale" => mutated.rationale = "A different rationale.".to_owned(),
            "proposal" => mutated.proposal.worker_guidance = Some("A different guide.".to_owned()),
            "authority" => mutated.authority_mode = BoardSupervisionMode::Autonomous,
            "revision" => mutated.supervision_revision = Some(2),
            "idempotency key" => mutated.idempotency_key = "another-key".to_owned(),
            "sequence" => mutated.expected_work_item_sequence = 2,
            "recorded at" => mutated.recorded_at = "2026-08-10T12:30:00Z".to_owned(),
            _ => unreachable!("every declared immutable ticket-effect field is covered"),
        }

        assert!(matches!(
            store.update_ticket_effect(mutated),
            Err(EventStoreError::TicketEffectInvalidOutcomeTransition { .. })
        ));
    }
}

fn effect(id: &str, key: &str) -> TicketEffect {
    TicketEffect {
        schema: SchemaMetadata::current(),
        id: TicketEffectId::from(id),
        board_id: BoardId::from("board-1"),
        work_item_id: WorkItemId::from("task-1"),
        organiser_profile_name: "organiser".to_owned(),
        action: TicketEffectAction::GiveWorkerGuidance,
        prompt_summary: "Tell the worker where to start.".to_owned(),
        recommendation: "Guide the worker through the first check.".to_owned(),
        rationale: "The task needs a concise starting point.".to_owned(),
        proposal: TicketEffectProposal {
            worker_guidance: Some("Run the focused tests before editing.".to_owned()),
            ..TicketEffectProposal::default()
        },
        authority_mode: BoardSupervisionMode::Manual,
        supervision_revision: None,
        policy_result: SupervisionPolicyResult::NotRequired,
        outcome: TicketEffectOutcome::Pending,
        idempotency_key: key.to_owned(),
        expected_work_item_sequence: 1,
        recorded_at: "2026-08-10T12:00:00Z".to_owned(),
        outcome_at: None,
    }
}
