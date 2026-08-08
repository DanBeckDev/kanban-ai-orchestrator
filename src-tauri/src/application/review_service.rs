use crate::domain::{EvidenceKind, EvidenceResult, WorkItemId, WorkItemState};

use super::board_service::validate_required;
use super::{
    BoardRepository, BoardService, BoardServiceError, BoardSnapshot, RecordEvidenceRequest,
    RecordReviewCheckRequest, RecordReviewDecisionRequest,
};

impl<Repository> BoardService<Repository>
where
    Repository: BoardRepository,
{
    pub fn record_review_check(
        &mut self,
        request: RecordReviewCheckRequest,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        validate_required(&request.evidence_id, "review-check evidence id")?;
        validate_required(&request.work_item_id, "work item id")?;
        validate_required(&request.summary, "review-check summary")?;
        validate_required(&request.recorded_at, "review-check recorded-at time")?;

        self.record_review_evidence(RecordEvidenceRequest {
            evidence_id: request.evidence_id,
            work_item_id: request.work_item_id,
            kind: EvidenceKind::Check,
            result: if request.passed {
                EvidenceResult::Passed
            } else {
                EvidenceResult::Failed
            },
            summary: request.summary,
            recorded_at: request.recorded_at,
        })
    }

    pub fn record_review_decision(
        &mut self,
        request: RecordReviewDecisionRequest,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        validate_required(&request.evidence_id, "review-decision evidence id")?;
        validate_required(&request.work_item_id, "review-decision work item id")?;
        validate_required(&request.reviewer, "reviewer")?;
        validate_required(&request.summary, "review-decision summary")?;
        validate_required(&request.recorded_at, "review-decision recorded-at time")?;

        self.record_review_evidence(RecordEvidenceRequest {
            evidence_id: request.evidence_id,
            work_item_id: request.work_item_id,
            kind: EvidenceKind::ReviewDecision,
            result: if request.accepted {
                EvidenceResult::Passed
            } else {
                EvidenceResult::Failed
            },
            summary: format!("{}: {}", request.reviewer, request.summary),
            recorded_at: request.recorded_at,
        })
    }

    fn record_review_evidence(
        &mut self,
        evidence: RecordEvidenceRequest,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        let work_item_id = WorkItemId::from(evidence.work_item_id.as_str());
        let work_item = self.work_item(&work_item_id)?;
        if work_item.work_item.state != WorkItemState::Review {
            return Err(BoardServiceError::WorkItemNotInReview {
                work_item_id,
                state: work_item.work_item.state,
            });
        }
        self.record_evidence(evidence)
    }
}
