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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionRole {
    #[default]
    Implementation,
    IndependentReview,
}

impl ExecutionRole {
    pub const fn is_independent_review(self) -> bool {
        matches!(self, Self::IndependentReview)
    }
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
    #[serde(default)]
    pub role: ExecutionRole,
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
    QualityGate,
    Diff,
    Commit,
    PullRequest,
    CompletionReport,
    CleanCodeReview,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<ExecutionId>,
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
