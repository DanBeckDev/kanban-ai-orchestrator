use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{
    BoardId, OrganiserDefaults, SchemaMetadata, SupervisionDecisionId, TicketWorkerDefaults,
    VersionedSchema, WorkItemId,
};

pub const DEFAULT_MAX_AUTONOMOUS_RETRIES_PER_WORK_ITEM: u32 = 1;
pub const DEFAULT_MAX_AUTONOMOUS_WORKERS: u32 = 1;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoardSupervisionMode {
    #[default]
    Manual,
    Autonomous,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupervisionAction {
    PrepareWork,
    MakeWorkReady,
    StartWork,
    RetryWork,
    ReturnForCorrection,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardSupervisionLimits {
    pub max_parallel_work_items: u32,
    pub max_retries_per_work_item: u32,
}

impl Default for BoardSupervisionLimits {
    fn default() -> Self {
        Self {
            max_parallel_work_items: DEFAULT_MAX_AUTONOMOUS_WORKERS,
            max_retries_per_work_item: DEFAULT_MAX_AUTONOMOUS_RETRIES_PER_WORK_ITEM,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardSupervision {
    pub schema: SchemaMetadata,
    pub board_id: BoardId,
    pub mode: BoardSupervisionMode,
    pub organiser: OrganiserDefaults,
    pub ticket_worker: TicketWorkerDefaults,
    pub limits: BoardSupervisionLimits,
    pub permitted_actions: BTreeSet<SupervisionAction>,
    pub configured_by: String,
    pub configured_at: String,
    pub paused_by: Option<String>,
    pub paused_at: Option<String>,
    pub revision: u64,
}

impl VersionedSchema for BoardSupervision {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupervisionPolicyResult {
    NotRequired,
    Allowed,
    Denied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupervisionDecisionOutcome {
    Pending,
    Executed,
    RecommendedForApproval,
    Denied,
    Stale,
    Paused,
    Recovered,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupervisionDecision {
    pub schema: SchemaMetadata,
    pub id: SupervisionDecisionId,
    pub board_id: BoardId,
    pub work_item_id: Option<WorkItemId>,
    pub organiser_profile_name: String,
    pub action: SupervisionAction,
    pub recommendation: String,
    pub rationale: String,
    pub policy_result: SupervisionPolicyResult,
    pub outcome: SupervisionDecisionOutcome,
    pub idempotency_key: String,
    pub expected_work_item_sequence: Option<u64>,
    pub recorded_at: String,
    pub resolved_at: Option<String>,
}

impl VersionedSchema for SupervisionDecision {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}
