use std::{error::Error, fmt};

use crate::domain::{PolicyDecisionId, WorkItemEventId, WorkItemId};

#[derive(Debug)]
pub enum EventStoreError {
    Database(rusqlite::Error),
    Serialization(serde_json::Error),
    StateTransition(crate::domain::TransitionError),
    WorkItemAlreadyExists { work_item_id: WorkItemId },
    WorkItemNotFound { work_item_id: WorkItemId },
    EventIdConflict { event_id: WorkItemEventId },
    PolicyDecisionIdConflict { decision_id: PolicyDecisionId },
    ProtectedGitApprovalIdConflict { decision_id: PolicyDecisionId },
    PolicyApprovalDecisionNotFound { decision_id: PolicyDecisionId },
    PolicyApprovalDecisionMismatch { decision_id: PolicyDecisionId },
    MissingTransitionReason { event_id: WorkItemEventId },
    MissingRecoveryEventId { work_item_id: WorkItemId },
    InvalidEventSequence { value: i64 },
    UnsupportedDatabaseSchemaVersion { current: i64, supported: i64 },
}

impl fmt::Display for EventStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => write!(formatter, "SQLite event-store error: {error}"),
            Self::Serialization(error) => {
                write!(formatter, "event-store serialization error: {error}")
            }
            Self::StateTransition(error) => write!(formatter, "state transition rejected: {error}"),
            Self::WorkItemAlreadyExists { work_item_id } => {
                write!(formatter, "work item {} already exists", work_item_id.0)
            }
            Self::WorkItemNotFound { work_item_id } => {
                write!(formatter, "work item {} was not found", work_item_id.0)
            }
            Self::EventIdConflict { event_id } => write!(
                formatter,
                "event id {} conflicts with a recorded event",
                event_id.0
            ),
            Self::PolicyDecisionIdConflict { decision_id } => write!(
                formatter,
                "policy decision id {} conflicts with a recorded decision",
                decision_id.0
            ),
            Self::ProtectedGitApprovalIdConflict { decision_id } => write!(
                formatter,
                "protected Git approval id {} conflicts with a recorded approval",
                decision_id.0
            ),
            Self::PolicyApprovalDecisionNotFound { decision_id } => write!(
                formatter,
                "protected Git approval requires recorded policy decision {}",
                decision_id.0
            ),
            Self::PolicyApprovalDecisionMismatch { decision_id } => write!(
                formatter,
                "policy decision {} does not authorize the protected Git approval",
                decision_id.0
            ),
            Self::MissingTransitionReason { event_id } => write!(
                formatter,
                "state-transition event {} requires a reason",
                event_id.0
            ),
            Self::MissingRecoveryEventId { work_item_id } => write!(
                formatter,
                "restart reconciliation requires an event id for uncertain work item {}",
                work_item_id.0
            ),
            Self::InvalidEventSequence { value } => write!(
                formatter,
                "event sequence {value} is outside the supported range"
            ),
            Self::UnsupportedDatabaseSchemaVersion { current, supported } => write!(
                formatter,
                "database schema version {current} is newer than the supported version {supported}"
            ),
        }
    }
}

impl Error for EventStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::Serialization(error) => Some(error),
            Self::StateTransition(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for EventStoreError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}
impl From<serde_json::Error> for EventStoreError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}
impl From<crate::domain::TransitionError> for EventStoreError {
    fn from(error: crate::domain::TransitionError) -> Self {
        Self::StateTransition(error)
    }
}
