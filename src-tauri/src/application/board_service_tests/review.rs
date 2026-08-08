use super::{
    BoardServiceError, CompletionEvidence, EvidenceKind, EvidenceResult, RecordEvidenceRequest,
    WorkItemId, WorkItemState, create_board, create_work_item_request, service, transition_request,
};
use crate::application::{RecordReviewCheckRequest, RecordReviewDecisionRequest};

#[test]
fn requires_a_durable_accepted_review_decision_before_human_reviewed_work_can_finish() {
    let mut service = service();
    create_board(&mut service);
    let mut request = create_work_item_request("task-1");
    request.requires_human_review = true;
    service
        .create_work_item(request)
        .expect("work item should be created");
    move_task_to_review(&mut service);
    service
        .record_review_check(RecordReviewCheckRequest {
            evidence_id: "check-task-1".to_owned(),
            work_item_id: "task-1".to_owned(),
            summary: "Required checks passed.".to_owned(),
            passed: true,
            recorded_at: "2026-08-08T00:03:00Z".to_owned(),
        })
        .expect("passing check should be recorded");
    service
        .record_evidence(RecordEvidenceRequest {
            evidence_id: "report-task-1".to_owned(),
            work_item_id: "task-1".to_owned(),
            kind: EvidenceKind::CompletionReport,
            result: EvidenceResult::Recorded,
            summary: "The agent submitted a completion report.".to_owned(),
            recorded_at: "2026-08-08T00:03:01Z".to_owned(),
        })
        .expect("completion report should be recorded");
    let completion = CompletionEvidence {
        checks_passed: true,
        completion_report_present: true,
        review_accepted: true,
    };
    let declined_snapshot = service
        .record_review_decision(RecordReviewDecisionRequest {
            evidence_id: "review-decision-task-1-rejected".to_owned(),
            work_item_id: "task-1".to_owned(),
            reviewer: "Daniel".to_owned(),
            summary: "One acceptance criterion remains open.".to_owned(),
            accepted: false,
            recorded_at: "2026-08-08T00:03:02Z".to_owned(),
        })
        .expect("rejected review decision should be recorded");
    assert!(matches!(
        declined_snapshot.evidence.last(),
        Some(record) if record.kind == EvidenceKind::ReviewDecision
            && record.result == EvidenceResult::Failed
    ));
    assert!(matches!(
        service.transition_work_item(transition_request(
            "done-task-1-without-decision",
            "task-1",
            WorkItemState::Done,
            Some(completion),
        )),
        Err(BoardServiceError::MissingRecordedEvidence {
            kind: EvidenceKind::ReviewDecision,
            result: EvidenceResult::Passed,
            ..
        })
    ));
    let snapshot = service
        .record_review_decision(RecordReviewDecisionRequest {
            evidence_id: "review-decision-task-1".to_owned(),
            work_item_id: "task-1".to_owned(),
            reviewer: "Daniel".to_owned(),
            summary: "Acceptance criteria verified.".to_owned(),
            accepted: true,
            recorded_at: "2026-08-08T00:03:03Z".to_owned(),
        })
        .expect("accepted review decision should be recorded");
    assert!(matches!(
        snapshot.evidence.last(),
        Some(record) if record.kind == EvidenceKind::ReviewDecision
            && record.result == EvidenceResult::Passed
            && record.summary == "Daniel: Acceptance criteria verified."
    ));
    assert_eq!(
        service
            .transition_work_item(transition_request(
                "done-task-1-with-decision",
                "task-1",
                WorkItemState::Done,
                Some(completion),
            ))
            .expect("accepted review should allow completion")
            .work_items[0]
            .work_item
            .state,
        WorkItemState::Done
    );
}

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
    assert_eq!(snapshot.evidence[0].kind, EvidenceKind::Check);
    assert_eq!(snapshot.evidence[0].result, EvidenceResult::Passed);
}

fn move_task_to_review(service: &mut super::BoardService<crate::persistence::SqliteEventStore>) {
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
