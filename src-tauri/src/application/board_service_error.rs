use std::{error::Error, fmt};

use crate::domain::{
    BoardId, EvidenceKind, EvidenceResult, ExecutionId, ProjectId, WorkItemId, WorkItemState,
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
    ExternalResourceNotLinked {
        connector_id: &'static str,
        external_id: String,
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
            Self::ExternalResourceNotLinked {
                connector_id,
                external_id,
            } => write!(
                formatter,
                "external resource {connector_id}:{external_id} is not linked to a local task"
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
            | Self::BoardNotFound { .. }
            | Self::WorkItemNotFound { .. }
            | Self::ExecutionNotFound { .. }
            | Self::ExecutionNotPending { .. }
            | Self::WorkItemNotReady { .. }
            | Self::WorkItemNotInReview { .. }
            | Self::ExternalResourceNotLinked { .. }
            | Self::MissingRecordedEvidence { .. }
            | Self::PlanNotFound { .. } => None,
        }
    }
}
