use std::{error::Error, fmt};

use crate::{
    domain::{BoardId, ProjectId},
    orchestration::{PlannerProfile, PlannerProfileError},
};

use super::{BoardRepository, BoardService};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannerContext {
    pub profile: PlannerProfile,
    pub repository_path: String,
}

impl<Repository> BoardService<Repository>
where
    Repository: BoardRepository,
{
    pub fn save_planner_profile(
        &mut self,
        profile: PlannerProfile,
    ) -> Result<PlannerProfile, PlannerProfileServiceError<Repository::Error>> {
        profile
            .validate()
            .map_err(PlannerProfileServiceError::InvalidProfile)?;
        self.repository
            .save_planner_profile(profile)
            .map_err(PlannerProfileServiceError::Repository)
    }

    pub fn planner_profiles(
        &self,
    ) -> Result<Vec<PlannerProfile>, PlannerProfileServiceError<Repository::Error>> {
        self.repository
            .planner_profiles()
            .map_err(PlannerProfileServiceError::Repository)
    }

    pub fn planner_context(
        &self,
        board_id: &str,
        profile_name: &str,
    ) -> Result<PlannerContext, PlannerProfileServiceError<Repository::Error>> {
        let board_id = BoardId::from(board_id);
        let board = self
            .repository
            .board(&board_id)
            .map_err(PlannerProfileServiceError::Repository)?
            .ok_or_else(|| PlannerProfileServiceError::BoardNotFound {
                board_id: board_id.clone(),
            })?;
        let project = self
            .repository
            .project(&board.project_id)
            .map_err(PlannerProfileServiceError::Repository)?
            .ok_or_else(|| PlannerProfileServiceError::ProjectNotFound {
                project_id: board.project_id,
            })?;
        let profile = self
            .repository
            .planner_profile(profile_name)
            .map_err(PlannerProfileServiceError::Repository)?
            .ok_or_else(|| PlannerProfileServiceError::NotFound {
                name: profile_name.to_owned(),
            })?;
        Ok(PlannerContext {
            profile,
            repository_path: project.repository_path,
        })
    }
}

#[derive(Debug)]
pub enum PlannerProfileServiceError<RepositoryError> {
    Repository(RepositoryError),
    InvalidProfile(PlannerProfileError),
    NotFound { name: String },
    BoardNotFound { board_id: BoardId },
    ProjectNotFound { project_id: ProjectId },
}

impl<RepositoryError> fmt::Display for PlannerProfileServiceError<RepositoryError>
where
    RepositoryError: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repository(error) => write!(formatter, "planner-profile storage error: {error}"),
            Self::InvalidProfile(error) => write!(formatter, "invalid planner profile: {error}"),
            Self::NotFound { name } => write!(formatter, "planner profile {name} was not found"),
            Self::BoardNotFound { board_id } => {
                write!(formatter, "board {} was not found", board_id.0)
            }
            Self::ProjectNotFound { project_id } => {
                write!(formatter, "project {} was not found", project_id.0)
            }
        }
    }
}

impl<RepositoryError> Error for PlannerProfileServiceError<RepositoryError>
where
    RepositoryError: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Repository(error) => Some(error),
            Self::InvalidProfile(error) => Some(error),
            Self::NotFound { .. } | Self::BoardNotFound { .. } | Self::ProjectNotFound { .. } => {
                None
            }
        }
    }
}
