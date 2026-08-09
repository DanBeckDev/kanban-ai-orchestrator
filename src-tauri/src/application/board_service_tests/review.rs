use super::{
    BoardServiceError, CompletionEvidence, EvidenceKind, EvidenceResult, RecordEvidenceRequest,
    RecordExecutionRequest, UpdateExecutionRequest, WorkItemId, WorkItemState, create_board,
    create_work_item_request, service, transition_request,
};
use crate::{
    application::RecordReviewCheckRequest,
    domain::{ExecutionRole, ExecutionStatus, ExecutionUsage},
};

#[test]
fn records_review_checks_only_after_a_task_reaches_review() {
    let mut service = service();
    create_board(&mut service);
    service
        .create_work_item(create_work_item_request("task-1"))
        .expect("work item should be created");

    let request = RecordReviewCheckRequest {
        evidence_id: "review-check-1".to_owned(),
        work_item_id: "task-1".to_owned(),
        summary: "Unit tests passed.".to_owned(),
        passed: true,
        recorded_at: "2026-08-08T00:03:00Z".to_owned(),
    };
    assert!(matches!(
        service.record_review_check(request.clone()),
        Err(BoardServiceError::WorkItemNotInReview {
            work_item_id,
            state: WorkItemState::Inbox,
        }) if work_item_id == WorkItemId::from("task-1")
    ));
    move_task_to_review(&mut service);

    let snapshot = service
        .record_review_check(request)
        .expect("review check should persist");
    assert_eq!(snapshot.evidence.len(), 1);
    assert_eq!(snapshot.evidence[0].kind, EvidenceKind::QualityGate);
    assert_eq!(snapshot.evidence[0].result, EvidenceResult::Passed);
}

#[test]
fn rejects_a_generic_check_or_an_earlier_cycle_quality_gate_when_completing_work() {
    let mut service = service();
    create_board(&mut service);
    service
        .create_work_item(create_work_item_request("task-1"))
        .expect("work item should be created");
    move_task_to_review(&mut service);
    record_completion_report(&mut service, "completion-1");
    service
        .record_evidence(RecordEvidenceRequest {
            evidence_id: "generic-check".to_owned(),
            work_item_id: "task-1".to_owned(),
            kind: EvidenceKind::Check,
            result: EvidenceResult::Passed,
            summary: "A non-quality check passed.".to_owned(),
            recorded_at: "2026-08-08T00:03:01Z".to_owned(),
        })
        .expect("generic check should be recorded");
    let completion = CompletionEvidence {
        quality_gate_passed: true,
        completion_report_present: true,
        review_accepted: false,
    };
    assert_missing_quality_gate(&mut service, "done-with-generic-check", completion);
    record_quality_gate(&mut service);
    service
        .transition_work_item(transition_request(
            "return-to-ready",
            "task-1",
            WorkItemState::Ready,
            None,
        ))
        .expect("a new implementation attempt should be allowed");
    service
        .transition_work_item(transition_request(
            "run-task-1-again",
            "task-1",
            WorkItemState::Running,
            None,
        ))
        .expect("task should run again");
    service
        .transition_work_item(transition_request(
            "review-task-1-again",
            "task-1",
            WorkItemState::Review,
            None,
        ))
        .expect("task should return for review");
    record_completion_report(&mut service, "completion-2");
    assert_missing_quality_gate(&mut service, "done-with-stale-quality-gate", completion);
}

#[test]
fn requires_an_independent_profile_and_keeps_reviewer_activation_in_review() {
    let mut service = service();
    create_board(&mut service);
    service
        .create_work_item(create_work_item_request("task-1"))
        .expect("work item should be created");
    for (event_id, next_state) in [
        ("plan-task-1", WorkItemState::Planned),
        ("ready-task-1", WorkItemState::Ready),
    ] {
        service
            .transition_work_item(transition_request(event_id, "task-1", next_state, None))
            .expect("task should become ready");
    }
    service
        .record_execution(RecordExecutionRequest {
            execution_id: "implementation-execution".to_owned(),
            work_item_id: "task-1".to_owned(),
            role: ExecutionRole::Implementation,
            adapter_name: "worker".to_owned(),
            workspace_path: "/workspaces/task-1".to_owned(),
        })
        .expect("implementation execution should be recorded");
    service
        .transition_work_item(transition_request(
            "run-task-1",
            "task-1",
            WorkItemState::Running,
            None,
        ))
        .expect("task should run");
    service
        .transition_work_item(transition_request(
            "review-task-1",
            "task-1",
            WorkItemState::Review,
            None,
        ))
        .expect("task should enter review");

    assert!(matches!(
        service.record_execution(RecordExecutionRequest {
            execution_id: "duplicate-profile-review".to_owned(),
            work_item_id: "task-1".to_owned(),
            role: ExecutionRole::IndependentReview,
            adapter_name: "worker".to_owned(),
            workspace_path: "/workspaces/task-1".to_owned(),
        }),
        Err(BoardServiceError::IndependentReviewProfileMatchesImplementation { .. })
    ));
    service
        .record_execution(RecordExecutionRequest {
            execution_id: "review-execution-1".to_owned(),
            work_item_id: "task-1".to_owned(),
            role: ExecutionRole::IndependentReview,
            adapter_name: "reviewer".to_owned(),
            workspace_path: "/workspaces/task-1".to_owned(),
        })
        .expect("independent reviewer should be recorded");
    let snapshot = service
        .activate_execution(
            "review-execution-1",
            "review-session",
            "2026-08-08T00:03:00Z",
        )
        .expect("independent reviewer should start");
    assert_eq!(
        snapshot.work_items[0].work_item.state,
        WorkItemState::Review
    );
}

pub(super) fn move_task_to_review(
    service: &mut super::BoardService<crate::persistence::SqliteEventStore>,
) {
    for (event_id, next_state) in [
        ("plan-task-1", WorkItemState::Planned),
        ("ready-task-1", WorkItemState::Ready),
        ("run-task-1", WorkItemState::Running),
        ("review-task-1", WorkItemState::Review),
    ] {
        service
            .transition_work_item(transition_request(event_id, "task-1", next_state, None))
            .expect("task should reach review");
    }
}

pub(super) fn record_completion_report(
    service: &mut super::BoardService<crate::persistence::SqliteEventStore>,
    evidence_id: &str,
) {
    service
        .record_evidence(RecordEvidenceRequest {
            evidence_id: evidence_id.to_owned(),
            work_item_id: "task-1".to_owned(),
            kind: EvidenceKind::CompletionReport,
            result: EvidenceResult::Recorded,
            summary: "Implementation completed and requested review.".to_owned(),
            recorded_at: "2026-08-08T00:03:00Z".to_owned(),
        })
        .expect("completion report should persist");
}

pub(super) fn record_quality_gate(
    service: &mut super::BoardService<crate::persistence::SqliteEventStore>,
) {
    service
        .record_review_check(RecordReviewCheckRequest {
            evidence_id: "quality-gate".to_owned(),
            work_item_id: "task-1".to_owned(),
            summary: "The full quality gate passed.".to_owned(),
            passed: true,
            recorded_at: "2026-08-08T00:03:01Z".to_owned(),
        })
        .expect("quality gate should persist");
}

pub(super) fn record_completed_independent_review(
    service: &mut super::BoardService<crate::persistence::SqliteEventStore>,
    execution_id: &str,
    reviewer_profile: &str,
) {
    service
        .record_execution(RecordExecutionRequest {
            execution_id: execution_id.to_owned(),
            work_item_id: "task-1".to_owned(),
            role: ExecutionRole::IndependentReview,
            adapter_name: reviewer_profile.to_owned(),
            workspace_path: "/workspaces/task-1".to_owned(),
        })
        .expect("review execution should be recorded");
    service
        .activate_execution(execution_id, "review-session", "2026-08-08T00:03:00Z")
        .expect("review execution should start");
    service
        .update_execution(UpdateExecutionRequest {
            execution_id: execution_id.to_owned(),
            status: ExecutionStatus::Completed,
            session_id: Some("review-session".to_owned()),
            usage: ExecutionUsage {
                input_tokens: 0,
                output_tokens: 0,
                cost_micros: None,
            },
            last_event_sequence: 1,
        })
        .expect("review execution should complete");
}

fn assert_missing_quality_gate(
    service: &mut super::BoardService<crate::persistence::SqliteEventStore>,
    event_id: &str,
    completion: CompletionEvidence,
) {
    assert!(matches!(
        service.transition_work_item(transition_request(
            event_id,
            "task-1",
            WorkItemState::Done,
            Some(completion),
        )),
        Err(BoardServiceError::MissingRecordedEvidence {
            kind: EvidenceKind::QualityGate,
            result: EvidenceResult::Passed,
            ..
        })
    ));
}
