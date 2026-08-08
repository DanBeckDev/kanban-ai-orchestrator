use serde::{Deserialize, Serialize};

use super::{BoardId, ProjectId, SchemaMetadata, VersionedSchema};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub schema: SchemaMetadata,
    pub id: ProjectId,
    pub name: String,
    pub repository_path: String,
    pub base_ref: String,
    pub policy_set_id: String,
}

impl VersionedSchema for Project {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub schema: SchemaMetadata,
    pub id: BoardId,
    pub project_id: ProjectId,
    pub name: String,
}

impl VersionedSchema for Board {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}
