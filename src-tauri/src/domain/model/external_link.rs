use serde::{Deserialize, Serialize};

use super::{ExternalLinkId, SchemaMetadata, VersionedSchema, WorkItemId};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalLinkProvenance {
    Imported,
    UserLinked,
    Synchronized,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalLink {
    pub schema: SchemaMetadata,
    pub id: ExternalLinkId,
    pub work_item_id: WorkItemId,
    pub connector_id: String,
    pub provenance: ExternalLinkProvenance,
    pub external_id: String,
    pub url: String,
}

impl VersionedSchema for ExternalLink {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}
