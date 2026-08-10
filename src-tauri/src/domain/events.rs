use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{
    CompletionEvidence, SchemaMetadata, TransitionConfig, VersionedSchema, WorkItem,
    WorkItemEventId, WorkItemId, WorkItemState,
};

pub type EventSequence = u64;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItemEvent {
    pub schema: SchemaMetadata,
    pub id: WorkItemEventId,
    pub work_item_id: WorkItemId,
    pub kind: WorkItemEventKind,
    pub recorded_at: String,
}

impl VersionedSchema for WorkItemEvent {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkItemEventKind {
    Created {
        work_item: WorkItem,
    },
    StateTransitioned {
        from: WorkItemState,
        to: WorkItemState,
        config: TransitionConfig,
        evidence: Option<CompletionEvidence>,
        reason: String,
    },
    DetailsRefined {
        title: String,
        description: String,
        acceptance_criteria: Vec<String>,
        reason: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordedWorkItemEvent {
    pub sequence: EventSequence,
    pub event: WorkItemEvent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterializedWorkItem {
    pub work_item: WorkItem,
    pub last_event_sequence: EventSequence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateWorkItemCommand {
    pub event_id: WorkItemEventId,
    pub work_item: WorkItem,
    pub recorded_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransitionWorkItemCommand {
    pub event_id: WorkItemEventId,
    pub work_item_id: WorkItemId,
    pub next_state: WorkItemState,
    pub config: TransitionConfig,
    pub evidence: Option<CompletionEvidence>,
    pub reason: String,
    pub recorded_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefineWorkItemDetailsCommand {
    pub event_id: WorkItemEventId,
    pub work_item_id: WorkItemId,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub expected_work_item_sequence: EventSequence,
    pub reason: String,
    pub recorded_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestartReconciliationCommand {
    pub confirmed_active_work_item_ids: BTreeSet<WorkItemId>,
    pub recovery_event_ids: BTreeMap<WorkItemId, WorkItemEventId>,
    pub recorded_at: String,
}
