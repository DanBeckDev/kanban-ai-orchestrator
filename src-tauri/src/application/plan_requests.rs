use serde::Deserialize;

use crate::domain::{DependencyKind, WorkItemBudget};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmPlanRequest {
    pub board_id: String,
    pub plan_id: String,
    pub confirmed_by: String,
    pub confirmed_at: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposePlanRequest {
    pub plan_id: String,
    pub board_id: String,
    pub proposed_by: String,
    pub proposed_at: String,
    pub work_items: Vec<ProposedPlanWorkItemRequest>,
    pub dependencies: Vec<ProposedPlanDependencyRequest>,
    pub unresolved_assumptions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposedPlanWorkItemRequest {
    pub work_item_id: String,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub budget: WorkItemBudget,
    pub requires_human_review: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposedPlanDependencyRequest {
    pub dependency_id: String,
    pub upstream_work_item_id: String,
    pub downstream_work_item_id: String,
    pub kind: DependencyKind,
    pub reason: String,
    pub owner: String,
    pub next_action: String,
}
