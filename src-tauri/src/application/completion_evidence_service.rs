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
        if completion.checks_passed {
            require_evidence(
                &evidence,
                work_item_id,
                EvidenceKind::Check,
                EvidenceResult::Passed,
            )?;
        }
        if completion.completion_report_present {
            require_evidence(
                &evidence,
                work_item_id,
                EvidenceKind::CompletionReport,
                EvidenceResult::Recorded,
            )?;
        }
        if completion.review_accepted {
            require_evidence(
                &evidence,
                work_item_id,
                EvidenceKind::ReviewDecision,
                EvidenceResult::Passed,
            )?;
        }
        Ok(())
    }
}

fn require_evidence<RepositoryError>(
    evidence: &[crate::domain::Evidence],
    work_item_id: &WorkItemId,
    kind: EvidenceKind,
    result: EvidenceResult,
) -> Result<(), BoardServiceError<RepositoryError>> {
    if evidence
        .iter()
        .any(|record| record.kind == kind && record.result == result)
    {
        Ok(())
    } else {
        Err(BoardServiceError::MissingRecordedEvidence {
            work_item_id: work_item_id.clone(),
            kind,
            result,
        })
    }
}
