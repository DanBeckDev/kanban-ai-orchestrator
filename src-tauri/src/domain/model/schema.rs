use serde::{Deserialize, Serialize};

pub type SchemaVersion = u16;

pub const CURRENT_SCHEMA_VERSION: SchemaVersion = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaMetadata {
    pub version: SchemaVersion,
}

impl SchemaMetadata {
    pub const fn current() -> Self {
        Self {
            version: CURRENT_SCHEMA_VERSION,
        }
    }

    pub const fn is_current(self) -> bool {
        self.version == CURRENT_SCHEMA_VERSION
    }
}

pub trait VersionedSchema {
    fn schema(&self) -> SchemaMetadata;

    fn uses_current_schema(&self) -> bool {
        self.schema().is_current()
    }
}

macro_rules! domain_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }
    };
}

domain_id!(ProjectId);
domain_id!(BoardId);
domain_id!(WorkItemId);
domain_id!(ExecutionId);
domain_id!(EvidenceId);
domain_id!(PolicyDecisionId);
domain_id!(PlanId);
domain_id!(ExternalLinkId);
domain_id!(ConnectorOutboxItemId);
domain_id!(ConnectorReconciliationItemId);
domain_id!(DependencyId);
domain_id!(WorkItemEventId);
