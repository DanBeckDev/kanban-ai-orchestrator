use std::{thread, time::Duration};

use crate::{
    application::{ConfigureBoardSupervisionRequest, RecordExecutionRequest},
    domain::{
        BoardId, BoardSupervisionMode, Evidence, EvidenceId, EvidenceKind, EvidenceResult,
        SupervisionAction, SupervisionDecisionOutcome, WorkItemId, WorkItemState,
    },
};

use super::{
    supervision_selection::{
        candidates, dependencies_are_complete, next_candidate, organiser_input,
    },
    supervision_test_fixtures::{
        board_snapshot, configured_runtime, configured_runtime_with_script, failed_execution,
        pending_decision, supervision, transition_to_blocked, transition_to_ready,
    },
};

#[test]
fn manual_supervision_records_a_recommendation_without_transitioning_work() {
    let (service, runtime, _repository) = configured_runtime(BoardSupervisionMode::Manual);

    runtime
        .coordinate_board("board-1")
        .expect("manual assessment should be recorded");

    let service = service.lock().expect("service should be available");
    let snapshot = service
        .snapshot(&BoardId::from("board-1"))
        .expect("board should be available");
    let decisions = service
        .supervision_decisions(&BoardId::from("board-1"))
        .expect("decisions should be available");
    assert_eq!(snapshot.work_items[0].work_item.state, WorkItemState::Inbox);
    assert_eq!(decisions.len(), 1);
    assert_eq!(
        decisions[0].outcome,
        crate::domain::SupervisionDecisionOutcome::RecommendedForApproval
    );
    assert_eq!(
        decisions[0].action,
        crate::domain::SupervisionAction::PrepareWork
    );
}

#[test]
fn supervisor_prioritizes_preparation_then_dependency_safe_readiness() {
    let supervision = supervision(BoardSupervisionMode::Autonomous);
    let inbox = board_snapshot(WorkItemState::Inbox, WorkItemState::Planned);
    let prepare = next_candidate(&inbox, &supervision).expect("inbox work should be prepared");
    assert_eq!(
        prepare.action,
        crate::domain::SupervisionAction::PrepareWork
    );
    assert_eq!(prepare.work_item_id, "foundation");

    let eligible = board_snapshot(WorkItemState::Done, WorkItemState::Planned);
    let ready =
        next_candidate(&eligible, &supervision).expect("planned dependent work should be ready");
    assert_eq!(
        ready.action,
        crate::domain::SupervisionAction::MakeWorkReady
    );
    assert_eq!(ready.work_item_id, "interface");
}

#[test]
fn supervisor_starts_only_a_dependency_ready_task() {
    let supervision = supervision(BoardSupervisionMode::Autonomous);
    let blocked = board_snapshot(WorkItemState::Ready, WorkItemState::Ready);
    let start = next_candidate(&blocked, &supervision).expect("root task should start");
    assert_eq!(start.action, crate::domain::SupervisionAction::StartWork);
    assert_eq!(start.work_item_id, "foundation");
    assert!(!dependencies_are_complete(&blocked, "interface"));
}

#[test]
fn retry_candidate_respects_the_persisted_retry_limit() {
    let mut supervision = supervision(BoardSupervisionMode::Autonomous);
    supervision.limits.max_retries_per_work_item = 0;
    let mut exhausted = board_snapshot(WorkItemState::Failed, WorkItemState::Planned);
    exhausted.executions.push(failed_execution("foundation"));
    assert!(next_candidate(&exhausted, &supervision).is_none());
    assert!(
        next_candidate(
            &board_snapshot(WorkItemState::Failed, WorkItemState::Planned),
            &supervision,
        )
        .is_none()
    );
}

#[test]
fn organiser_context_contains_normalized_facts_without_task_instructions() {
    let supervision = supervision(BoardSupervisionMode::Manual);
    let snapshot = board_snapshot(WorkItemState::Inbox, WorkItemState::Planned);
    let input = organiser_input(&snapshot, &candidates(&snapshot, &supervision))
        .expect("organiser context should construct");
    let encoded = serde_json::to_string(&input).expect("organiser context should serialize");

    assert!(encoded.contains("Build foundation"));
    assert!(!encoded.contains("A bounded task."));
    assert!(!encoded.contains("Tests pass."));
}

#[test]
fn a_named_pause_prevents_an_in_flight_organiser_assessment_from_changing_work() {
    let script = "cat >/dev/null; sleep 0.2; printf '%s' '{\"action\":\"prepare_work\",\"workItemId\":\"foundation\",\"recommendation\":\"Prepare foundation.\",\"rationale\":\"It is confirmed work.\"}'";
    let (service, runtime, _repository) =
        configured_runtime_with_script(BoardSupervisionMode::Autonomous, script);
    let coordinator = runtime.clone();
    let coordination = thread::spawn(move || coordinator.coordinate_board("board-1"));

    thread::sleep(Duration::from_millis(50));
    service
        .lock()
        .expect("service should be available")
        .configure_board_supervision(ConfigureBoardSupervisionRequest {
            board_id: "board-1".to_owned(),
            mode: BoardSupervisionMode::Manual,
            configured_by: "Alex".to_owned(),
            configured_at: "2026-08-10T10:01:00Z".to_owned(),
        })
        .expect("pause should persist");

    coordination
        .join()
        .expect("coordination thread should finish")
        .expect("paused coordination should return a snapshot");
    let service = service.lock().expect("service should be available");
    assert_eq!(
        service
            .work_item(&WorkItemId::from("foundation"))
            .expect("work item should load")
            .work_item
            .state,
        WorkItemState::Inbox
    );
    assert!(
        service
            .supervision_decisions(&BoardId::from("board-1"))
            .expect("decisions should load")
            .is_empty()
    );
}

#[test]
fn autonomous_supervision_records_a_capacity_denial_without_starting_another_worker() {
    let script = "cat >/dev/null; printf '%s' '{\"action\":\"start_work\",\"workItemId\":\"foundation\",\"recommendation\":\"Start foundation.\",\"rationale\":\"It is ready.\"}'";
    let (board_service, runtime, _repository) =
        configured_runtime_with_script(BoardSupervisionMode::Autonomous, script);
    let mut service = board_service.lock().expect("service should be available");
    transition_to_ready(&mut service, "foundation");
    service
        .record_execution(RecordExecutionRequest {
            execution_id: "active-worker".to_owned(),
            work_item_id: "foundation".to_owned(),
            role: crate::domain::ExecutionRole::Implementation,
            adapter_name: "worker".to_owned(),
            workspace_path: "/workspaces/active-worker".to_owned(),
        })
        .expect("existing worker should persist");
    drop(service);

    runtime
        .coordinate_board("board-1")
        .expect("capacity assessment should return a snapshot");

    let service = board_service.lock().expect("service should be available");
    let decisions = service
        .supervision_decisions(&BoardId::from("board-1"))
        .expect("decisions should load");
    assert_eq!(decisions.len(), 1);
    assert_eq!(
        decisions[0].action,
        crate::domain::SupervisionAction::StartWork
    );
    assert_eq!(
        decisions[0].policy_result,
        crate::domain::SupervisionPolicyResult::Denied
    );
    assert_eq!(
        decisions[0].outcome,
        crate::domain::SupervisionDecisionOutcome::Denied
    );
    assert_eq!(
        service
            .work_item(&WorkItemId::from("foundation"))
            .expect("work item should load")
            .work_item
            .state,
        WorkItemState::Ready
    );
}

#[test]
fn stale_organiser_recommendations_are_audited_without_overwriting_newer_work() {
    let script = "cat >/dev/null; sleep 0.2; printf '%s' '{\"action\":\"prepare_work\",\"workItemId\":\"foundation\",\"recommendation\":\"Prepare foundation.\",\"rationale\":\"It is confirmed work.\"}'";
    let (service, runtime, _repository) =
        configured_runtime_with_script(BoardSupervisionMode::Autonomous, script);
    let coordinator = runtime.clone();
    let coordination = thread::spawn(move || coordinator.coordinate_board("board-1"));

    thread::sleep(Duration::from_millis(50));
    let mut board_service = service.lock().expect("service should be available");
    transition_to_blocked(&mut board_service, "foundation");
    drop(board_service);

    coordination
        .join()
        .expect("coordination thread should finish")
        .expect("stale coordination should return a snapshot");
    let board_service = service.lock().expect("service should be available");
    let decisions = board_service
        .supervision_decisions(&BoardId::from("board-1"))
        .expect("decisions should load");
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].outcome, SupervisionDecisionOutcome::Stale);
    assert_eq!(
        board_service
            .work_item(&WorkItemId::from("foundation"))
            .expect("work item should load")
            .work_item
            .state,
        WorkItemState::Blocked
    );
}

#[test]
fn pending_decisions_are_recovered_before_a_new_supervision_pass() {
    let (service, runtime, _repository) = configured_runtime(BoardSupervisionMode::Autonomous);
    let mut board_service = service.lock().expect("service should be available");
    transition_to_blocked(&mut board_service, "foundation");
    board_service
        .record_supervision_decision(pending_decision())
        .expect("pending decision should persist");
    drop(board_service);

    runtime
        .coordinate_board("board-1")
        .expect("recovery pass should return a snapshot");

    let decisions = service
        .lock()
        .expect("service should be available")
        .supervision_decisions(&BoardId::from("board-1"))
        .expect("decisions should load");
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].outcome, SupervisionDecisionOutcome::Recovered);
}

#[test]
fn organiser_can_offer_recorded_review_evidence_for_correction() {
    let supervision = supervision(BoardSupervisionMode::Autonomous);
    let mut snapshot = board_snapshot(WorkItemState::Review, WorkItemState::Planned);
    snapshot.evidence.push(Evidence {
        schema: crate::domain::SchemaMetadata::current(),
        id: EvidenceId::from("failed-review"),
        work_item_id: WorkItemId::from("foundation"),
        execution_id: None,
        kind: EvidenceKind::ReviewDecision,
        result: EvidenceResult::Failed,
        summary: "The reviewer found a regression.".to_owned(),
        recorded_at: "2026-08-10T10:00:00Z".to_owned(),
    });

    assert_eq!(
        next_candidate(&snapshot, &supervision)
            .expect("failed review should become a correction candidate")
            .action,
        SupervisionAction::ReturnForCorrection
    );
}
