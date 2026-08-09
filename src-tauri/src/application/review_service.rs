use crate::domain::{
    Evidence, EvidenceId, EvidenceKind, EvidenceResult, ExecutionId, ExecutionRole,
    ExecutionStatus, SchemaMetadata, TransitionConfig, TransitionWorkItemCommand, WorkItemEventId,
    WorkItemId, WorkItemState,
};

use super::board_service::validate_required;
use super::{
    BoardRepository, BoardService, BoardServiceError, BoardSnapshot, RecordCleanCodeReviewRequest,
    RecordEvidenceRequest, RecordReviewCheckRequest, RecordReviewDecisionRequest,
};

const MAX_CLEAN_CODE_REVIEW_SUMMARY_CHARS: usize = 2_000;

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
            kind: EvidenceKind::QualityGate,
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

    pub fn record_clean_code_review(
        &mut self,
        request: RecordCleanCodeReviewRequest,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        validate_required(&request.evidence_id, "Clean Code review evidence id")?;
        validate_required(&request.work_item_id, "Clean Code review work item id")?;
        validate_required(&request.review_execution_id, "review execution id")?;
        validate_required(&request.summary, "Clean Code review summary")?;
        validate_required(&request.recorded_at, "Clean Code review recorded-at time")?;
        if request.summary.chars().count() > MAX_CLEAN_CODE_REVIEW_SUMMARY_CHARS {
            return Err(BoardServiceError::CleanCodeReviewSummaryTooLong {
                maximum_characters: MAX_CLEAN_CODE_REVIEW_SUMMARY_CHARS,
            });
        }

        let work_item_id = WorkItemId::from(request.work_item_id.as_str());
        let work_item = self.work_item(&work_item_id)?;
        if work_item.work_item.state != WorkItemState::Review {
            return Err(BoardServiceError::WorkItemNotInReview {
                work_item_id,
                state: work_item.work_item.state,
            });
        }
        let review_execution =
            self.execution(&ExecutionId::from(request.review_execution_id.as_str()))?;
        validate_review_execution(&review_execution, &work_item.work_item.id)?;
        if self
            .repository
            .evidence_for_work_item(&work_item.work_item.id)
            .map_err(BoardServiceError::Repository)?
            .iter()
            .any(|record| {
                record.kind == EvidenceKind::CleanCodeReview
                    && record.execution_id.as_ref() == Some(&review_execution.id)
            })
        {
            return Err(BoardServiceError::CleanCodeReviewAlreadyRecorded {
                execution_id: review_execution.id,
            });
        }
        let finding_summary = format!(
            "{} ({} actionable finding(s)): {}",
            review_execution.adapter_name, request.actionable_finding_count, request.summary
        );
        let result = if request.actionable_finding_count == 0 {
            EvidenceResult::Passed
        } else {
            EvidenceResult::Failed
        };
        let evidence = Evidence {
            schema: SchemaMetadata::current(),
            id: EvidenceId::from(request.evidence_id.as_str()),
            work_item_id: work_item.work_item.id.clone(),
            execution_id: Some(review_execution.id.clone()),
            kind: EvidenceKind::CleanCodeReview,
            result,
            summary: finding_summary,
            recorded_at: request.recorded_at.clone(),
        };
        if request.actionable_finding_count == 0 {
            self.repository
                .record_evidence(evidence)
                .map_err(BoardServiceError::Repository)?;
            return self.snapshot(&work_item.work_item.board_id);
        }
        self.repository
            .record_evidence_and_transition(
                evidence,
                TransitionWorkItemCommand {
                    event_id: WorkItemEventId::from(
                        format!(
                            "clean-code-review-remediation-{}",
                            request.review_execution_id
                        )
                        .as_str(),
                    ),
                    work_item_id: work_item.work_item.id.clone(),
                    next_state: WorkItemState::Ready,
                    config: TransitionConfig {
                        human_review_required: work_item.work_item.requires_human_review,
                    },
                    evidence: None,
                    reason: format!(
                        "Independent Clean Code review found {} actionable finding(s).",
                        request.actionable_finding_count
                    ),
                    recorded_at: request.recorded_at,
                },
            )
            .map_err(BoardServiceError::Repository)?;
        self.snapshot(&work_item.work_item.board_id)
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

fn validate_review_execution<RepositoryError>(
    execution: &crate::domain::Execution,
    work_item_id: &WorkItemId,
) -> Result<(), BoardServiceError<RepositoryError>> {
    if execution.work_item_id != *work_item_id {
        return Err(BoardServiceError::ReviewExecutionDoesNotMatchWorkItem {
            execution_id: execution.id.clone(),
            work_item_id: work_item_id.clone(),
        });
    }
    if execution.role != ExecutionRole::IndependentReview {
        return Err(BoardServiceError::ReviewExecutionIsNotIndependent {
            execution_id: execution.id.clone(),
        });
    }
    if execution.status != ExecutionStatus::Completed {
        return Err(BoardServiceError::ReviewExecutionNotCompleted {
            execution_id: execution.id.clone(),
            status: execution.status,
        });
    }
    Ok(())
}
