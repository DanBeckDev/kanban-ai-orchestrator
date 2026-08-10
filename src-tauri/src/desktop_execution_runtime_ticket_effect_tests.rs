#![cfg(unix)]

use crate::{
    application::{
        RecordExecutionRequest, ResolveTicketEffectRequest, TicketEffectPromptRequest,
        TransitionWorkItemRequest, UpdateExecutionRequest,
    },
    domain::{
        ExecutionStatus, ExecutionUsage, TicketEffectAction, TicketEffectOutcome,
        TicketEffectResolution, WorkItemId, WorkItemState,
    },
};

use super::supervision_test_fixtures::{configured_runtime_with_script, transition_to_ready};
use crate::domain::BoardSupervisionMode;

#[test]
fn manual_refinement_waits_for_approval_then_updates_task_details_without_persisting_secrets() {
    let script = ticket_script(
        "refine_specification",
        "{\"title\":\"Clarify foundation\",\"description\":\"Describe the first safe step.\",\"acceptanceCriteria\":[\"The setup is understandable.\"]}",
    );
    let (service, runtime, _repository) =
        configured_runtime_with_script(BoardSupervisionMode::Manual, &script);

    let effect = runtime
        .request_ticket_effect(request(
            "refine-1",
            TicketEffectAction::RefineSpecification,
            "Authorization: Bearer copied-secret; make this task clearer.",
        ))
        .expect("manual refinement should be recorded");

    assert_eq!(effect.outcome, TicketEffectOutcome::AwaitingApproval);
    assert!(effect.prompt_summary.contains("[redacted]"));
    assert!(!effect.prompt_summary.contains("copied-secret"));
    assert_eq!(
        service
            .lock()
            .expect("service should be available")
            .work_item(&WorkItemId::from("foundation"))
            .expect("work item should load")
            .work_item
            .title,
        "Build foundation"
    );

    runtime
        .resolve_ticket_effect(ResolveTicketEffectRequest {
            effect_id: effect.id.0,
            resolution: TicketEffectResolution::Apply,
        })
        .expect("approval should apply the refinement");
    let service = service.lock().expect("service should be available");
    let work_item = service
        .work_item(&WorkItemId::from("foundation"))
        .expect("work item should load");
    assert_eq!(work_item.work_item.title, "Clarify foundation");
    assert_eq!(
        work_item.work_item.acceptance_criteria,
        vec!["The setup is understandable."]
    );
}

#[test]
fn a_manual_proposal_can_be_rejected_cancelled_or_marked_stale_without_a_side_effect() {
    let script = ticket_script(
        "give_worker_guidance",
        "{\"workerGuidance\":\"Start with the focused test.\"}",
    );
    let (service, runtime, _repository) =
        configured_runtime_with_script(BoardSupervisionMode::Manual, &script);
    let rejected = runtime
        .request_ticket_effect(request(
            "guidance-reject",
            TicketEffectAction::GiveWorkerGuidance,
            "Give the worker one focused next step.",
        ))
        .expect("guidance should be recorded");
    runtime
        .resolve_ticket_effect(resolution(&rejected, TicketEffectResolution::Reject))
        .expect("rejection should be recorded");

    let cancelled = runtime
        .request_ticket_effect(request(
            "guidance-cancel",
            TicketEffectAction::GiveWorkerGuidance,
            "Give the worker one focused next step.",
        ))
        .expect("second guidance should be recorded");
    runtime
        .resolve_ticket_effect(resolution(&cancelled, TicketEffectResolution::Cancel))
        .expect("cancellation should be recorded");

    let stale = runtime
        .request_ticket_effect(request(
            "guidance-stale",
            TicketEffectAction::GiveWorkerGuidance,
            "Give the worker one focused next step.",
        ))
        .expect("third guidance should be recorded");
    service
        .lock()
        .expect("service should be available")
        .transition_work_item(transition("move-foundation", WorkItemState::Planned))
        .expect("user state change should make the proposal stale");
    runtime
        .resolve_ticket_effect(resolution(&stale, TicketEffectResolution::Apply))
        .expect("stale approval should remain safely recorded");

    let effects = runtime
        .ticket_effects_for_work_item("foundation")
        .expect("effects should load");
    assert_eq!(
        effect_outcome(&effects, "guidance-reject"),
        TicketEffectOutcome::Rejected
    );
    assert_eq!(
        effect_outcome(&effects, "guidance-cancel"),
        TicketEffectOutcome::Cancelled
    );
    assert_eq!(
        effect_outcome(&effects, "guidance-stale"),
        TicketEffectOutcome::Stale
    );
}

#[test]
fn evidence_explanation_is_read_only_and_applies_without_a_manual_change() {
    let script = ticket_script(
        "explain_evidence",
        "{\"evidenceExplanation\":\"The failing check identifies the first correction.\"}",
    );
    let (service, runtime, _repository) =
        configured_runtime_with_script(BoardSupervisionMode::Manual, &script);
    let effect = runtime
        .request_ticket_effect(request(
            "explain-1",
            TicketEffectAction::ExplainEvidence,
            "Explain the current evidence.",
        ))
        .expect("explanation should be recorded");

    assert_eq!(effect.outcome, TicketEffectOutcome::Applied);
    assert_eq!(
        service
            .lock()
            .expect("service should be available")
            .work_item(&WorkItemId::from("foundation"))
            .expect("work item should load")
            .work_item
            .state,
        WorkItemState::Inbox
    );
}

#[test]
fn a_retried_request_returns_the_durable_proposal_without_replaying_an_effect() {
    let script = ticket_script(
        "give_worker_guidance",
        "{\"workerGuidance\":\"Start with the focused test.\"}",
    );
    let (_service, runtime, _repository) =
        configured_runtime_with_script(BoardSupervisionMode::Manual, &script);

    let first = runtime
        .request_ticket_effect(request(
            "retry-safe",
            TicketEffectAction::GiveWorkerGuidance,
            "Give the worker one focused next step.",
        ))
        .expect("first request should be recorded");
    let second = runtime
        .request_ticket_effect(request(
            "retry-safe",
            TicketEffectAction::GiveWorkerGuidance,
            "Give the worker one focused next step.",
        ))
        .expect("retry should reuse its durable proposal");

    assert_eq!(first, second);
    assert_eq!(
        runtime
            .ticket_effects_for_work_item("foundation")
            .expect("effects should load")
            .len(),
        1
    );
}

#[test]
fn autonomous_mode_refuses_to_change_the_specification_without_saved_authority() {
    let script = ticket_script(
        "refine_specification",
        "{\"title\":\"Unsafe autonomous edit\",\"description\":\"This must wait.\",\"acceptanceCriteria\":[\"A person approved it.\"]}",
    );
    let (service, runtime, _repository) =
        configured_runtime_with_script(BoardSupervisionMode::Autonomous, &script);

    let effect = runtime
        .request_ticket_effect(request(
            "auto-refine",
            TicketEffectAction::RefineSpecification,
            "Improve the task automatically.",
        ))
        .expect("request should be audited even when authority denies it");

    assert_eq!(effect.outcome, TicketEffectOutcome::Denied);
    assert_eq!(
        effect.policy_result,
        crate::domain::SupervisionPolicyResult::Denied
    );
    assert_eq!(
        service
            .lock()
            .expect("service should be available")
            .work_item(&WorkItemId::from("foundation"))
            .expect("work item should load")
            .work_item
            .title,
        "Build foundation"
    );
}

#[test]
fn start_effect_records_a_policy_capacity_denial_before_workspace_or_worker_start() {
    let script = ticket_script("prepare_start", "{}");
    let (service, runtime, _repository) =
        configured_runtime_with_script(BoardSupervisionMode::Manual, &script);
    let mut board_service = service.lock().expect("service should be available");
    transition_to_ready(&mut board_service, "foundation");
    board_service
        .record_execution(RecordExecutionRequest {
            execution_id: "active-worker".to_owned(),
            work_item_id: "foundation".to_owned(),
            adapter_name: "worker".to_owned(),
            workspace_path: "/workspaces/active-worker".to_owned(),
            role: crate::domain::ExecutionRole::Implementation,
        })
        .expect("active worker record should persist");
    board_service
        .update_execution(UpdateExecutionRequest {
            execution_id: "active-worker".to_owned(),
            status: ExecutionStatus::Running,
            session_id: Some("active-session".to_owned()),
            usage: ExecutionUsage {
                input_tokens: 0,
                output_tokens: 0,
                cost_micros: None,
            },
            last_event_sequence: 1,
        })
        .expect("active worker should be represented for policy");
    drop(board_service);

    let effect = runtime
        .request_ticket_effect(request(
            "start-denied",
            TicketEffectAction::PrepareStart,
            "Start this task.",
        ))
        .expect("start request should record a policy result");
    assert_eq!(effect.outcome, TicketEffectOutcome::AwaitingApproval);
    runtime
        .resolve_ticket_effect(resolution(&effect, TicketEffectResolution::Apply))
        .expect("policy denial should resolve without a worker start");

    let effects = runtime
        .ticket_effects_for_work_item("foundation")
        .expect("effects should load");
    let denied = effects
        .into_iter()
        .find(|effect| effect.id.0 == "start-denied")
        .expect("start effect should be retained");
    assert_eq!(denied.outcome, TicketEffectOutcome::Denied);
    assert_eq!(
        denied.policy_result,
        crate::domain::SupervisionPolicyResult::Denied
    );
}

fn request(id: &str, action: TicketEffectAction, prompt: &str) -> TicketEffectPromptRequest {
    TicketEffectPromptRequest {
        request_id: id.to_owned(),
        work_item_id: "foundation".to_owned(),
        action,
        prompt: prompt.to_owned(),
    }
}

fn resolution(
    effect: &crate::domain::TicketEffect,
    resolution: TicketEffectResolution,
) -> ResolveTicketEffectRequest {
    ResolveTicketEffectRequest {
        effect_id: effect.id.0.clone(),
        resolution,
    }
}

fn transition(event_id: &str, next_state: WorkItemState) -> TransitionWorkItemRequest {
    TransitionWorkItemRequest {
        event_id: event_id.to_owned(),
        work_item_id: "foundation".to_owned(),
        next_state,
        evidence: None,
        reason: "A person changed the task while reviewing task AI advice.".to_owned(),
        recorded_at: "2026-08-10T12:00:00Z".to_owned(),
    }
}

fn effect_outcome(effects: &[crate::domain::TicketEffect], id: &str) -> TicketEffectOutcome {
    effects
        .iter()
        .find(|effect| effect.id.0 == id)
        .map(|effect| effect.outcome)
        .expect("effect should be retained")
}

fn ticket_script(action: &str, proposal: &str) -> String {
    format!(
        "cat >/dev/null; printf '%s' '{{\"action\":\"{action}\",\"recommendation\":\"Prepare the requested task action.\",\"rationale\":\"The task context supports it.\",\"proposal\":{proposal}}}'"
    )
}
