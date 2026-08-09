use std::{error::Error, fmt};

use crate::domain::{
    AgentModelPreference, BoardId, OrganiserDefaults, ProjectAgentSettings, TicketWorkerDefaults,
};

use super::{BoardRepository, BoardService, BoardServiceError};

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProjectAgentSettingsRequest {
    pub board_id: String,
    #[serde(default)]
    pub organiser: Option<OrganiserDefaults>,
    #[serde(default)]
    pub ticket_worker: Option<TicketWorkerDefaults>,
}

impl<Repository> BoardService<Repository>
where
    Repository: BoardRepository,
{
    pub fn save_project_agent_settings(
        &mut self,
        request: SaveProjectAgentSettingsRequest,
    ) -> Result<ProjectAgentSettings, ProjectAgentSettingsError<Repository::Error>> {
        let board = self.board_for_agent_settings(&request.board_id)?;
        validate_organiser(&request.organiser)?;
        validate_ticket_worker(&request.ticket_worker)?;
        self.ensure_selected_profiles_exist(&request.organiser, &request.ticket_worker)?;
        self.repository
            .save_project_agent_settings(ProjectAgentSettings {
                project_id: board.project_id,
                organiser: request.organiser,
                ticket_worker: request.ticket_worker,
            })
            .map_err(ProjectAgentSettingsError::Repository)
    }

    pub fn project_agent_settings_for_board(
        &self,
        board_id: &str,
    ) -> Result<Option<ProjectAgentSettings>, ProjectAgentSettingsError<Repository::Error>> {
        let board = self.board_for_agent_settings(board_id)?;
        self.repository
            .project_agent_settings(&board.project_id)
            .map_err(ProjectAgentSettingsError::Repository)
    }

    fn board_for_agent_settings(
        &self,
        board_id: &str,
    ) -> Result<crate::domain::Board, ProjectAgentSettingsError<Repository::Error>> {
        validate_name(board_id, "board id")?;
        let board_id = BoardId::from(board_id);
        self.repository
            .board(&board_id)
            .map_err(ProjectAgentSettingsError::Repository)?
            .ok_or(ProjectAgentSettingsError::BoardNotFound { board_id })
    }

    pub(crate) fn default_ticket_worker(
        &self,
        board_id: &BoardId,
    ) -> Result<Option<TicketWorkerDefaults>, BoardServiceError<Repository::Error>> {
        let board = self
            .repository
            .board(board_id)
            .map_err(BoardServiceError::Repository)?
            .ok_or_else(|| BoardServiceError::BoardNotFound {
                board_id: board_id.clone(),
            })?;
        self.repository
            .project_agent_settings(&board.project_id)
            .map_err(BoardServiceError::Repository)
            .map(|settings| settings.and_then(|settings| settings.ticket_worker))
    }

    fn ensure_selected_profiles_exist(
        &self,
        organiser: &Option<OrganiserDefaults>,
        ticket_worker: &Option<TicketWorkerDefaults>,
    ) -> Result<(), ProjectAgentSettingsError<Repository::Error>> {
        if let Some(organiser) = organiser {
            let exists = self
                .repository
                .planner_profile(&organiser.planner_profile_name)
                .map_err(ProjectAgentSettingsError::Repository)?
                .is_some();
            if !exists {
                return Err(ProjectAgentSettingsError::OrganiserProfileNotFound {
                    profile_name: organiser.planner_profile_name.clone(),
                });
            }
        }
        if let Some(ticket_worker) = ticket_worker {
            let exists = self
                .repository
                .agent_profile(&ticket_worker.agent_profile_name)
                .map_err(ProjectAgentSettingsError::Repository)?
                .is_some();
            if !exists {
                return Err(ProjectAgentSettingsError::TicketWorkerProfileNotFound {
                    profile_name: ticket_worker.agent_profile_name.clone(),
                });
            }
        }
        Ok(())
    }
}

fn validate_organiser<RepositoryError>(
    organiser: &Option<OrganiserDefaults>,
) -> Result<(), ProjectAgentSettingsError<RepositoryError>> {
    if let Some(organiser) = organiser {
        validate_name(&organiser.planner_profile_name, "organiser profile")?;
        validate_model(&organiser.model)?;
    }
    Ok(())
}

fn validate_ticket_worker<RepositoryError>(
    ticket_worker: &Option<TicketWorkerDefaults>,
) -> Result<(), ProjectAgentSettingsError<RepositoryError>> {
    if let Some(ticket_worker) = ticket_worker {
        validate_name(&ticket_worker.agent_profile_name, "ticket worker profile")?;
        validate_model(&ticket_worker.model)?;
    }
    Ok(())
}

fn validate_model<RepositoryError>(
    model: &AgentModelPreference,
) -> Result<(), ProjectAgentSettingsError<RepositoryError>> {
    let AgentModelPreference::Named(name) = model else {
        return Ok(());
    };
    validate_name(name, "model")?;
    if name.len() > 128 {
        return Err(ProjectAgentSettingsError::ModelNameTooLong);
    }
    Ok(())
}

fn validate_name<RepositoryError>(
    value: &str,
    field: &'static str,
) -> Result<(), ProjectAgentSettingsError<RepositoryError>> {
    if value.trim().is_empty() {
        Err(ProjectAgentSettingsError::MissingRequiredField { field })
    } else if value.contains('\0') {
        Err(ProjectAgentSettingsError::FieldContainsNull { field })
    } else {
        Ok(())
    }
}

#[derive(Debug)]
pub enum ProjectAgentSettingsError<RepositoryError> {
    Repository(RepositoryError),
    BoardNotFound { board_id: BoardId },
    MissingRequiredField { field: &'static str },
    FieldContainsNull { field: &'static str },
    ModelNameTooLong,
    OrganiserProfileNotFound { profile_name: String },
    TicketWorkerProfileNotFound { profile_name: String },
}

impl<RepositoryError> fmt::Display for ProjectAgentSettingsError<RepositoryError>
where
    RepositoryError: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repository(error) => write!(formatter, "project agent settings error: {error}"),
            Self::BoardNotFound { board_id } => {
                write!(formatter, "board {} was not found", board_id.0)
            }
            Self::MissingRequiredField { field } => write!(formatter, "{field} is required"),
            Self::FieldContainsNull { field } => {
                write!(formatter, "{field} cannot contain a null character")
            }
            Self::ModelNameTooLong => {
                formatter.write_str("model name cannot exceed 128 characters")
            }
            Self::OrganiserProfileNotFound { profile_name } => {
                write!(formatter, "organiser profile {profile_name} was not found")
            }
            Self::TicketWorkerProfileNotFound { profile_name } => {
                write!(
                    formatter,
                    "ticket worker profile {profile_name} was not found"
                )
            }
        }
    }
}

impl<RepositoryError> Error for ProjectAgentSettingsError<RepositoryError>
where
    RepositoryError: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Repository(error) => Some(error),
            Self::BoardNotFound { .. }
            | Self::MissingRequiredField { .. }
            | Self::FieldContainsNull { .. }
            | Self::ModelNameTooLong
            | Self::OrganiserProfileNotFound { .. }
            | Self::TicketWorkerProfileNotFound { .. } => None,
        }
    }
}
