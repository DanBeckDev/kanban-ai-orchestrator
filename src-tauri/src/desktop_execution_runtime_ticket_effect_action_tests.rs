#![cfg(unix)]

use crate::{
    application::{
        RecordEvidenceRequest, ResolveTicketEffectRequest, TicketEffectPromptRequest,
        TransitionWorkItemRequest,
    },
    domain::{
        BoardId, BoardSupervisionMode, EvidenceKind, EvidenceResult, SchemaMetadata,
        SupervisionPolicyResult, TicketEffect, TicketEffectAction, TicketEffectId,
        TicketEffectOutcome, TicketEffectProposal, TicketEffectResolution, WorkItemId,
        WorkItemState,
    },
};

use super::supervision_test_fixtures::{configured_runtime_with_script, transition_to_ready};

#[test]
fn reviewed_restart_preparation_returns_a_failed_task_to_ready() {
    let script = ticket_script("prepare_restart");
    let (service, runtime, _repository) =
        configured_runtime_with_script(BoardSupervisionMode::Manual, &script);
    move_to_running(&service);
    transition(&service, "mark-failed", WorkItemState::Failed);

    let effect = request(&runtime, "restart", TicketEffectAction::PrepareRestart);
    resolve(&runtime, &effect);

    assert_state(&service, WorkItemState::Ready);
    assert_eq!(
        effect_outcome(&runtime, "restart"),
        TicketEffectOutcome::Applied
    );
}

#[test]
fn reviewed_correction_returns_a_failed_review_to_ready() {
    let script = ticket_script("return_for_correction");
    let (service, runtime, _repository) =
        configured_runtime_with_script(BoardSupervisionMode::Manual, &script);
    move_to_running(&service);
    transition(&service, "mark-review", WorkItemState::Review);
    service
        .lock()
        .expect("service should be available")
        .record_evidence(RecordEvidenceRequest {
            evidence_id: "failed-review".to_owned(),
            work_item_id: "foundation".to_owned(),
            kind: EvidenceKind::ReviewDecision,
            result: EvidenceResult::Failed,
            summary: "The implementation did not meet the acceptance criteria.".to_owned(),
            recorded_at: "2026-08-10T12:00:00Z".to_owned(),
        })
        .expect("failed review evidence should persist");

    let effect = request(
        &runtime,
        "correction",
        TicketEffectAction::ReturnForCorrection,
    );
    resolve(&runtime, &effect);

    assert_state(&service, WorkItemState::Ready);
    assert_eq!(
        effect_outcome(&runtime, "correction"),
        TicketEffectOutcome::Applied
    );
}

#[test]
fn reviewed_interruption_recovery_returns_an_interrupted_task_to_ready() {
    let script = ticket_script("recover_interrupted");
    let (service, runtime, _repository) =
        configured_runtime_with_script(BoardSupervisionMode::Manual, &script);
    move_to_running(&service);
    transition(&service, "mark-interrupted", WorkItemState::Interrupted);

    let effect = request(&runtime, "recover", TicketEffectAction::RecoverInterrupted);
    resolve(&runtime, &effect);

    assert_state(&service, WorkItemState::Ready);
    assert_eq!(
        effect_outcome(&runtime, "recover"),
        TicketEffectOutcome::Applied
    );
}

#[test]
fn pending_manual_effect_recovers_to_a_reviewable_decision_without_replaying_it() {
    let script = ticket_script("give_worker_guidance");
    let (service, runtime, _repository) =
        configured_runtime_with_script(BoardSupervisionMode::Manual, &script);
    service
        .lock()
        .expect("service should be available")
        .record_ticket_effect(pending_effect())
        .expect("uncertain effect should persist before recovery");

    let effects = runtime
        .ticket_effects_for_work_item("foundation")
        .expect("recovery should load effects");

    assert_eq!(
        effects
            .into_iter()
            .find(|effect| effect.id.0 == "pending-recovery")
            .expect("recovered effect should remain visible")
            .outcome,
        TicketEffectOutcome::AwaitingApproval
    );
}

#[test]
fn autonomous_guidance_applies_only_with_matching_saved_authority() {
    let script = ticket_script("give_worker_guidance");
    let (service, runtime, _repository) =
        configured_runtime_with_script(BoardSupervisionMode::Autonomous, &script);

    let effect = request(
        &runtime,
        "auto-guidance",
        TicketEffectAction::GiveWorkerGuidance,
    );

    assert_eq!(effect.outcome, TicketEffectOutcome::Applied);
    assert_eq!(effect.policy_result, SupervisionPolicyResult::NotRequired);
    assert_eq!(
        service
            .lock()
            .expect("service should be available")
            .applied_worker_guidance(&WorkItemId::from("foundation"))
            .expect("guidance should load"),
        Some("Run the focused test before changing the implementation.".to_owned())
    );
}

fn request(
    runtime: &super::ExecutionRuntime,
    id: &str,
    action: TicketEffectAction,
) -> TicketEffect {
    runtime
        .request_ticket_effect(TicketEffectPromptRequest {
            request_id: id.to_owned(),
            work_item_id: "foundation".to_owned(),
            action,
            prompt: "Prepare the next safe task action.".to_owned(),
        })
        .expect("task AI request should persist")
}

fn resolve(runtime: &super::ExecutionRuntime, effect: &TicketEffect) {
    runtime
        .resolve_ticket_effect(ResolveTicketEffectRequest {
            effect_id: effect.id.0.clone(),
            resolution: TicketEffectResolution::Apply,
        })
        .expect("reviewed action should apply");
}

fn move_to_running(service: &std::sync::Arc<std::sync::Mutex<crate::desktop::LocalBoardService>>) {
    let mut service = service.lock().expect("service should be available");
    transition_to_ready(&mut service, "foundation");
    service
        .transition_work_item(transition_request("start-running", WorkItemState::Running))
        .expect("task should start for state-specific action");
}

fn transition(
    service: &std::sync::Arc<std::sync::Mutex<crate::desktop::LocalBoardService>>,
    event_id: &str,
    next_state: WorkItemState,
) {
    service
        .lock()
        .expect("service should be available")
        .transition_work_item(transition_request(event_id, next_state))
        .expect("state transition should persist");
}

fn transition_request(event_id: &str, next_state: WorkItemState) -> TransitionWorkItemRequest {
    TransitionWorkItemRequest {
        event_id: event_id.to_owned(),
        work_item_id: "foundation".to_owned(),
        next_state,
        evidence: None,
        reason: "Exercise a reviewed task-AI lifecycle action.".to_owned(),
        recorded_at: "2026-08-10T12:00:00Z".to_owned(),
    }
}

fn assert_state(
    service: &std::sync::Arc<std::sync::Mutex<crate::desktop::LocalBoardService>>,
    state: WorkItemState,
) {
    assert_eq!(
        service
            .lock()
            .expect("service should be available")
            .work_item(&WorkItemId::from("foundation"))
            .expect("work item should load")
            .work_item
            .state,
        state
    );
}

fn effect_outcome(runtime: &super::ExecutionRuntime, id: &str) -> TicketEffectOutcome {
    runtime
        .ticket_effects_for_work_item("foundation")
        .expect("effects should load")
        .into_iter()
        .find(|effect| effect.id.0 == id)
        .expect("effect should remain visible")
        .outcome
}

fn pending_effect() -> TicketEffect {
    TicketEffect {
        schema: SchemaMetadata::current(),
        id: TicketEffectId::from("pending-recovery"),
        board_id: BoardId::from("board-1"),
        work_item_id: WorkItemId::from("foundation"),
        organiser_profile_name: "organiser".to_owned(),
        action: TicketEffectAction::GiveWorkerGuidance,
        prompt_summary: "Tell the worker where to start.".to_owned(),
        recommendation: "Start with the focused test.".to_owned(),
        rationale: "The task has a bounded first step.".to_owned(),
        proposal: TicketEffectProposal {
            worker_guidance: Some(
                "Run the focused test before changing the implementation.".to_owned(),
            ),
            ..TicketEffectProposal::default()
        },
        authority_mode: BoardSupervisionMode::Manual,
        supervision_revision: Some(1),
        policy_result: SupervisionPolicyResult::NotRequired,
        outcome: TicketEffectOutcome::Pending,
        idempotency_key: "pending-recovery".to_owned(),
        expected_work_item_sequence: 1,
        recorded_at: "2026-08-10T12:00:00Z".to_owned(),
        outcome_at: None,
    }
}

fn ticket_script(action: &str) -> String {
    let proposal = if action == "give_worker_guidance" {
        "{\"workerGuidance\":\"Run the focused test before changing the implementation.\"}"
    } else {
        "{}"
    };
    format!(
        "cat >/dev/null; printf '%s' '{{\"action\":\"{action}\",\"recommendation\":\"Prepare the requested task action.\",\"rationale\":\"The task context supports it.\",\"proposal\":{proposal}}}'"
    )
}
