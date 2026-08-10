mod dependency_graph;
mod events;
mod model;
mod state_machine;

pub use dependency_graph::{
    DependencyBlocker, DependencyBlockerReason, DependencyContextField, DependencyEligibility,
    DependencyGraph, DependencyGraphError, WorkItemProgress,
};
pub use events::{
    CreateWorkItemCommand, EventSequence, MaterializedWorkItem, RecordedWorkItemEvent,
    RestartReconciliationCommand, TransitionWorkItemCommand, WorkItemEvent, WorkItemEventKind,
};
pub use model::{
    AgentEffort, AgentModelPreference, Board, BoardId, BoardSupervision, BoardSupervisionLimits,
    BoardSupervisionMode, CURRENT_SCHEMA_VERSION, ConnectorOutboxItem, ConnectorOutboxItemId,
    ConnectorOutboxOperation, ConnectorOutboxState, ConnectorReconciliationItem,
    ConnectorReconciliationItemId, ConnectorReconciliationState, ConnectorSharedField, Dependency,
    DependencyId, DependencyKind, DependencySource, Evidence, EvidenceId, EvidenceKind,
    EvidenceResult, Execution, ExecutionId, ExecutionRole, ExecutionStatus, ExecutionUsage,
    ExternalConnectionMode, ExternalLink, ExternalLinkId, ExternalLinkProvenance,
    OrganiserDefaults, PlanId, PolicyAction, PolicyDecision, PolicyDecisionId, PolicyDecisionKind,
    Project, ProjectAgentSettings, ProjectId, ProtectedGitAction, SchemaMetadata, SchemaVersion,
    SupervisionAction, SupervisionDecision, SupervisionDecisionId, SupervisionDecisionOutcome,
    SupervisionPolicyResult, TicketWorkerDefaults, ToolScope, VersionedSchema, WorkItem,
    WorkItemBudget, WorkItemEventId, WorkItemId, WorkItemState,
};
pub use state_machine::{
    CompletionEvidence, TransitionConfig, TransitionError, transition_work_item,
};
