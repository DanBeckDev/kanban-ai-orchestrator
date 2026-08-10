use super::TicketEffectServiceError;
use crate::{
    application::board_service_tests::{create_board, create_work_item_request, service},
    domain::{
        BoardId, BoardSupervisionMode, SchemaMetadata, SupervisionPolicyResult, TicketEffect,
        TicketEffectAction, TicketEffectId, TicketEffectOutcome, TicketEffectProposal, WorkItemId,
    },
    persistence::{BoardStoreError, EventStoreError},
};

#[test]
fn records_reads_and_applies_durable_worker_guidance() {
    let mut service = service();
    create_board(&mut service);
    service
        .create_work_item(create_work_item_request("task-1"))
        .expect("work item should exist");
    let mut effect = effect("guidance", TicketEffectAction::GiveWorkerGuidance, 1);
    effect.proposal.worker_guidance = Some("Run the focused test first.".to_owned());

    service
        .record_ticket_effect(effect.clone())
        .expect("effect should persist");
    effect.outcome = TicketEffectOutcome::Applied;
    effect.outcome_at = Some("2026-08-10T12:01:00Z".to_owned());
    service
        .update_ticket_effect(effect.clone())
        .expect("guidance should become applied");

    assert_eq!(
        service
            .ticket_effect(&TicketEffectId::from("guidance"))
            .expect("effect should load"),
        effect
    );
    assert_eq!(
        service
            .applied_worker_guidance(&WorkItemId::from("task-1"))
            .expect("guidance should load"),
        Some("Run the focused test first.".to_owned())
    );
}

#[test]
fn refuses_missing_or_incomplete_task_effects_without_mutating_the_task() {
    let mut service = service();
    create_board(&mut service);
    service
        .create_work_item(create_work_item_request("task-1"))
        .expect("work item should exist");

    assert!(matches!(
        service.ticket_effect(&TicketEffectId::from("missing")),
        Err(TicketEffectServiceError::NotFound { .. })
    ));
    assert_eq!(
        service
            .applied_worker_guidance(&WorkItemId::from("task-1"))
            .expect("empty guidance should load"),
        None
    );

    let work_item = service
        .ticket_effect_work_item(&WorkItemId::from("task-1"))
        .expect("work item should load");
    let incomplete = effect(
        "incomplete-refinement",
        TicketEffectAction::RefineSpecification,
        work_item.last_event_sequence,
    );
    assert!(matches!(
        service.refine_work_item_details(
            &work_item,
            &incomplete,
            "2026-08-10T12:00:00Z".to_owned()
        ),
        Err(TicketEffectServiceError::InvalidRefinement { .. })
    ));
    assert_eq!(
        service
            .work_item(&WorkItemId::from("task-1"))
            .expect("work item should reload")
            .work_item
            .title,
        "Implement task-1"
    );
}

#[test]
fn refines_once_idempotently_and_rejects_a_stale_follow_up() {
    let mut service = service();
    create_board(&mut service);
    service
        .create_work_item(create_work_item_request("task-1"))
        .expect("work item should exist");
    let original = service
        .ticket_effect_work_item(&WorkItemId::from("task-1"))
        .expect("work item should load");
    let mut refinement = effect(
        "refine-once",
        TicketEffectAction::RefineSpecification,
        original.last_event_sequence,
    );
    refinement.proposal = TicketEffectProposal {
        title: Some("Clarify setup".to_owned()),
        description: Some("Describe the first safe setup step.".to_owned()),
        acceptance_criteria: vec!["The setup path is clear.".to_owned()],
        ..TicketEffectProposal::default()
    };

    service
        .refine_work_item_details(&original, &refinement, "2026-08-10T12:00:00Z".to_owned())
        .expect("complete refinement should apply");
    service
        .refine_work_item_details(&original, &refinement, "2026-08-10T12:00:01Z".to_owned())
        .expect("matching refinement should be idempotent");

    let mut stale = refinement.clone();
    stale.id = TicketEffectId::from("stale-refinement");
    assert!(matches!(
        service.refine_work_item_details(&original, &stale, "2026-08-10T12:00:02Z".to_owned()),
        Err(TicketEffectServiceError::Repository(
            BoardStoreError::EventStore(EventStoreError::StaleWorkItem { .. })
        ))
    ));
    assert_eq!(
        service
            .work_item(&WorkItemId::from("task-1"))
            .expect("work item should reload")
            .work_item
            .title,
        "Clarify setup"
    );
}

fn effect(id: &str, action: TicketEffectAction, expected_sequence: u64) -> TicketEffect {
    TicketEffect {
        schema: SchemaMetadata::current(),
        id: TicketEffectId::from(id),
        board_id: BoardId::from("board-1"),
        work_item_id: WorkItemId::from("task-1"),
        organiser_profile_name: "organiser".to_owned(),
        action,
        prompt_summary: "Prepare a safe task action.".to_owned(),
        recommendation: "Use the current task context.".to_owned(),
        rationale: "The task has a bounded next step.".to_owned(),
        proposal: TicketEffectProposal::default(),
        authority_mode: BoardSupervisionMode::Manual,
        supervision_revision: None,
        policy_result: SupervisionPolicyResult::NotRequired,
        outcome: TicketEffectOutcome::Pending,
        idempotency_key: id.to_owned(),
        expected_work_item_sequence: expected_sequence,
        recorded_at: "2026-08-10T12:00:00Z".to_owned(),
        outcome_at: None,
    }
}
