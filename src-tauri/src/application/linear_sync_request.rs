use serde::Deserialize;

use crate::domain::ConnectorSharedField;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueLinearCommentRequest {
    pub outbox_item_id: String,
    pub work_item_id: String,
    pub idempotency_key: String,
    pub public_summary: String,
    pub recorded_at: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObserveLinearSharedFieldRequest {
    pub reconciliation_item_id: String,
    pub external_link_id: String,
    pub field: ConnectorSharedField,
    pub remote_value: String,
    pub remote_revision: String,
    pub observed_at: String,
}
