use serde::Deserialize;

use crate::domain::ExternalConnectionMode;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportLinearIssueRequest {
    pub external_link_id: String,
    pub work_item_id: String,
    pub issue_id: String,
    pub display_identifier: String,
    pub url: String,
    pub connection_mode: ExternalConnectionMode,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportLinearBlockerRequest {
    pub dependency_id: String,
    pub upstream_issue_id: String,
    pub downstream_issue_id: String,
    pub reason: String,
    pub owner: String,
    pub next_action: String,
    pub created_at: String,
}
