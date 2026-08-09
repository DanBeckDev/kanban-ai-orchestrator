use std::{error::Error, fmt};

use crate::{
    agent::AgentProfileError,
    domain::{
        ConnectorOutboxItemId, ConnectorReconciliationItemId, EvidenceId, ExecutionId,
        ExternalLinkId, PolicyDecisionId, WorkItemEventId, WorkItemId,
    },
    orchestration::PlannerProfileError,
};

#[derive(Debug)]
pub enum EventStoreError {
    Database(rusqlite::Error),
    Serialization(serde_json::Error),
    StateTransition(crate::domain::TransitionError),
    WorkItemAlreadyExists {
        work_item_id: WorkItemId,
    },
    WorkItemNotFound {
        work_item_id: WorkItemId,
    },
    ExecutionAlreadyExists {
        execution_id: ExecutionId,
    },
    ExecutionNotFound {
        execution_id: ExecutionId,
    },
    InvalidExecutionUpdate {
        execution_id: ExecutionId,
        reason: &'static str,
    },
    InvalidAgentProfile(AgentProfileError),
    InvalidPlannerProfile(PlannerProfileError),
    EvidenceAlreadyExists {
        evidence_id: EvidenceId,
    },
    EvidenceWorkItemMismatch {
        evidence_id: EvidenceId,
        work_item_id: WorkItemId,
    },
    ExternalLinkIdConflict {
        link_id: ExternalLinkId,
    },
    ExternalResourceAlreadyLinked {
        connector_id: String,
        external_id: String,
    },
    ExternalLinkNotFound {
        link_id: ExternalLinkId,
    },
    ConnectorOutboxItemConflict {
        item_id: ConnectorOutboxItemId,
    },
    ConnectorOutboxIdempotencyConflict {
        connector_id: String,
        idempotency_key: String,
    },
    ConnectorOutboxCannotTransition {
        item_id: ConnectorOutboxItemId,
    },
    ConnectorReconciliationItemConflict {
        item_id: ConnectorReconciliationItemId,
    },
    ConnectorReconciliationRevisionConflict {
        external_link_id: ExternalLinkId,
        field: String,
        remote_revision: String,
    },
    EventIdConflict {
        event_id: WorkItemEventId,
    },
    PolicyDecisionIdConflict {
        decision_id: PolicyDecisionId,
    },
    ProtectedGitApprovalIdConflict {
        decision_id: PolicyDecisionId,
    },
    PolicyApprovalDecisionNotFound {
        decision_id: PolicyDecisionId,
    },
    PolicyApprovalDecisionMismatch {
        decision_id: PolicyDecisionId,
    },
    MissingTransitionReason {
        event_id: WorkItemEventId,
    },
    MissingRecoveryEventId {
        work_item_id: WorkItemId,
    },
    InvalidEventSequence {
        value: i64,
    },
    UnsupportedDatabaseSchemaVersion {
        current: i64,
        supported: i64,
    },
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
            Self::ExecutionAlreadyExists { execution_id } => {
                write!(formatter, "execution {} already exists", execution_id.0)
            }
            Self::ExecutionNotFound { execution_id } => {
                write!(formatter, "execution {} was not found", execution_id.0)
            }
            Self::InvalidExecutionUpdate {
                execution_id,
                reason,
            } => write!(
                formatter,
                "execution {} cannot be updated: {reason}",
                execution_id.0
            ),
            Self::InvalidAgentProfile(error) => write!(formatter, "invalid agent profile: {error}"),
            Self::InvalidPlannerProfile(error) => {
                write!(formatter, "invalid planner profile: {error}")
            }
            Self::EvidenceAlreadyExists { evidence_id } => {
                write!(formatter, "evidence {} already exists", evidence_id.0)
            }
            Self::EvidenceWorkItemMismatch {
                evidence_id,
                work_item_id,
            } => write!(
                formatter,
                "evidence {} does not belong to work item {}",
                evidence_id.0, work_item_id.0
            ),
            Self::ExternalLinkIdConflict { link_id } => {
                write!(
                    formatter,
                    "external link {} conflicts with an existing link",
                    link_id.0
                )
            }
            Self::ExternalResourceAlreadyLinked {
                connector_id,
                external_id,
            } => write!(
                formatter,
                "external resource {connector_id}:{external_id} is already linked"
            ),
            Self::ExternalLinkNotFound { link_id } => {
                write!(formatter, "external link {} was not found", link_id.0)
            }
            Self::ConnectorOutboxItemConflict { item_id } => {
                write!(
                    formatter,
                    "connector outbox item {} conflicts with an existing item",
                    item_id.0
                )
            }
            Self::ConnectorOutboxIdempotencyConflict {
                connector_id,
                idempotency_key,
            } => write!(
                formatter,
                "connector outbox key {connector_id}:{idempotency_key} conflicts with an existing item"
            ),
            Self::ConnectorOutboxCannotTransition { item_id } => write!(
                formatter,
                "connector outbox item {} is not pending delivery",
                item_id.0
            ),
            Self::ConnectorReconciliationItemConflict { item_id } => write!(
                formatter,
                "connector reconciliation item {} conflicts with an existing item",
                item_id.0
            ),
            Self::ConnectorReconciliationRevisionConflict {
                external_link_id,
                field,
                remote_revision,
            } => write!(
                formatter,
                "connector reconciliation revision {}:{}:{} conflicts with an existing item",
                external_link_id.0, field, remote_revision
            ),
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
            Self::InvalidAgentProfile(error) => Some(error),
            Self::InvalidPlannerProfile(error) => Some(error),
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
