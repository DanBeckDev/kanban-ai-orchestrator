use super::{
    BoardServiceError, CompletionEvidence, EvidenceKind, EvidenceResult, RecordEvidenceRequest,
    WorkItemState, create_board, create_work_item_request, service, transition_request,
};
use crate::application::{RecordCleanCodeReviewRequest, RecordReviewDecisionRequest};

use super::review::{
    move_task_to_review, record_completed_independent_review, record_quality_gate,
};

#[test]
fn requires_current_quality_and_independent_review_evidence_before_human_reviewed_work_can_finish()
{
    let mut service = service();
    create_board(&mut service);
    let mut request = create_work_item_request("task-1");
    request.requires_human_review = true;
    service
        .create_work_item(request)
        .expect("work item should be created");
    move_task_to_review(&mut service);
    service
        .record_evidence(RecordEvidenceRequest {
            evidence_id: "report-task-1".to_owned(),
            work_item_id: "task-1".to_owned(),
            kind: EvidenceKind::CompletionReport,
            result: EvidenceResult::Recorded,
            summary: "The agent submitted a completion report.".to_owned(),
            recorded_at: "2026-08-08T00:03:00Z".to_owned(),
        })
        .expect("completion report should be recorded");
    record_quality_gate(&mut service);
    let completion = CompletionEvidence {
        quality_gate_passed: true,
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
    assert_missing_evidence(
        &mut service,
        "done-task-1-without-independent-review",
        completion,
        EvidenceKind::CleanCodeReview,
    );
    let accepted_snapshot = service
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
        accepted_snapshot.evidence.last(),
        Some(record) if record.kind == EvidenceKind::ReviewDecision
            && record.result == EvidenceResult::Passed
            && record.summary == "Daniel: Acceptance criteria verified."
    ));
    record_completed_independent_review(&mut service, "review-execution-1", "reviewer");
    let independent_review_snapshot = service
        .record_clean_code_review(review_request("clean-code-review-task-1", 0))
        .expect("independent review should be recorded");
    assert!(matches!(
        independent_review_snapshot.evidence.last(),
        Some(record)
            if record.execution_id.as_ref().map(|execution_id| execution_id.0.as_str())
                == Some("review-execution-1")
    ));
    assert!(matches!(
        service.record_clean_code_review(review_request("duplicate-clean-code-review-task-1", 0,)),
        Err(BoardServiceError::CleanCodeReviewAlreadyRecorded { .. })
    ));
    assert_missing_evidence(
        &mut service,
        "done-task-1-with-decision-before-clean-code-review",
        completion,
        EvidenceKind::ReviewDecision,
    );
    service
        .record_review_decision(RecordReviewDecisionRequest {
            evidence_id: "review-decision-after-clean-code-review".to_owned(),
            work_item_id: "task-1".to_owned(),
            reviewer: "Daniel".to_owned(),
            summary: "Independent review passed and acceptance criteria are verified.".to_owned(),
            accepted: true,
            recorded_at: "2026-08-08T00:03:05Z".to_owned(),
        })
        .expect("human review should follow the independent review");
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
fn actionable_independent_review_findings_return_work_to_ready() {
    let mut service = service();
    create_board(&mut service);
    service
        .create_work_item(create_work_item_request("task-1"))
        .expect("work item should be created");
    move_task_to_review(&mut service);
    record_completed_independent_review(&mut service, "review-execution-1", "reviewer");

    let snapshot = service
        .record_clean_code_review(review_request("clean-code-review-task-1", 2))
        .expect("findings should return the task to implementation");
    assert_eq!(snapshot.work_items[0].work_item.state, WorkItemState::Ready);
    assert!(matches!(
        snapshot.evidence.last(),
        Some(record) if record.kind == EvidenceKind::CleanCodeReview
            && record.result == EvidenceResult::Failed
    ));
}

#[test]
fn rejects_an_unbounded_clean_code_review_summary() {
    let mut service = service();
    create_board(&mut service);
    service
        .create_work_item(create_work_item_request("task-1"))
        .expect("work item should be created");
    move_task_to_review(&mut service);
    record_completed_independent_review(&mut service, "review-execution-1", "reviewer");

    let mut request = review_request("clean-code-review-task-1", 0);
    request.summary = "x".repeat(2_001);
    assert!(matches!(
        service.record_clean_code_review(request),
        Err(BoardServiceError::CleanCodeReviewSummaryTooLong {
            maximum_characters: 2_000,
        })
    ));
}

fn review_request(
    evidence_id: &str,
    actionable_finding_count: u32,
) -> RecordCleanCodeReviewRequest {
    RecordCleanCodeReviewRequest {
        evidence_id: evidence_id.to_owned(),
        work_item_id: "task-1".to_owned(),
        review_execution_id: "review-execution-1".to_owned(),
        actionable_finding_count,
        summary: "No actionable findings.".to_owned(),
        recorded_at: "2026-08-08T00:03:04Z".to_owned(),
    }
}

fn assert_missing_evidence(
    service: &mut super::BoardService<crate::persistence::SqliteEventStore>,
    event_id: &str,
    completion: CompletionEvidence,
    kind: EvidenceKind,
) {
    assert!(matches!(
        service.transition_work_item(transition_request(
            event_id,
            "task-1",
            WorkItemState::Done,
            Some(completion),
        )),
        Err(BoardServiceError::MissingRecordedEvidence {
            kind: missing_kind,
            result: EvidenceResult::Passed,
            ..
        }) if missing_kind == kind
    ));
}
