use std::fmt;

use serde::{Deserialize, Serialize};

use super::{
    AgentEffort, AgentModelPreference, BoardId, DependencyId, SchemaMetadata, VersionedSchema,
    WorkItemId,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkItemState {
    Inbox,
    Planned,
    Ready,
    Running,
    AwaitingInput,
    Review,
    Done,
    Blocked,
    Failed,
    Cancelled,
    Interrupted,
}

impl WorkItemState {
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Done | Self::Cancelled)
    }

    pub const fn is_recoverable(self) -> bool {
        matches!(self, Self::Blocked | Self::Failed | Self::Interrupted)
    }
}

impl fmt::Display for WorkItemState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Inbox => "inbox",
            Self::Planned => "planned",
            Self::Ready => "ready",
            Self::Running => "running",
            Self::AwaitingInput => "awaiting input",
            Self::Review => "review",
            Self::Done => "done",
            Self::Blocked => "blocked",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Interrupted => "interrupted",
        };
        formatter.write_str(name)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItemBudget {
    pub max_agent_turns: Option<u32>,
    pub max_duration_seconds: Option<u64>,
    pub max_cost_micros: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItem {
    pub schema: SchemaMetadata,
    pub id: WorkItemId,
    pub board_id: BoardId,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub budget: WorkItemBudget,
    pub state: WorkItemState,
    pub requires_human_review: bool,
    #[serde(default)]
    pub assigned_agent_profile_name: Option<String>,
    #[serde(default)]
    pub assigned_agent_model: AgentModelPreference,
    #[serde(default)]
    pub assigned_agent_effort: AgentEffort,
}

impl VersionedSchema for WorkItem {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyKind {
    Blocks,
    ReviewRequired,
    Contract,
    Soft,
}

impl DependencyKind {
    pub const fn is_hard(self) -> bool {
        matches!(self, Self::Blocks | Self::ReviewRequired)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DependencySource {
    User,
    Orchestrator,
    Connector { connector_id: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    pub schema: SchemaMetadata,
    pub id: DependencyId,
    pub upstream_work_item_id: WorkItemId,
    pub downstream_work_item_id: WorkItemId,
    pub kind: DependencyKind,
    pub source: DependencySource,
    pub reason: String,
    pub owner: String,
    pub next_action: String,
    pub created_by: String,
    pub created_at: String,
}

impl VersionedSchema for Dependency {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}
