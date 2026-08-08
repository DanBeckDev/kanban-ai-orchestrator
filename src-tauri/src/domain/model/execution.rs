use serde::{Deserialize, Serialize};

use super::{EvidenceId, ExecutionId, SchemaMetadata, VersionedSchema, WorkItemId};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Pending,
    Running,
    AwaitingInput,
    AwaitingReview,
    Completed,
    Failed,
    Interrupted,
    Cancelled,
}

impl ExecutionStatus {
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Interrupted | Self::Cancelled
        )
    }

    pub const fn allows_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (
                Self::Pending,
                Self::Running | Self::Failed | Self::Interrupted | Self::Cancelled
            ) | (
                Self::Running,
                Self::AwaitingInput
                    | Self::AwaitingReview
                    | Self::Completed
                    | Self::Failed
                    | Self::Interrupted
                    | Self::Cancelled
            ) | (
                Self::AwaitingInput,
                Self::Running | Self::Failed | Self::Interrupted | Self::Cancelled
            ) | (
                Self::AwaitingReview,
                Self::Completed | Self::Failed | Self::Interrupted | Self::Cancelled
            )
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_micros: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Execution {
    pub schema: SchemaMetadata,
    pub id: ExecutionId,
    pub work_item_id: WorkItemId,
    pub adapter_name: String,
    pub status: ExecutionStatus,
    pub session_id: Option<String>,
    pub workspace_path: String,
    pub usage: ExecutionUsage,
    pub last_event_sequence: u64,
}

impl VersionedSchema for Execution {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    AgentReport,
    Check,
    Commit,
    PullRequest,
    CompletionReport,
    ReviewDecision,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceResult {
    Recorded,
    Passed,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Evidence {
    pub schema: SchemaMetadata,
    pub id: EvidenceId,
    pub work_item_id: WorkItemId,
    pub kind: EvidenceKind,
    pub result: EvidenceResult,
    pub summary: String,
    pub recorded_at: String,
}

impl VersionedSchema for Evidence {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}
