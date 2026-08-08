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
    Board, BoardId, CURRENT_SCHEMA_VERSION, Dependency, DependencyId, DependencyKind,
    DependencySource, Evidence, EvidenceId, EvidenceKind, EvidenceResult, Execution, ExecutionId,
    ExecutionStatus, ExecutionUsage, ExternalLink, ExternalLinkId, ExternalLinkProvenance,
    PolicyDecision, PolicyDecisionId, PolicyDecisionKind, Project, ProjectId, SchemaMetadata,
    SchemaVersion, VersionedSchema, WorkItem, WorkItemBudget, WorkItemEventId, WorkItemId,
    WorkItemState,
};
pub use state_machine::{
    CompletionEvidence, TransitionConfig, TransitionError, transition_work_item,
};
