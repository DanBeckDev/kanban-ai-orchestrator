use std::{error::Error, fmt};

use crate::domain::{
    BoardId, EvidenceKind, EvidenceResult, ExecutionId, ExternalLinkId, ProjectId, WorkItemId,
    WorkItemState,
};

#[derive(Debug)]
pub enum BoardServiceError<RepositoryError> {
    Repository(RepositoryError),
    MissingRequiredField {
        field: &'static str,
    },
    InvalidAcceptanceCriteria,
    InvalidExternalIdentifier {
        field: &'static str,
    },
    InvalidExternalUrl,
    ProjectNotFound {
        project_id: ProjectId,
    },
    RepositoryUnavailable {
        project_id: ProjectId,
        repository_path: String,
    },
    BoardNotFound {
        board_id: BoardId,
    },
    WorkItemNotFound {
        work_item_id: WorkItemId,
    },
    ExecutionNotFound {
        execution_id: ExecutionId,
    },
    ExecutionNotPending {
        execution_id: ExecutionId,
        status: crate::domain::ExecutionStatus,
    },
    WorkItemNotReady {
        work_item_id: WorkItemId,
        state: WorkItemState,
    },
    WorkItemNotInReview {
        work_item_id: WorkItemId,
        state: WorkItemState,
    },
    IndependentReviewProfileMatchesImplementation {
        work_item_id: WorkItemId,
        adapter_name: String,
    },
    ReviewExecutionDoesNotMatchWorkItem {
        execution_id: ExecutionId,
        work_item_id: WorkItemId,
    },
    ReviewExecutionNotCompleted {
        execution_id: ExecutionId,
        status: crate::domain::ExecutionStatus,
    },
    ReviewExecutionIsNotIndependent {
        execution_id: ExecutionId,
    },
    CleanCodeReviewAlreadyRecorded {
        execution_id: ExecutionId,
    },
    CleanCodeReviewSummaryTooLong {
        maximum_characters: usize,
    },
    ExternalResourceNotLinked {
        connector_id: &'static str,
        external_id: String,
    },
    ExternalLinkNotFound {
        link_id: ExternalLinkId,
    },
    ExternalSyncRequiresLinkedExecution {
        work_item_id: WorkItemId,
    },
    InvalidPublicExternalComment {
        reason: &'static str,
    },
    ExternalSyncValueTooLong {
        field: &'static str,
        maximum_bytes: usize,
    },
    MissingRecordedEvidence {
        work_item_id: WorkItemId,
        kind: EvidenceKind,
        result: EvidenceResult,
    },
    PlanProposal(crate::orchestration::PlanProposalError),
    PlanConfirmation(crate::orchestration::PlanConfirmationError),
    PlanNotFound {
        plan_id: crate::domain::PlanId,
    },
}

impl<RepositoryError> fmt::Display for BoardServiceError<RepositoryError>
where
    RepositoryError: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repository(error) => write!(formatter, "board repository error: {error}"),
            Self::MissingRequiredField { field } => write!(formatter, "{field} is required"),
            Self::InvalidAcceptanceCriteria => {
                formatter.write_str("at least one non-empty acceptance criterion is required")
            }
            Self::InvalidExternalIdentifier { field } => {
                write!(formatter, "{field} must be a valid UUID")
            }
            Self::InvalidExternalUrl => {
                formatter.write_str("Linear issue URL must be an HTTPS linear.app URL")
            }
            Self::ProjectNotFound { project_id } => {
                write!(formatter, "project {} was not found", project_id.0)
            }
            Self::RepositoryUnavailable {
                project_id,
                repository_path,
            } => write!(
                formatter,
                "repository for project {} is unavailable at {repository_path}",
                project_id.0
            ),
            Self::BoardNotFound { board_id } => {
                write!(formatter, "board {} was not found", board_id.0)
            }
            Self::WorkItemNotFound { work_item_id } => {
                write!(formatter, "work item {} was not found", work_item_id.0)
            }
            Self::ExecutionNotFound { execution_id } => {
                write!(formatter, "execution {} was not found", execution_id.0)
            }
            Self::ExecutionNotPending {
                execution_id,
                status,
            } => write!(
                formatter,
                "execution {} cannot start because it is {status:?}",
                execution_id.0
            ),
            Self::WorkItemNotReady {
                work_item_id,
                state,
            } => write!(
                formatter,
                "work item {} cannot start because it is {state:?}",
                work_item_id.0
            ),
            Self::WorkItemNotInReview {
                work_item_id,
                state,
            } => write!(
                formatter,
                "work item {} cannot record review evidence because it is {state:?}",
                work_item_id.0
            ),
            Self::IndependentReviewProfileMatchesImplementation {
                work_item_id,
                adapter_name,
            } => write!(
                formatter,
                "independent review for work item {} cannot reuse implementation profile {adapter_name}",
                work_item_id.0
            ),
            Self::ReviewExecutionDoesNotMatchWorkItem {
                execution_id,
                work_item_id,
            } => write!(
                formatter,
                "review execution {} does not belong to work item {}",
                execution_id.0, work_item_id.0
            ),
            Self::ReviewExecutionNotCompleted {
                execution_id,
                status,
            } => write!(
                formatter,
                "review execution {} cannot record a decision because it is {status:?}",
                execution_id.0
            ),
            Self::ReviewExecutionIsNotIndependent { execution_id } => write!(
                formatter,
                "execution {} is not an independent review",
                execution_id.0
            ),
            Self::CleanCodeReviewAlreadyRecorded { execution_id } => write!(
                formatter,
                "independent review execution {} already has a recorded decision",
                execution_id.0
            ),
            Self::CleanCodeReviewSummaryTooLong { maximum_characters } => write!(
                formatter,
                "Clean Code review summary exceeds the {maximum_characters}-character limit"
            ),
            Self::ExternalResourceNotLinked {
                connector_id,
                external_id,
            } => write!(
                formatter,
                "external resource {connector_id}:{external_id} is not linked to a local task"
            ),
            Self::ExternalLinkNotFound { link_id } => {
                write!(formatter, "external link {} was not found", link_id.0)
            }
            Self::ExternalSyncRequiresLinkedExecution { work_item_id } => write!(
                formatter,
                "work item {} needs a Linear linked-execution connection before it can queue an external update",
                work_item_id.0
            ),
            Self::InvalidPublicExternalComment { reason } => {
                write!(
                    formatter,
                    "public Linear comment is not safe to queue: {reason}"
                )
            }
            Self::ExternalSyncValueTooLong {
                field,
                maximum_bytes,
            } => write!(
                formatter,
                "{field} exceeds the {maximum_bytes}-byte connector-sync limit"
            ),
            Self::MissingRecordedEvidence {
                work_item_id,
                kind,
                result,
            } => write!(
                formatter,
                "work item {} requires recorded {result:?} {kind:?} evidence before Done",
                work_item_id.0
            ),
            Self::PlanProposal(error) => write!(formatter, "invalid plan proposal: {error}"),
            Self::PlanConfirmation(error) => {
                write!(formatter, "invalid plan confirmation: {error}")
            }
            Self::PlanNotFound { plan_id } => {
                write!(formatter, "plan {} was not found for this board", plan_id.0)
            }
        }
    }
}

impl<RepositoryError> Error for BoardServiceError<RepositoryError>
where
    RepositoryError: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Repository(error) => Some(error),
            Self::PlanProposal(error) => Some(error),
            Self::PlanConfirmation(error) => Some(error),
            Self::MissingRequiredField { .. }
            | Self::InvalidAcceptanceCriteria
            | Self::InvalidExternalIdentifier { .. }
            | Self::InvalidExternalUrl
            | Self::ProjectNotFound { .. }
            | Self::RepositoryUnavailable { .. }
            | Self::BoardNotFound { .. }
            | Self::WorkItemNotFound { .. }
            | Self::ExecutionNotFound { .. }
            | Self::ExecutionNotPending { .. }
            | Self::WorkItemNotReady { .. }
            | Self::WorkItemNotInReview { .. }
            | Self::IndependentReviewProfileMatchesImplementation { .. }
            | Self::ReviewExecutionDoesNotMatchWorkItem { .. }
            | Self::ReviewExecutionNotCompleted { .. }
            | Self::ReviewExecutionIsNotIndependent { .. }
            | Self::CleanCodeReviewAlreadyRecorded { .. }
            | Self::CleanCodeReviewSummaryTooLong { .. }
            | Self::ExternalResourceNotLinked { .. }
            | Self::ExternalLinkNotFound { .. }
            | Self::ExternalSyncRequiresLinkedExecution { .. }
            | Self::InvalidPublicExternalComment { .. }
            | Self::ExternalSyncValueTooLong { .. }
            | Self::MissingRecordedEvidence { .. }
            | Self::PlanNotFound { .. } => None,
        }
    }
}
