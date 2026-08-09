use serde::{Deserialize, Serialize};

use super::{
    ConnectorOutboxItemId, ConnectorReconciliationItemId, ExternalLinkId, SchemaMetadata,
    VersionedSchema, WorkItemId,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorOutboxState {
    #[default]
    Pending,
    Delivering,
    Delivered,
    DeliveryUncertain,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorOutboxOperation {
    Comment { body: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorOutboxItem {
    pub schema: SchemaMetadata,
    pub id: ConnectorOutboxItemId,
    pub work_item_id: WorkItemId,
    pub connector_id: String,
    pub external_link_id: ExternalLinkId,
    pub idempotency_key: String,
    pub operation: ConnectorOutboxOperation,
    pub state: ConnectorOutboxState,
    pub created_at: String,
    pub delivered_at: Option<String>,
}

impl VersionedSchema for ConnectorOutboxItem {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorSharedField {
    Title,
    Description,
    WorkflowState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorReconciliationState {
    Matched,
    NeedsResolution,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorReconciliationItem {
    pub schema: SchemaMetadata,
    pub id: ConnectorReconciliationItemId,
    pub work_item_id: WorkItemId,
    pub connector_id: String,
    pub external_link_id: ExternalLinkId,
    pub field: ConnectorSharedField,
    pub local_value: String,
    pub remote_value: String,
    pub remote_revision: String,
    pub state: ConnectorReconciliationState,
    pub observed_at: String,
}

impl VersionedSchema for ConnectorReconciliationItem {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}
