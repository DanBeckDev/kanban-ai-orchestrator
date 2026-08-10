use serde::{Deserialize, Serialize};

use super::{
    BoardId, BoardSupervisionMode, SchemaMetadata, TicketEffectId, VersionedSchema, WorkItemId,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketEffectAction {
    RefineSpecification,
    GiveWorkerGuidance,
    PrepareStart,
    PrepareRestart,
    ExplainEvidence,
    ReturnForCorrection,
    RecoverInterrupted,
}

impl TicketEffectAction {
    pub const fn requires_user_decision_in_manual_mode(self) -> bool {
        !matches!(self, Self::ExplainEvidence)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketEffectOutcome {
    Pending,
    AwaitingApproval,
    Applied,
    Rejected,
    Cancelled,
    Denied,
    Stale,
    Recovered,
}

impl TicketEffectOutcome {
    pub const fn is_terminal(self) -> bool {
        !matches!(self, Self::Pending | Self::AwaitingApproval)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketEffectResolution {
    Apply,
    Reject,
    Cancel,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TicketEffectProposal {
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    pub worker_guidance: Option<String>,
    pub evidence_explanation: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketEffect {
    pub schema: SchemaMetadata,
    pub id: TicketEffectId,
    pub board_id: BoardId,
    pub work_item_id: WorkItemId,
    pub organiser_profile_name: String,
    pub action: TicketEffectAction,
    pub prompt_summary: String,
    pub recommendation: String,
    pub rationale: String,
    pub proposal: TicketEffectProposal,
    pub authority_mode: BoardSupervisionMode,
    pub supervision_revision: Option<u64>,
    pub policy_result: super::SupervisionPolicyResult,
    pub outcome: TicketEffectOutcome,
    pub idempotency_key: String,
    pub expected_work_item_sequence: u64,
    pub recorded_at: String,
    pub outcome_at: Option<String>,
}

impl VersionedSchema for TicketEffect {
    fn schema(&self) -> SchemaMetadata {
        self.schema
    }
}
