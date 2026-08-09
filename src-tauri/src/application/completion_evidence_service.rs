use crate::domain::{EvidenceKind, EvidenceResult, WorkItemId, WorkItemState};

use super::{BoardRepository, BoardService, BoardServiceError, TransitionWorkItemRequest};

impl<Repository> BoardService<Repository>
where
    Repository: BoardRepository,
{
    pub(super) fn require_recorded_completion_evidence(
        &self,
        work_item_id: &WorkItemId,
        request: &TransitionWorkItemRequest,
    ) -> Result<(), BoardServiceError<Repository::Error>> {
        if request.next_state != WorkItemState::Done {
            return Ok(());
        }
        let Some(completion) = &request.evidence else {
            return Ok(());
        };
        let evidence = self
            .repository
            .evidence_for_work_item(work_item_id)
            .map_err(BoardServiceError::Repository)?;
        let current_completion_index = current_completion_index(&evidence).ok_or_else(|| {
            BoardServiceError::MissingRecordedEvidence {
                work_item_id: work_item_id.clone(),
                kind: EvidenceKind::CompletionReport,
                result: EvidenceResult::Recorded,
            }
        })?;
        let current_evidence = &evidence[current_completion_index + 1..];
        if completion.quality_gate_passed {
            require_current_evidence(
                current_evidence,
                work_item_id,
                EvidenceKind::QualityGate,
                EvidenceResult::Passed,
            )?;
        }
        if completion.review_accepted {
            let independent_review_index = require_current_evidence(
                current_evidence,
                work_item_id,
                EvidenceKind::CleanCodeReview,
                EvidenceResult::Passed,
            )?;
            require_current_evidence(
                &current_evidence[independent_review_index + 1..],
                work_item_id,
                EvidenceKind::ReviewDecision,
                EvidenceResult::Passed,
            )?;
        }
        Ok(())
    }
}

fn current_completion_index(evidence: &[crate::domain::Evidence]) -> Option<usize> {
    evidence.iter().rposition(|record| {
        record.kind == EvidenceKind::CompletionReport && record.result == EvidenceResult::Recorded
    })
}

fn require_current_evidence<RepositoryError>(
    current_evidence: &[crate::domain::Evidence],
    work_item_id: &WorkItemId,
    kind: EvidenceKind,
    result: EvidenceResult,
) -> Result<usize, BoardServiceError<RepositoryError>> {
    current_evidence
        .iter()
        .position(|record| record.kind == kind && record.result == result)
        .ok_or_else(|| BoardServiceError::MissingRecordedEvidence {
            work_item_id: work_item_id.clone(),
            kind,
            result,
        })
}
