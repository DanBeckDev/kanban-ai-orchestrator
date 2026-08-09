use std::{error::Error, fmt};

use crate::agent::{AgentProfile, AgentProfileError};

use super::{BoardRepository, BoardService};

impl<Repository> BoardService<Repository>
where
    Repository: BoardRepository,
{
    pub fn save_agent_profile(
        &mut self,
        profile: AgentProfile,
    ) -> Result<AgentProfile, AgentProfileServiceError<Repository::Error>> {
        profile
            .validate()
            .map_err(AgentProfileServiceError::InvalidProfile)?;
        self.repository
            .save_agent_profile(profile)
            .map_err(AgentProfileServiceError::Repository)
    }

    pub fn agent_profile(
        &self,
        name: &str,
    ) -> Result<AgentProfile, AgentProfileServiceError<Repository::Error>> {
        self.repository
            .agent_profile(name)
            .map_err(AgentProfileServiceError::Repository)?
            .ok_or_else(|| AgentProfileServiceError::NotFound {
                name: name.to_owned(),
            })
    }

    pub fn agent_profiles(
        &self,
    ) -> Result<Vec<AgentProfile>, AgentProfileServiceError<Repository::Error>> {
        self.repository
            .agent_profiles()
            .map_err(AgentProfileServiceError::Repository)
    }
}

#[derive(Debug)]
pub enum AgentProfileServiceError<RepositoryError> {
    Repository(RepositoryError),
    InvalidProfile(AgentProfileError),
    NotFound { name: String },
}

impl<RepositoryError> fmt::Display for AgentProfileServiceError<RepositoryError>
where
    RepositoryError: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repository(error) => write!(formatter, "agent-profile storage error: {error}"),
            Self::InvalidProfile(error) => write!(formatter, "invalid agent profile: {error}"),
            Self::NotFound { name } => write!(formatter, "agent profile {name} was not found"),
        }
    }
}

impl<RepositoryError> Error for AgentProfileServiceError<RepositoryError>
where
    RepositoryError: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Repository(error) => Some(error),
            Self::InvalidProfile(error) => Some(error),
            Self::NotFound { .. } => None,
        }
    }
}
