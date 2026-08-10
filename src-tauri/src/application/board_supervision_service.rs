use std::{collections::BTreeSet, error::Error, fmt};

use crate::domain::{
    BoardId, BoardSupervision, BoardSupervisionMode, SchemaMetadata, SupervisionAction,
    SupervisionDecision, TicketWorkerDefaults,
};

use super::{BoardRepository, BoardService};

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigureBoardSupervisionRequest {
    pub board_id: String,
    pub mode: BoardSupervisionMode,
    pub configured_by: String,
    pub configured_at: String,
}

impl<Repository> BoardService<Repository>
where
    Repository: BoardRepository,
{
    pub fn configure_board_supervision(
        &mut self,
        request: ConfigureBoardSupervisionRequest,
    ) -> Result<BoardSupervision, BoardSupervisionServiceError<Repository::Error>> {
        validate_required(&request.board_id, "board id")?;
        validate_required(&request.configured_by, "configured by")?;
        validate_required(&request.configured_at, "configured-at time")?;
        let board_id = BoardId::from(request.board_id.as_str());
        let board = self
            .repository
            .board(&board_id)
            .map_err(BoardSupervisionServiceError::Repository)?
            .ok_or_else(|| BoardSupervisionServiceError::BoardNotFound {
                board_id: board_id.clone(),
            })?;
        let existing = self
            .repository
            .board_supervision(&board_id)
            .map_err(BoardSupervisionServiceError::Repository)?;
        let (organiser, ticket_worker) =
            self.role_defaults(&board.project_id, existing.as_ref())?;
        let supervision = BoardSupervision {
            schema: SchemaMetadata::current(),
            board_id,
            mode: request.mode,
            organiser,
            ticket_worker,
            limits: existing
                .as_ref()
                .map(|record| record.limits.clone())
                .unwrap_or_default(),
            permitted_actions: permitted_actions(),
            configured_by: request.configured_by.clone(),
            configured_at: request.configured_at.clone(),
            paused_by: (request.mode == BoardSupervisionMode::Manual)
                .then_some(request.configured_by),
            paused_at: (request.mode == BoardSupervisionMode::Manual)
                .then_some(request.configured_at),
            revision: existing.map_or(1, |record| record.revision.saturating_add(1)),
        };
        self.repository
            .save_board_supervision(supervision)
            .map_err(BoardSupervisionServiceError::Repository)
    }

    pub fn board_supervision(
        &self,
        board_id: &str,
    ) -> Result<Option<BoardSupervision>, BoardSupervisionServiceError<Repository::Error>> {
        validate_required(board_id, "board id")?;
        let board_id = BoardId::from(board_id);
        if self
            .repository
            .board(&board_id)
            .map_err(BoardSupervisionServiceError::Repository)?
            .is_none()
        {
            return Err(BoardSupervisionServiceError::BoardNotFound { board_id });
        }
        self.repository
            .board_supervision(&board_id)
            .map_err(BoardSupervisionServiceError::Repository)
    }

    pub fn supervision_decisions(
        &self,
        board_id: &BoardId,
    ) -> Result<Vec<SupervisionDecision>, BoardSupervisionServiceError<Repository::Error>> {
        self.repository
            .supervision_decisions_for_board(board_id)
            .map_err(BoardSupervisionServiceError::Repository)
    }

    pub(crate) fn record_supervision_decision(
        &mut self,
        decision: SupervisionDecision,
    ) -> Result<SupervisionDecision, BoardSupervisionServiceError<Repository::Error>> {
        self.repository
            .record_supervision_decision(decision)
            .map_err(BoardSupervisionServiceError::Repository)
    }

    pub(crate) fn resolve_supervision_decision(
        &mut self,
        decision: SupervisionDecision,
    ) -> Result<SupervisionDecision, BoardSupervisionServiceError<Repository::Error>> {
        self.repository
            .resolve_supervision_decision(decision)
            .map_err(BoardSupervisionServiceError::Repository)
    }

    fn role_defaults(
        &self,
        project_id: &crate::domain::ProjectId,
        existing: Option<&BoardSupervision>,
    ) -> Result<
        (crate::domain::OrganiserDefaults, TicketWorkerDefaults),
        BoardSupervisionServiceError<Repository::Error>,
    > {
        let settings = self
            .repository
            .project_agent_settings(project_id)
            .map_err(BoardSupervisionServiceError::Repository)?;
        let organiser = settings
            .as_ref()
            .and_then(|settings| settings.organiser.clone())
            .or_else(|| existing.map(|record| record.organiser.clone()))
            .ok_or(BoardSupervisionServiceError::OrganiserNotConfigured)?;
        let ticket_worker = settings
            .and_then(|settings| settings.ticket_worker)
            .or_else(|| existing.map(|record| record.ticket_worker.clone()))
            .ok_or(BoardSupervisionServiceError::TicketWorkerNotConfigured)?;
        Ok((organiser, ticket_worker))
    }
}

fn permitted_actions() -> BTreeSet<SupervisionAction> {
    BTreeSet::from([
        SupervisionAction::PrepareWork,
        SupervisionAction::MakeWorkReady,
        SupervisionAction::StartWork,
        SupervisionAction::RetryWork,
        SupervisionAction::ReturnForCorrection,
    ])
}

fn validate_required<RepositoryError>(
    value: &str,
    field: &'static str,
) -> Result<(), BoardSupervisionServiceError<RepositoryError>> {
    if value.trim().is_empty() {
        Err(BoardSupervisionServiceError::MissingRequiredField { field })
    } else if value.contains('\0') {
        Err(BoardSupervisionServiceError::FieldContainsNull { field })
    } else {
        Ok(())
    }
}

#[derive(Debug)]
pub enum BoardSupervisionServiceError<RepositoryError> {
    Repository(RepositoryError),
    BoardNotFound { board_id: BoardId },
    MissingRequiredField { field: &'static str },
    FieldContainsNull { field: &'static str },
    OrganiserNotConfigured,
    TicketWorkerNotConfigured,
}

impl<RepositoryError> fmt::Display for BoardSupervisionServiceError<RepositoryError>
where
    RepositoryError: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repository(error) => {
                write!(formatter, "board supervision storage error: {error}")
            }
            Self::BoardNotFound { board_id } => {
                write!(formatter, "board {} was not found", board_id.0)
            }
            Self::MissingRequiredField { field } => write!(formatter, "{field} is required"),
            Self::FieldContainsNull { field } => {
                write!(formatter, "{field} cannot contain a null character")
            }
            Self::OrganiserNotConfigured => {
                formatter.write_str("choose an organiser in Settings before enabling automation")
            }
            Self::TicketWorkerNotConfigured => {
                formatter.write_str("choose a ticket worker in Settings before enabling automation")
            }
        }
    }
}

impl<RepositoryError> Error for BoardSupervisionServiceError<RepositoryError>
where
    RepositoryError: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Repository(error) => Some(error),
            Self::BoardNotFound { .. }
            | Self::MissingRequiredField { .. }
            | Self::FieldContainsNull { .. }
            | Self::OrganiserNotConfigured
            | Self::TicketWorkerNotConfigured => None,
        }
    }
}
