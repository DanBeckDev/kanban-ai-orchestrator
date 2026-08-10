#![cfg(unix)]

use crate::{
    application::{
        ConfigureBoardSupervisionRequest, ResolveTicketEffectRequest, TicketEffectPromptRequest,
        TransitionWorkItemRequest,
    },
    domain::{
        BoardId, BoardSupervisionMode, SchemaMetadata, SupervisionPolicyResult, TicketEffect,
        TicketEffectAction, TicketEffectId, TicketEffectOutcome, TicketEffectProposal,
        TicketEffectResolution, WorkItemId, WorkItemState,
    },
};

use super::supervision_test_fixtures::{configured_runtime_with_script, transition_to_ready};

#[test]
fn manual_state_actions_are_denied_when_the_current_task_state_does_not_allow_them() {
    let (service, runtime, _repository) =
        configured_runtime_with_script(BoardSupervisionMode::Manual, "cat >/dev/null; exit 1");
    for (index, action) in [
        TicketEffectAction::PrepareStart,
        TicketEffectAction::PrepareRestart,
        TicketEffectAction::ReturnForCorrection,
        TicketEffectAction::RecoverInterrupted,
    ]
    .into_iter()
    .enumerate()
    {
        let effect = pending_effect(
            &format!("denied-{index}"),
            action,
            BoardSupervisionMode::Manual,
        );
        service
            .lock()
            .expect("service should be available")
            .record_ticket_effect(effect.clone())
            .expect("decision should persist");
        resolve(&runtime, &effect);
        assert_eq!(
            outcome(&runtime, &effect.id.0),
            TicketEffectOutcome::Denied,
            "{action:?} should not bypass the current state"
        );
    }
    assert_state(&service, WorkItemState::Inbox);
}

#[test]
fn autonomous_restart_applies_only_through_the_saved_retry_authority() {
    let script = ticket_script("prepare_restart");
    let (service, runtime, _repository) =
        configured_runtime_with_script(BoardSupervisionMode::Autonomous, &script);
    move_to_running(&service);
    transition(&service, "mark-failed", WorkItemState::Failed);

    let effect = runtime
        .request_ticket_effect(TicketEffectPromptRequest {
            request_id: "auto-restart".to_owned(),
            work_item_id: "foundation".to_owned(),
            action: TicketEffectAction::PrepareRestart,
            prompt: "Prepare the failed task to restart.".to_owned(),
        })
        .expect("saved retry authority should apply the action");

    assert_eq!(effect.outcome, TicketEffectOutcome::Applied);
    assert_state(&service, WorkItemState::Ready);
}

#[test]
fn autonomous_pending_effect_is_marked_recovered_without_a_second_attempt() {
    let (service, runtime, _repository) =
        configured_runtime_with_script(BoardSupervisionMode::Autonomous, "cat >/dev/null; exit 1");
    let mut effect = pending_effect(
        "auto-pending",
        TicketEffectAction::GiveWorkerGuidance,
        BoardSupervisionMode::Autonomous,
    );
    effect.outcome = TicketEffectOutcome::Pending;
    service
        .lock()
        .expect("service should be available")
        .record_ticket_effect(effect)
        .expect("uncertain action should persist");

    assert_eq!(
        outcome(&runtime, "auto-pending"),
        TicketEffectOutcome::Recovered
    );
}

#[test]
fn automatic_effects_fail_closed_after_an_authority_change_or_failed_review_gap() {
    let (service, runtime, _repository) =
        configured_runtime_with_script(BoardSupervisionMode::Autonomous, "cat >/dev/null; exit 1");
    let mut changed_authority = pending_effect(
        "changed-authority",
        TicketEffectAction::GiveWorkerGuidance,
        BoardSupervisionMode::Autonomous,
    );
    changed_authority.outcome = TicketEffectOutcome::Pending;
    service
        .lock()
        .expect("service should be available")
        .record_ticket_effect(changed_authority.clone())
        .expect("effect should persist");
    service
        .lock()
        .expect("service should be available")
        .configure_board_supervision(ConfigureBoardSupervisionRequest {
            board_id: "board-1".to_owned(),
            mode: BoardSupervisionMode::Autonomous,
            configured_by: "local-user".to_owned(),
            configured_at: "2026-08-10T12:01:00Z".to_owned(),
        })
        .expect("new authority revision should persist");
    runtime
        .apply_ticket_effect_under_gate(changed_authority, true)
        .expect("changed authority should deny safely");
    assert_eq!(
        outcome(&runtime, "changed-authority"),
        TicketEffectOutcome::Denied
    );

    let mut manual_effect = pending_effect(
        "manual-not-autonomous",
        TicketEffectAction::GiveWorkerGuidance,
        BoardSupervisionMode::Manual,
    );
    manual_effect.outcome = TicketEffectOutcome::Pending;
    service
        .lock()
        .expect("service should be available")
        .record_ticket_effect(manual_effect.clone())
        .expect("manual effect should persist");
    runtime
        .apply_ticket_effect_under_gate(manual_effect, true)
        .expect("manual effect should fail closed when called automatically");
    assert_eq!(
        outcome(&runtime, "manual-not-autonomous"),
        TicketEffectOutcome::Denied
    );

    move_to_running(&service);
    transition(&service, "mark-review", WorkItemState::Review);
    let mut correction = pending_effect(
        "no-failed-review",
        TicketEffectAction::ReturnForCorrection,
        BoardSupervisionMode::Manual,
    );
    correction.expected_work_item_sequence = service
        .lock()
        .expect("service should be available")
        .work_item(&WorkItemId::from("foundation"))
        .expect("work item should load")
        .last_event_sequence;
    service
        .lock()
        .expect("service should be available")
        .record_ticket_effect(correction.clone())
        .expect("correction proposal should persist");
    resolve(&runtime, &correction);
    assert_eq!(
        outcome(&runtime, "no-failed-review"),
        TicketEffectOutcome::Denied
    );
    assert_state(&service, WorkItemState::Review);
}

#[test]
fn blank_task_effect_identifiers_fail_before_any_daemon_action() {
    let (_service, runtime, _repository) =
        configured_runtime_with_script(BoardSupervisionMode::Manual, "cat >/dev/null; exit 1");
    assert!(
        runtime
            .request_ticket_effect(TicketEffectPromptRequest {
                request_id: " ".to_owned(),
                work_item_id: "foundation".to_owned(),
                action: TicketEffectAction::ExplainEvidence,
                prompt: "Explain the evidence.".to_owned(),
            })
            .is_err()
    );
    assert!(
        runtime
            .resolve_ticket_effect(ResolveTicketEffectRequest {
                effect_id: " ".to_owned(),
                resolution: TicketEffectResolution::Apply,
            })
            .is_err()
    );
}

fn pending_effect(
    id: &str,
    action: TicketEffectAction,
    authority_mode: BoardSupervisionMode,
) -> TicketEffect {
    TicketEffect {
        schema: SchemaMetadata::current(),
        id: TicketEffectId::from(id),
        board_id: BoardId::from("board-1"),
        work_item_id: WorkItemId::from("foundation"),
        organiser_profile_name: "organiser".to_owned(),
        action,
        prompt_summary: "Prepare a safe task action.".to_owned(),
        recommendation: "Use the current task state.".to_owned(),
        rationale: "The state machine must remain authoritative.".to_owned(),
        proposal: TicketEffectProposal::default(),
        authority_mode,
        supervision_revision: Some(1),
        policy_result: SupervisionPolicyResult::NotRequired,
        outcome: TicketEffectOutcome::AwaitingApproval,
        idempotency_key: id.to_owned(),
        expected_work_item_sequence: 1,
        recorded_at: "2026-08-10T12:00:00Z".to_owned(),
        outcome_at: None,
    }
}

fn resolve(runtime: &super::ExecutionRuntime, effect: &TicketEffect) {
    runtime
        .resolve_ticket_effect(ResolveTicketEffectRequest {
            effect_id: effect.id.0.clone(),
            resolution: TicketEffectResolution::Apply,
        })
        .expect("reviewed decision should resolve safely");
}

fn move_to_running(service: &std::sync::Arc<std::sync::Mutex<crate::desktop::LocalBoardService>>) {
    let mut service = service.lock().expect("service should be available");
    transition_to_ready(&mut service, "foundation");
    service
        .transition_work_item(transition_request("start-running", WorkItemState::Running))
        .expect("task should start for restart preparation");
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
        reason: "Exercise the durable task-AI state guard.".to_owned(),
        recorded_at: "2026-08-10T12:00:00Z".to_owned(),
    }
}

fn outcome(runtime: &super::ExecutionRuntime, effect_id: &str) -> TicketEffectOutcome {
    runtime
        .ticket_effects_for_work_item("foundation")
        .expect("effects should load")
        .into_iter()
        .find(|effect| effect.id.0 == effect_id)
        .expect("effect should remain visible")
        .outcome
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

fn ticket_script(action: &str) -> String {
    format!(
        "cat >/dev/null; printf '%s' '{{\"action\":\"{action}\",\"recommendation\":\"Prepare a safe retry.\",\"rationale\":\"The task failed.\",\"proposal\":{{}}}}'"
    )
}
