mod board_supervision;
mod connector_sync;
mod execution;
mod external_link;
mod policy;
mod project;
mod project_agent_settings;
mod schema;
mod work_item;

pub use board_supervision::{
    BoardSupervision, BoardSupervisionLimits, BoardSupervisionMode, SupervisionAction,
    SupervisionDecision, SupervisionDecisionOutcome, SupervisionPolicyResult,
};
pub use execution::{
    Evidence, EvidenceKind, EvidenceResult, Execution, ExecutionRole, ExecutionStatus,
    ExecutionUsage,
};
pub use external_link::{ExternalConnectionMode, ExternalLink, ExternalLinkProvenance};
pub use policy::{PolicyAction, PolicyDecision, PolicyDecisionKind, ProtectedGitAction, ToolScope};
pub use project::{Board, Project};
pub use project_agent_settings::{
    AgentEffort, AgentModelPreference, OrganiserDefaults, ProjectAgentSettings,
    TicketWorkerDefaults,
};
pub use schema::{
    BoardId, CURRENT_SCHEMA_VERSION, ConnectorOutboxItemId, ConnectorReconciliationItemId,
    DependencyId, EvidenceId, ExecutionId, ExternalLinkId, PlanId, PolicyDecisionId, ProjectId,
    SchemaMetadata, SchemaVersion, SupervisionDecisionId, VersionedSchema, WorkItemEventId,
    WorkItemId,
};
pub use work_item::{
    Dependency, DependencyKind, DependencySource, WorkItem, WorkItemBudget, WorkItemState,
};

#[cfg(test)]
mod tests;
pub use connector_sync::{
    ConnectorOutboxItem, ConnectorOutboxOperation, ConnectorOutboxState,
    ConnectorReconciliationItem, ConnectorReconciliationState, ConnectorSharedField,
};
