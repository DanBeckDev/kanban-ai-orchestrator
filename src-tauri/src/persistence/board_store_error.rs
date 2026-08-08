use std::{error::Error, fmt};

use crate::domain::{BoardId, DependencyGraphError, DependencyId, ProjectId, WorkItemId};

use super::EventStoreError;

#[derive(Debug)]
pub enum BoardStoreError {
    EventStore(EventStoreError),
    DependencyGraph(DependencyGraphError),
    ProjectAlreadyExists {
        project_id: ProjectId,
    },
    ProjectNotFound {
        project_id: ProjectId,
    },
    BoardAlreadyExists {
        board_id: BoardId,
    },
    BoardNotFound {
        board_id: BoardId,
    },
    WorkItemNotFound {
        work_item_id: WorkItemId,
    },
    CrossBoardDependency {
        dependency_id: DependencyId,
        upstream_board_id: BoardId,
        downstream_board_id: BoardId,
    },
    DependencyIdConflict {
        dependency_id: DependencyId,
    },
}

impl fmt::Display for BoardStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EventStore(error) => write!(formatter, "board-store event error: {error}"),
            Self::DependencyGraph(error) => write!(formatter, "invalid board dependency: {error}"),
            Self::ProjectAlreadyExists { project_id } => {
                write!(formatter, "project {} already exists", project_id.0)
            }
            Self::ProjectNotFound { project_id } => {
                write!(formatter, "project {} was not found", project_id.0)
            }
            Self::BoardAlreadyExists { board_id } => {
                write!(formatter, "board {} already exists", board_id.0)
            }
            Self::BoardNotFound { board_id } => {
                write!(formatter, "board {} was not found", board_id.0)
            }
            Self::WorkItemNotFound { work_item_id } => {
                write!(formatter, "work item {} was not found", work_item_id.0)
            }
            Self::CrossBoardDependency {
                dependency_id,
                upstream_board_id,
                downstream_board_id,
            } => write!(
                formatter,
                "dependency {} crosses boards {} and {}",
                dependency_id.0, upstream_board_id.0, downstream_board_id.0
            ),
            Self::DependencyIdConflict { dependency_id } => write!(
                formatter,
                "dependency id {} conflicts with a recorded dependency",
                dependency_id.0
            ),
        }
    }
}

impl Error for BoardStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EventStore(error) => Some(error),
            Self::DependencyGraph(error) => Some(error),
            Self::ProjectAlreadyExists { .. }
            | Self::ProjectNotFound { .. }
            | Self::BoardAlreadyExists { .. }
            | Self::BoardNotFound { .. }
            | Self::WorkItemNotFound { .. }
            | Self::CrossBoardDependency { .. }
            | Self::DependencyIdConflict { .. } => None,
        }
    }
}

impl From<EventStoreError> for BoardStoreError {
    fn from(error: EventStoreError) -> Self {
        Self::EventStore(error)
    }
}

impl From<DependencyGraphError> for BoardStoreError {
    fn from(error: DependencyGraphError) -> Self {
        Self::DependencyGraph(error)
    }
}

impl From<rusqlite::Error> for BoardStoreError {
    fn from(error: rusqlite::Error) -> Self {
        Self::EventStore(EventStoreError::Database(error))
    }
}

impl From<serde_json::Error> for BoardStoreError {
    fn from(error: serde_json::Error) -> Self {
        Self::EventStore(EventStoreError::Serialization(error))
    }
}
