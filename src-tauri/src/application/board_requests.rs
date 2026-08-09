use serde::Deserialize;

use crate::domain::{
    CompletionEvidence, DependencyKind, EvidenceKind, EvidenceResult, ExecutionRole,
    ExecutionStatus, ExecutionUsage, WorkItemBudget, WorkItemState,
};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectRequest {
    pub project_id: String,
    pub name: String,
    pub repository_path: String,
    pub base_ref: String,
    pub policy_set_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBoardRequest {
    pub board_id: String,
    pub project_id: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLocalBoardRequest {
    pub name: String,
    pub repository_path: String,
    #[serde(default)]
    pub base_ref: Option<String>,
    #[serde(default)]
    pub policy_set_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkItemRequest {
    pub event_id: String,
    pub work_item_id: String,
    pub board_id: String,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub budget: WorkItemBudget,
    pub requires_human_review: bool,
    pub recorded_at: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddDependencyRequest {
    pub dependency_id: String,
    pub upstream_work_item_id: String,
    pub downstream_work_item_id: String,
    pub kind: DependencyKind,
    pub reason: String,
    pub owner: String,
    pub next_action: String,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitionWorkItemRequest {
    pub event_id: String,
    pub work_item_id: String,
    pub next_state: WorkItemState,
    pub evidence: Option<CompletionEvidence>,
    pub reason: String,
    pub recorded_at: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordExecutionRequest {
    pub execution_id: String,
    pub work_item_id: String,
    pub adapter_name: String,
    pub workspace_path: String,
    #[serde(default)]
    pub role: ExecutionRole,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordEvidenceRequest {
    pub evidence_id: String,
    pub work_item_id: String,
    pub kind: EvidenceKind,
    pub result: EvidenceResult,
    pub summary: String,
    pub recorded_at: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateExecutionRequest {
    pub execution_id: String,
    pub status: ExecutionStatus,
    pub session_id: Option<String>,
    pub usage: ExecutionUsage,
    pub last_event_sequence: u64,
}
